use {
    crate::{
        nonblocking::{rpc_client::RpcClient, tpu_client::TpuClient},
        rpc_client::RpcClient as BlockingRpcClient,
    },
    bincode::serialize,
    dashmap::DashMap,
    futures_util::future::{join_all, FutureExt},
    solana_quic_client::{QuicConfig, QuicConnectionManager, QuicPool},
    solana_rpc_client::spinner::{self, SendTransactionProgress},
    solana_rpc_client_api::{
        client_error::ErrorKind,
        request::{RpcError, RpcResponseErrorData, MAX_GET_SIGNATURE_STATUSES_QUERY_ITEMS},
        response::RpcSimulateTransactionResult,
    },
    solana_sdk::{
        hash::Hash,
        message::Message,
        signature::{Signature, SignerError},
        signers::Signers,
        transaction::{Transaction, TransactionError},
    },
    solana_tpu_client::tpu_client::{Result, TpuSenderError},
    std::{
        sync::{
            atomic::{AtomicU64, AtomicUsize, Ordering},
            Arc,
        },
        time::Duration,
    },
    tokio::{sync::RwLock, task::JoinHandle, time::Instant},
    sonic_printer::{func, show},
};

// Sonic Rate
const BLOCKHASH_REFRESH_RATE: Duration = Duration::from_secs(5); //default 5
const TPU_RESEND_REFRESH_RATE: Duration = Duration::from_secs(2); //default 2
const SEND_INTERVAL: Duration = Duration::from_millis(50); //default 10
type QuicTpuClient = TpuClient<QuicPool, QuicConnectionManager, QuicConfig>;

#[derive(Clone, Debug)]
struct TransactionData {
    last_valid_block_height: u64,
    message: Message,
    index: usize,
    serialized_transaction: Vec<u8>,
}

#[derive(Clone, Debug, Copy)]
struct BlockHashData {
    pub blockhash: Hash,
    pub last_valid_block_height: u64,
}

#[derive(Clone, Debug, Copy)]
pub struct SendAndConfirmConfig {
    pub with_spinner: bool,
    pub resign_txs_count: Option<usize>,
}

/// Sends and confirms transactions concurrently in a sync context
pub fn send_and_confirm_transactions_in_parallel_blocking<T: Signers + ?Sized>(
    rpc_client: Arc<BlockingRpcClient>,
    tpu_client: Option<QuicTpuClient>,
    messages: &[Message],
    signers: &T,
    config: SendAndConfirmConfig,
) -> Result<Vec<Option<TransactionError>>> {
    show!(file!(), line!(), func!(), "mark");
    let fut = send_and_confirm_transactions_in_parallel(
        rpc_client.get_inner_client().clone(),
        tpu_client,
        messages,
        signers,
        config,
    );
    show!(file!(), line!(), func!(), "mark");
    tokio::task::block_in_place(|| rpc_client.runtime().block_on(fut))
}

fn create_blockhash_data_updating_task(
    rpc_client: Arc<RpcClient>,
    blockhash_data_rw: Arc<RwLock<BlockHashData>>,
    current_block_height: Arc<AtomicU64>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            if let Ok((blockhash, last_valid_block_height)) = rpc_client
                .get_latest_blockhash_with_commitment(rpc_client.commitment())
                .await
            {
                *blockhash_data_rw.write().await = BlockHashData {
                    blockhash,
                    last_valid_block_height,
                };
            }

            if let Ok(block_height) = rpc_client.get_block_height().await {
                current_block_height.store(block_height, Ordering::Relaxed);
            }
            tokio::time::sleep(BLOCKHASH_REFRESH_RATE).await;
        }
    })
}

