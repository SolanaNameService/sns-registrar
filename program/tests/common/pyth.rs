use std::{str::FromStr, time::UNIX_EPOCH};

use base64::Engine;
use bonfida_utils::pyth::{parse_price_v2, PRICE_FEED_DISCRIMATOR};
use borsh::BorshSerialize;
use serde::{Deserialize, Serialize};
use solana_sdk::{account::Account, pubkey::Pubkey};

#[derive(Serialize, Deserialize)]
pub struct DumpedAccount {
    pubkey: String,
    account: JsonAccount,
}

#[derive(Serialize, Deserialize)]
pub struct JsonAccount {
    lamports: u64,
    data: Vec<String>, // Assuming the data is always a Vec<String>, might need adjustment based on actual data structure and encoding.
    owner: String,
    executable: bool,
    #[serde(rename = "rentEpoch")]
    rent_epoch: u64,
    space: u64,
}

fn load_account(data: &str) -> (Account, Pubkey) {
    let dumped_acc: DumpedAccount = serde_json::from_str(data).unwrap();
    let acc: Account = Account {
        lamports: dumped_acc.account.lamports,
        data: base64::engine::general_purpose::STANDARD
            .decode(dumped_acc.account.data.first().unwrap())
            .unwrap(),
        owner: Pubkey::from_str(&dumped_acc.account.owner).unwrap(),
        executable: dumped_acc.account.executable,
        rent_epoch: dumped_acc.account.rent_epoch,
    };
    (acc, Pubkey::from_str(&dumped_acc.pubkey).unwrap())
}

pub struct PythAccounts {
    pub mapping: (Account, Pubkey),
    pub sol_product: (Account, Pubkey),
    pub sol_price: (Account, Pubkey),
    pub sol_feed_pull: (Account, Pubkey),
    pub fida_feed_pull: (Account, Pubkey),
}

pub struct Price {
    pub price: i64,
    pub conf: u64,
    pub exponent: i32,
    pub publish_time: i64,
}

pub fn load_pyth_accounts(adjust: bool) -> PythAccounts {
    let mapping = include_str!("../pyth/push/pyth_mapping.json");
    let product = include_str!("../pyth/push/sol_product.json");
    let price = include_str!("../pyth/push/sol_price.json");
    let sol_feed_pull = include_str!("../pyth/pull/sol_feed_pull.json");
    let fida_feed_pull = include_str!("../pyth/pull/fida_feed_pull.json");

    let mut sol_feed_pull = load_account(sol_feed_pull);
    let mut fida_feed_pull = load_account(fida_feed_pull);
    if adjust {
        sol_feed_pull.0.data = adjust_time(&mut sol_feed_pull.0.data);
        fida_feed_pull.0.data = adjust_time(&mut fida_feed_pull.0.data);
    }

    PythAccounts {
        mapping: load_account(mapping),
        sol_price: load_account(price),
        sol_product: load_account(product),
        sol_feed_pull,
        fida_feed_pull,
    }
}

fn adjust_time(data: &mut [u8]) -> Vec<u8> {
    let mut result: Vec<u8> = PRICE_FEED_DISCRIMATOR.to_vec();
    let mut parsed = parse_price_v2(data).unwrap();
    parsed.price_message.publish_time = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    parsed.serialize(&mut result).unwrap();
    result
}
