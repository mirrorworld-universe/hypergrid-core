use {
    base64::{self, Engine}, 
    dashmap::DashMap, // DashSet}, 
    // futures::{self, future::Remote, FutureExt}, 
    reqwest::{
        self,
        header::{
            self, 
            CONTENT_TYPE,
            // RETRY_AFTER
        }, 
    }, serde_json::json, 
    solana_sdk::{
        account::{AccountSharedData, WritableAccount}, 
        pubkey::Pubkey,
        // clock::Slot,
    }, solana_version, 
    std::{
        str::FromStr, 
        thread, 
        time::Duration, //{Duration, Instant},
    }, 
    // tokio::{self, runtime}, 
    zstd,
};

type AccountCacheKeyMap = DashMap<Pubkey, AccountSharedData>;

#[derive(Debug)]
pub struct RemoteAccountLoader {
    /// HTTP client used to send requests to the remote.
    client: reqwest::blocking::Client,
    // client: reqwest::Client,
    /// URL of the remote to load accounts from.
    url: String,
    /// Cache of accounts loaded from the remote.
    account_cache: AccountCacheKeyMap,
    /// Enable or disable the remote loader.
    enable: bool,
}

impl Default for RemoteAccountLoader {
    fn default() -> Self {
        Self::new("http://rpc.hypergrid.dev")
        // Self::new("https://api.devnet.solana.com/")
    }
}

/// blocking [`RemoteLoader`] over HTTP.
impl RemoteAccountLoader {   
    pub fn new<U: ToString>(url: U) -> Self {
        Self::new_with_timeout(url, Duration::from_secs(30))
    }

    pub fn new_with_timeout<U: ToString>(url: U, timeout: Duration) -> Self {
        Self {
            url: url.to_string(),
            client: reqwest::blocking::Client::builder()
                .default_headers(Self::default_headers())
                .timeout(timeout)
                .pool_idle_timeout(timeout)
                .build()
                .expect("build rpc client"),
            // client: reqwest::Client::builder()
            //     .default_headers(Self::default_headers())
            //     .timeout(timeout)
            //     .pool_idle_timeout(timeout)
            //     .build()
            //     .expect("build rpc client"),
            account_cache: AccountCacheKeyMap::default(),
            enable: true,
        }
    }

    /// Create default headers used by HTTP Sender.
    fn default_headers() -> header::HeaderMap {
        let mut default_headers = header::HeaderMap::new();
        default_headers.append(
            header::HeaderName::from_static("solana-client"),
            header::HeaderValue::from_str(
                format!("rust/{}", solana_version::Version::default()).as_str(),
            )
            .unwrap(),
        );
        default_headers
    }

    fn ignored_account(pubkey: &Pubkey) -> bool {
        let pk = pubkey.to_string();
        if pk.contains("1111111111111111") 
            // || pk.starts_with("Memo") 
            // || pk.starts_with("Token") 
            // || pk.starts_with("AToken") 
        {
            return true;
        }
        false
    }
    pub fn get_account(&self, pubkey: &Pubkey) -> Option<AccountSharedData> {
        if !self.enable || Self::ignored_account(pubkey) {
            return None;
        }
        println!("RemoteAccountLoader.get_account: {}", pubkey.to_string());
        match self.account_cache.get(pubkey) {
            Some(account) =>    {
                println!("RemoteAccountLoader.get_account: {} match.", pubkey.to_string());
                return Some(account.clone());
            },
            None => self.load_account(pubkey),
        }
    }

    pub fn has_account(&self, pubkey: &Pubkey) -> bool {
        if !self.enable || Self::ignored_account(pubkey) {
            return false;
        }
        println!("RemoteAccountLoader.has_account: {}", pubkey.to_string());
        match self.account_cache.contains_key(pubkey) {
            true => true,
            false => self.load_account(pubkey).is_some(),
        }
    }

    fn deserialize_from_json(account_data: serde_json::Value) -> Option<AccountSharedData> {
        let result = &account_data["result"];
        if result.is_null() {
            return None;
        }
        
        let value = &result["value"];
        if value.is_null() {
            return None;
        }
   
        // println!("data: {:?}", account_data.to_string());
        // let slot = result["context"]["slot"].as_u64().unwrap_or(0);
        let data = value["data"][0].as_str().unwrap_or("");
        let encoding = value["data"][1].as_str().unwrap_or("");
        let lamports = value["lamports"].as_u64().unwrap_or(0);
        let owner = value["owner"].as_str().unwrap_or("");
        let rent_epoch = value["rentEpoch"].as_u64().unwrap_or(0);
        let space = value["space"].as_u64().unwrap();
        let executable = value["executable"].as_bool().unwrap_or(false);
        // if owner.eq("Feature111111111111111111111111111111111111") {
        //     return None;
        // }

        let data = match encoding {
            "base58" => bs58::decode(data).into_vec().unwrap_or_default(),
            "base64" => base64::engine::general_purpose::STANDARD.decode(data).unwrap_or_default(),
            "base64+zstd" => {
                let decoded = base64::engine::general_purpose::STANDARD.decode(data).unwrap_or_default();
                let decompressed = zstd::decode_all(decoded.as_slice()).unwrap_or_default();
                decompressed
            },
            _ => Vec::new(), // Add wildcard pattern to cover all other possible values
        };
    
        println!("data: {}, {}", space, data.len());
    
        let mut account = AccountSharedData::create(
                lamports,
                data,
                Pubkey::from_str(owner).unwrap(),
                executable,
                rent_epoch
        );
        account.remote = true;
    
        println!("account: {:?}", account);
        Some(account)
    }
    

    pub fn load_account(&self, pubkey: &Pubkey) -> Option<AccountSharedData> {
        if !self.enable || Self::ignored_account(pubkey) {
            return None;
        }
        self.load_account_from_remote(pubkey)
        // self.load_account_from_tests(pubkey)
    }

    fn load_account_from_remote(&self, pubkey: &Pubkey) -> Option<AccountSharedData> {
        if Self::ignored_account(pubkey) {
            print!("******* skip: {}\n", pubkey.to_string());
            return None;
        }
        println!("load_account_from_remote: {}", pubkey.to_string());

        let req = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getAccountInfo",
            "params": [
                pubkey.to_string(),
                {
                    "encoding": "base64+zstd" //"base58"
                }
            ]
        });

        let client = self.client.clone();
        let url = self.url.clone();
        let call = thread::spawn(move || {
            let res = client.post(url)
                .header(CONTENT_TYPE, "application/json")
                .body(req.to_string())
                .send().unwrap();
            println!("load_account_from_remote, response: {:?}", res.status());
            if res.status().is_success() {
                let account_json: serde_json::Value = res.json().unwrap();
                // println!("load_account_from_remote 1: {:?}", account_json);
                RemoteAccountLoader::deserialize_from_json(account_json)
            } else {
                None
            }
        });
        let result = call.join().unwrap();
        match result {
            Some(account) => {
                self.account_cache.insert(pubkey.clone(), account.clone());
                Some(account)
            },
            None => {
                println!("load_account_from_remote: failed to load account: {:?}\n", pubkey);
                None
            }
        }
        
        
    }

}