fn create_transaction_confirmation_task(
    rpc_client: Arc<RpcClient>,
    current_block_height: Arc<AtomicU64>,
    unconfirmed_transaction_map: Arc<DashMap<Signature, TransactionData>>,
    errors_map: Arc<DashMap<usize, TransactionError>>,
    num_confirmed_transactions: Arc<AtomicUsize>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        // check transactions that are not expired or have just expired between two checks
        let mut last_block_height = current_block_height.load(Ordering::Relaxed);

        loop {
            if !unconfirmed_transaction_map.is_empty() {
                let current_block_height = current_block_height.load(Ordering::Relaxed);
                let transactions_to_verify: Vec<Signature> = unconfirmed_transaction_map
                    .iter()
                    .filter(|x| {
                        let is_not_expired = current_block_height <= x.last_valid_block_height;
                        // transaction expired between last and current check
                        let is_recently_expired = last_block_height <= x.last_valid_block_height
                            && current_block_height > x.last_valid_block_height;
                        is_not_expired || is_recently_expired
                    })
                    .map(|x| *x.key())
                    .collect();
                for signatures in
                    transactions_to_verify.chunks(MAX_GET_SIGNATURE_STATUSES_QUERY_ITEMS)
                {
                    if let Ok(result) = rpc_client.get_signature_statuses(signatures).await {
                        let statuses = result.value;
                        for (signature, status) in signatures.iter().zip(statuses.into_iter()) {
                            if let Some((status, data)) = status
                                .filter(|status| {
                                    status.satisfies_commitment(rpc_client.commitment())
                                })
                                .and_then(|status| {
                                    unconfirmed_transaction_map
                                        .remove(signature)
                                        .map(|(_, data)| (status, data))
                                })
                            {
                                num_confirmed_transactions.fetch_add(1, Ordering::Relaxed);
                                if let Some(error) = status.err {
                                    errors_map.insert(data.index, error);
                                }
                            };
                        }
                    }
                }

                last_block_height = current_block_height;
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    })
}

#[derive(Clone, Debug)]
struct SendingContext {
    unconfirmed_transaction_map: Arc<DashMap<Signature, TransactionData>>,
    error_map: Arc<DashMap<usize, TransactionError>>,
    blockhash_data_rw: Arc<RwLock<BlockHashData>>,
    num_confirmed_transactions: Arc<AtomicUsize>,
    total_transactions: usize,
    current_block_height: Arc<AtomicU64>,
}
fn progress_from_context_and_block_height(
    context: &SendingContext,
    last_valid_block_height: u64,
) -> SendTransactionProgress {
    SendTransactionProgress {
        confirmed_transactions: context
            .num_confirmed_transactions
            .load(std::sync::atomic::Ordering::Relaxed),
        total_transactions: context.total_transactions,
        block_height: context
            .current_block_height
            .load(std::sync::atomic::Ordering::Relaxed),
        last_valid_block_height,
    }
}

async fn send_transaction_with_rpc_fallback(
    rpc_client: &RpcClient,
    tpu_client: &Option<QuicTpuClient>,
    transaction: Transaction,
    serialized_transaction: Vec<u8>,
    context: &SendingContext,
    index: usize,
) -> Result<()> {
    let send_over_rpc = if let Some(tpu_client) = tpu_client {
        !tpu_client
            .send_wire_transaction(serialized_transaction.clone())
            .await
    } else {
        true
    };
    if send_over_rpc {
        if let Err(e) = rpc_client.send_transaction(&transaction).await {
            match &e.kind {
                ErrorKind::Io(_) | ErrorKind::Reqwest(_) => {
                    // fall through on io error, we will retry the transaction
                }
                ErrorKind::TransactionError(TransactionError::BlockhashNotFound)
                | ErrorKind::RpcError(RpcError::RpcResponseError {
                    data:
                        RpcResponseErrorData::SendTransactionPreflightFailure(
                            RpcSimulateTransactionResult {
                                err: Some(TransactionError::BlockhashNotFound),
                                ..
                            },
                        ),
                    ..
                }) => {
                    // fall through so that we will resend with another blockhash
                }
                ErrorKind::TransactionError(transaction_error)
                | ErrorKind::RpcError(RpcError::RpcResponseError {
                    data:
                        RpcResponseErrorData::SendTransactionPreflightFailure(
                            RpcSimulateTransactionResult {
                                err: Some(transaction_error),
                                ..
                            },
                        ),
                    ..
                }) => {
                    // if we get other than blockhash not found error the transaction is invalid
                    context.error_map.insert(index, transaction_error.clone());
                }
                _ => {
                    return Err(TpuSenderError::from(e));
                }
            }
        }
    }
    Ok(())
}

