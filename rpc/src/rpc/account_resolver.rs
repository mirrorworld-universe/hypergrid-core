use {
    solana_runtime::bank::Bank,
    solana_sdk::{account::AccountSharedData, pubkey::Pubkey},
    std::collections::HashMap,
    sonic_printer::{show, func},
};

pub(crate) fn get_account_from_overwrites_or_bank(
    pubkey: &Pubkey,
    bank: &Bank,
    overwrite_accounts: Option<&HashMap<Pubkey, AccountSharedData>>,
) -> Option<AccountSharedData> {
    show!(file!(), line!(), func!(), overwrite_accounts);
    show!(file!(), line!(), func!(), bank.get_account(pubkey));
    overwrite_accounts
        .and_then(|accounts| accounts.get(pubkey).cloned())
        .or_else(|| bank.get_account(pubkey))
}

// Yusuf
pub(crate) fn get_account_from_remote(
    pubkey: &Pubkey,
    bank: &Bank,
    overwrite_accounts: Option<&HashMap<Pubkey, AccountSharedData>>,
) -> Option<AccountSharedData> {
    show!(file!(), line!(), func!(), overwrite_accounts);
    get_account_from_overwrites_or_bank(pubkey, bank, overwrite_accounts)
}