async fn sign_all_messages_and_send<T: Signers + ?Sized>(
    progress_bar: &Option<indicatif::ProgressBar>,
    rpc_client: &RpcClient,
    tpu_client: &Option<QuicTpuClient>,
    messages_with_index: Vec<(usize, Message)>,
    signers: &T,
    context: &SendingContext,
) -> Result<()> {
    let current_transaction_count = messages_with_index.len();
    let mut futures = vec![];
    // send all the transaction messages
    for (counter, (index, message)) in messages_with_index.iter().enumerate() {
        let mut transaction = Transaction::new_unsigned(message.clone());
        let blockhashdata = *context.blockhash_data_rw.read().await;

        // we have already checked if all transactions are signable.
        transaction
            .try_sign(signers, blockhashdata.blockhash)
            .expect("Transaction should be signable");
        let serialized_transaction = serialize(&transaction).expect("Transaction should serialize");
        let signature = transaction.signatures[0];
        futures.push(async move {
            tokio::time::sleep(SEND_INTERVAL.saturating_mul(counter as u32)).await;
            // send to confirm the transaction
            context.unconfirmed_transaction_map.insert(
                signature,
                TransactionData {
                    index: *index,
                    serialized_transaction: serialized_transaction.clone(),
                    last_valid_block_height: blockhashdata.last_valid_block_height,
                    message: message.clone(),
                },
            );
            if let Some(progress_bar) = progress_bar {
                let progress = progress_from_context_and_block_height(
                    context,
                    blockhashdata.last_valid_block_height,
                );
                progress.set_message_for_confirmed_transactions(
                    progress_bar,
                    &format!(
                        "Sending {}/{} transactions",
                        counter + 1,
                        current_transaction_count,
                    ),
                );
            }
            send_transaction_with_rpc_fallback(
                rpc_client,
                tpu_client,
                transaction,
                serialized_transaction,
                context,
                *index,
            )
            .await
        });
    }
    // collect to convert Vec<Result<_>> to Result<Vec<_>>
    join_all(futures).await.into_iter().collect::<Result<_>>()?;
    Ok(())
}

async fn confirm_transactions_till_block_height_and_resend_unexpired_transaction_over_tpu(
    progress_bar: &Option<indicatif::ProgressBar>,
    tpu_client: &Option<QuicTpuClient>,
    context: &SendingContext,
) {
    show!(file!(), line!(), func!(), "mark");
    let unconfirmed_transaction_map = context.unconfirmed_transaction_map.clone();
    let current_block_height = context.current_block_height.clone();

    let transactions_to_confirm = unconfirmed_transaction_map.len();
    let max_valid_block_height = unconfirmed_transaction_map
        .iter()
        .map(|x| x.last_valid_block_height)
        .max();
    show!(file!(), line!(), func!(), "mark");
    if let Some(mut max_valid_block_height) = max_valid_block_height {
        if let Some(progress_bar) = progress_bar {
            show!(file!(), line!(), func!(), "mark");
            let progress = progress_from_context_and_block_height(context, max_valid_block_height);
            progress.set_message_for_confirmed_transactions(
                progress_bar,
                &format!(
                    "Waiting for next block, {transactions_to_confirm} transactions pending..."
                ),
            );
        }
        show!(file!(), line!(), func!(), "mark");
        if let Some(progress_bar) = progress_bar {
            show!(file!(), line!(), func!(), "mark");
            let progress = progress_from_context_and_block_height(context, max_valid_block_height);
            progress.set_message_for_confirmed_transactions(
                progress_bar,
                "Checking transaction status...",
            );
        }

        // wait till all transactions are confirmed or we have surpassed max processing age for the last sent transaction
        while !unconfirmed_transaction_map.is_empty()
            && current_block_height.load(Ordering::Relaxed) <= max_valid_block_height
        {
            let block_height = current_block_height.load(Ordering::Relaxed);

            if let Some(tpu_client) = tpu_client {
                let instant = Instant::now();
                // retry sending transaction only over TPU port
                // any transactions sent over RPC will be automatically rebroadcast by the RPC server
                let txs_to_resend_over_tpu = unconfirmed_transaction_map
                    .iter()
                    .filter(|x| block_height < x.last_valid_block_height)
                    .map(|x| x.serialized_transaction.clone())
                    .collect::<Vec<_>>();
                let num_txs_to_resend = txs_to_resend_over_tpu.len();
                show!(file!(), line!(), func!(), &num_txs_to_resend);
                // show!(file!(), line!(), func!(), &txs_to_resend_over_tpu);
                // This is a "reasonable" constant for how long it should
                // take to fan the transactions out, taken from
                // `solana_tpu_client::nonblocking::tpu_client::send_wire_transaction_futures`
                const SEND_TIMEOUT_INTERVAL: Duration = Duration::from_secs(5);
                show!(file!(), line!(), func!(), "mark");
                let message = if tokio::time::timeout(
                    SEND_TIMEOUT_INTERVAL,
                    tpu_client.try_send_wire_transaction_batch(txs_to_resend_over_tpu),
                )
                .await
                .is_err()
                {
                    show!(file!(), line!(), func!(), "mark");
                    format!("Timed out resending {num_txs_to_resend} transactions...")
                } else {
                    show!(file!(), line!(), func!(), "mark");
                    format!("Resent {num_txs_to_resend} transactions...")
                };
                show!(file!(), line!(), func!(), "mark");
                if let Some(progress_bar) = progress_bar {
                    let progress =
                        progress_from_context_and_block_height(context, max_valid_block_height);
                    progress.set_message_for_confirmed_transactions(progress_bar, &message);
                }
                show!(file!(), line!(), func!(), "mark");
                let elapsed = instant.elapsed();
                if elapsed < TPU_RESEND_REFRESH_RATE {
                    show!(file!(), line!(), func!(), TPU_RESEND_REFRESH_RATE - elapsed);
                    tokio::time::sleep(TPU_RESEND_REFRESH_RATE - elapsed).await;
                }
            } else {
                show!(file!(), line!(), func!(), "mark");
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            show!(file!(), line!(), func!(), "mark");
            if let Some(max_valid_block_height_in_remaining_transaction) =
                unconfirmed_transaction_map
                    .iter()
                    .map(|x| x.last_valid_block_height)
                    .max()
            {
                
                max_valid_block_height = max_valid_block_height_in_remaining_transaction;
                show!(file!(), line!(), func!(), max_valid_block_height);
                

            }
        }
    }
}

/// Sends and confirms transactions concurrently
///
/// The sending and confirmation of transactions is done in parallel tasks
/// The method signs transactions just before sending so that blockhash does not
/// expire.
pub async fn send_and_confirm_transactions_in_parallel<T: Signers + ?Sized>(
    rpc_client: Arc<RpcClient>,
    tpu_client: Option<QuicTpuClient>,
    messages: &[Message],
    signers: &T,
    config: SendAndConfirmConfig,
) -> Result<Vec<Option<TransactionError>>> {
    // get current blockhash and corresponding last valid block height
    show!(file!(), line!(), func!(), "mark");
    let (blockhash, last_valid_block_height) = rpc_client
        .get_latest_blockhash_with_commitment(rpc_client.commitment())
        .await?;
    show!(file!(), line!(), func!(), "mark");
    let blockhash_data_rw = Arc::new(RwLock::new(BlockHashData {
        blockhash,
        last_valid_block_height,
    }));
    show!(file!(), line!(), func!(), "mark");
    // check if all the messages are signable by the signers
    messages
        .iter()
        .map(|x| {
            let mut transaction = Transaction::new_unsigned(x.clone());
            transaction.try_sign(signers, blockhash)
        })
        .collect::<std::result::Result<Vec<()>, SignerError>>()?;
    show!(file!(), line!(), func!(), "mark");
    // get current block height
    let block_height = rpc_client.get_block_height().await?;
    show!(file!(), line!(), func!(), "mark");
    let current_block_height = Arc::new(AtomicU64::new(block_height));
    show!(file!(), line!(), func!(), "mark");     
    let progress_bar = config.with_spinner.then(|| {
        show!(file!(), line!(), func!(), "mark");
        let progress_bar = spinner::new_progress_bar();
        show!(file!(), line!(), func!(), "mark");
        progress_bar.set_message("Setting up...");
        show!(file!(), line!(), func!(), "mark");
        progress_bar
    });
    show!(file!(), line!(), func!(), "mark");
    // blockhash and block height update task
    let block_data_task = create_blockhash_data_updating_task(
        rpc_client.clone(),
        blockhash_data_rw.clone(),
        current_block_height.clone(),
    );
    show!(file!(), line!(), func!(), "mark");
    let unconfirmed_transasction_map = Arc::new(DashMap::<Signature, TransactionData>::new());
    let error_map = Arc::new(DashMap::new());
    let num_confirmed_transactions = Arc::new(AtomicUsize::new(0));
    // tasks which confirms the transactions that were sent
    show!(file!(), line!(), func!(), "mark");
    let transaction_confirming_task = create_transaction_confirmation_task(
        rpc_client.clone(),
        current_block_height.clone(),
        unconfirmed_transasction_map.clone(),
        error_map.clone(),
        num_confirmed_transactions.clone(),
    );
    show!(file!(), line!(), func!(), "mark");
    // transaction sender task
    let total_transactions = messages.len();
    show!(file!(), line!(), func!(), "mark");
    let mut initial = true;
    show!(file!(), line!(), func!(), "mark");
    let signing_count = config.resign_txs_count.unwrap_or(1);
    show!(file!(), line!(), func!(), "mark");
    let context = SendingContext {
        unconfirmed_transaction_map: unconfirmed_transasction_map.clone(),
        blockhash_data_rw: blockhash_data_rw.clone(),
        num_confirmed_transactions: num_confirmed_transactions.clone(),
        current_block_height: current_block_height.clone(),
        error_map: error_map.clone(),
        total_transactions,
    };
    show!(file!(), line!(), func!(), signing_count);
    for expired_blockhash_retries in (0..signing_count).rev() {
        // only send messages which have not been confirmed
        show!(file!(), line!(), func!(), "mark");
        let messages_with_index: Vec<(usize, Message)> = if initial {
            initial = false;
            messages.iter().cloned().enumerate().collect()
        } else {
            // remove all the confirmed transactions
            unconfirmed_transasction_map
                .iter()
                .map(|x| (x.index, x.message.clone()))
                .collect()
        };
        show!(file!(), line!(), func!(), "mark");
        if messages_with_index.is_empty() {
            show!(file!(), line!(), func!(), "mark");
            break;
        }

        // clear the map so that we can start resending
        unconfirmed_transasction_map.clear();
        show!(file!(), line!(), func!(), "mark");
        let futures = [
            sign_all_messages_and_send(
                &progress_bar,
                &rpc_client,
                &tpu_client,
                messages_with_index,
                signers,
                &context,
            )
            .boxed_local(),
            async {
                // Give the signing and sending a head start before trying to
                // confirm and resend
                show!(file!(), line!(), func!(), "mark");
                tokio::time::sleep(TPU_RESEND_REFRESH_RATE).await;
                show!(file!(), line!(), func!(), "mark");
                confirm_transactions_till_block_height_and_resend_unexpired_transaction_over_tpu(
                    &progress_bar,
                    &tpu_client,
                    &context,
                )
                .await;
                // Infallible, but required to have the same return type as
                // `sign_all_messages_and_send`
                show!(file!(), line!(), func!(), "mark");
                Ok(())
            }
            .boxed_local(),
        ];
        show!(file!(), line!(), func!(), "mark");
        join_all(futures).await.into_iter().collect::<Result<_>>()?;
        show!(file!(), line!(), func!(), "mark");
        if unconfirmed_transasction_map.is_empty() {
            show!(file!(), line!(), func!(), "mark");
            break;
        }
        show!(file!(), line!(), func!(), "mark");
        if let Some(progress_bar) = &progress_bar {
            progress_bar.println(format!(
                "Blockhash expired. {expired_blockhash_retries} retries remaining"
            ));
        }
    }
    show!(file!(), line!(), func!(), "mark");
    block_data_task.abort();
    transaction_confirming_task.abort();
    show!(file!(), line!(), func!(), "mark");
    if unconfirmed_transasction_map.is_empty() {
        let mut transaction_errors = vec![None; messages.len()];
        for iterator in error_map.iter() {
            transaction_errors[*iterator.key()] = Some(iterator.value().clone());
        }
        Ok(transaction_errors)
    } else {
        Err(TpuSenderError::Custom("Max retries exceeded".into()))
    }
}
