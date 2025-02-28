use bonfida_utils::declare_id_with_central_state;

pub mod cpi;
pub mod entrypoint;
pub mod error;
pub mod instruction_auto;
pub mod processor;
pub mod state;
pub mod utils;

pub use error::Error;

#[cfg(not(feature = "devnet"))]
declare_id_with_central_state!("jCebN34bUfdeUYJT13J1yG16XWQpt5PDx6Mse9GUqhR");

#[cfg(feature = "devnet")]
declare_id_with_central_state!("snshBoEQ9jx4QoHBpZDQPYdNCtw7RMxJvYrKFEhwaPJ");

#[cfg(not(feature = "devnet"))]
pub mod constants {
    use phf::phf_map;
    use solana_program::{pubkey, pubkey::Pubkey};

    pub const VAULT_OWNER_DEPRECATED: Pubkey =
        pubkey!("GcWEQ9K78FV7LEHteFVciYApERk5YvQuFDQPk1yYJVXi");
    pub const VAULT_OWNER: Pubkey = pubkey!("5D2zKog251d6KPCyFyLMt3KroWwXXPWSgTPyhV22K2gR");

    pub const ROOT_DOMAIN_ACCOUNT: Pubkey = pubkey!("58PwtjSDuFHuUkYjH9BYnnQKHfwo9reZhC2zMJv9JPkx");
    pub const PYTH_MAPPING_ACCOUNT: Pubkey =
        pubkey!("AHtgzX45WTKfkPG53L6WYhGEXwQkN1BVknET3sVsLL8J");
    pub const REFERRER_WHITELIST: [Pubkey; 16] = [
        pubkey!("3ogYncmMM5CmytsGCqKHydmXmKUZ6sGWvizkzqwT7zb1"), // Test wallet
        pubkey!("DM1jJCkZZEwY5tmWbgvKRxsDFzXCdbfrYCCH1CtwguEs"), // 4Everland
        pubkey!("ADCp4QXFajHrhy4f43pD6GJFtQLkdBY2mjS9DfCk7tNW"), // Bandit network
        pubkey!("2XTgjw8yi1E3Etgj4CUyRD7Zk49gynH2U9gA5N2MY4NP"), // Altoscan
        pubkey!("5PwNeqQPiygQks9R17jUAodZQNuhvCqqkrxSaeNE8qTR"), // Solscan
        pubkey!("8kJqxAbqbPLGLMgB6FhLcnw2SiUEavx2aEGM3WQGhtJF"), // Domains labs
        pubkey!("HemvJzwxvVpWBjPETpaseAH395WAxb2G73MeUfjVkK1u"), // Solflare
        pubkey!("7hMiiUtkH4StMPJxyAtvzXTUjecTniQ8czkCPusf5eSW"), // Solnames
        pubkey!("DGpjHo4yYA3NgHvhHTp3XfBFrESsx1DnhfTr8D881ZBM"), // Brave
        pubkey!("7vWSqSw1eCXZXXUubuHWssXELNQ8MLaDgAs2ErEfCKxn"), // 585.eth
        pubkey!("5F6gcdzpw7wUjNEugdsD4aLJdEQ4Wt8d6E85vaQXZQSJ"), // wdotsol
        pubkey!("XEy9o73JBN2pEuN7aspe8mVLaWbL4ozjJs1tNRxx8bL"),  // GoDID
        pubkey!("D5cLoAGjNTHKU1UGv2bYwbnyRoGTMe3sbpLtJW3fRq91"), // SuiNS
        pubkey!("FePcCmrr7vgjeFXcXtJHqShSXydaTrga2wfHRt9RrYvP"), // Nansen
        pubkey!("5D2zKog251d6KPCyFyLMt3KroWwXXPWSgTPyhV22K2gR"), // SNS Campaign
        pubkey!("452cMqDHe5cf1Z96HxUNaQjiLckhMiZdZ5abe7oQ2iRB"), // Endless Domains
    ];
    /// Percentage of domain name creation cost trasnfered to the referrer if specified
    pub const REFERRER_FEES_PCT: u64 = 5;
    pub static TOKENS_SYM_MINT_DECIMALS: phf::Map<&'static str, (Pubkey, u8)> = phf_map! {
        "USDC" => (pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),6),
        "USDT" => (pubkey!("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"),6),
        "SOL" => (pubkey!("So11111111111111111111111111111111111111112"),9),
        "FIDA" => (pubkey!("EchesyfXePKdLtoiZSL8pBe8Myagyy8ZRqsACNCFGnvp"),6),
        "MSOL" => (pubkey!("mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So"), 9),
        "BONK" => (pubkey!("DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263"), 5),
        "BAT" => (pubkey!("EPeUFDgHRxs9xxEPVaL6kfGQvCon7jmAWKVUHuux1Tpz"), 8),
        "PYTH" => (pubkey!("HZ1JovNiVvGrGNiiYvEozEVgZ58xaU3RKwX8eACQBCt3"), 6),
        "BSOL" => (pubkey!("bSo13r4TkiE4KumL71LsHTPpL2euBYLFx6h9HP3piy1"), 9),
        "INJ" => (pubkey!("6McPRfPV6bY1e9hLxWyG54W9i9Epq75QBvXg2oetBVTB"), 8),
    };
    pub const FEES: &[u64] = &[500, 300, 200, 150, 100]; // Fees for low leverage orders for tiers [0, 1 ,2]
    pub const FEE_TIERS: [u64; 4] = [500_000_000, 5_000_000_000, 20_000_000_000, 40_000_000_000]; // Amount of FIDA tokens (with precision) that the discount account needs to hold
    pub const FIDA_MINT: Pubkey = pubkey!("EchesyfXePKdLtoiZSL8pBe8Myagyy8ZRqsACNCFGnvp");
    pub const ADMIN: Pubkey = pubkey!("VBx642K1hYGLU5Zm1CHW1uRXAtFgxN5mRqyMcXnLZFW");
    pub const AUCTION_PROGRAM_ID: Pubkey = pubkey!("AVWV7vdWbLqXiLKFaP19GhYurhwxaLp2qRBSjT5tR5vT");
    pub const WOLVES_COLLECTION: Pubkey = pubkey!("Dw74YSxTKVXsztPm3TmwbnfLK8KVaCZw69jVu4LE6uJe");
    pub const WOLVES_COLLECTION_METADATA: Pubkey =
        pubkey!("72aLKvXeV4aansAQtxKymeXDevT5ed6sCuz9iN62ugPT");

    // For marketing campaigns with partners
    // referrer public key -> (discount_pct, start_time, end_time, referrer_pct, start_time, end_time)
    // Example (Some(5), 1682864495, 1685060126, Some(10), 1682864495, 1685060126)
    pub type SpecialDiscountFee = (Option<u8>, u64, u64, Option<u8>, u64, u64);
    pub static REFERRER_DISCOUNT_AND_FEE: phf::Map<&'static str, SpecialDiscountFee> = phf_map! {
        // Test wallet
        "3ogYncmMM5CmytsGCqKHydmXmKUZ6sGWvizkzqwT7zb1" => (Some(10), 1682864495, 1719226965, Some(10), 1682864495, 1719226965),
        // Solscan
        "5PwNeqQPiygQks9R17jUAodZQNuhvCqqkrxSaeNE8qTR" => (None, 0, 0, Some(20), 0, u64::MAX),
        // Domain Labs
        "8kJqxAbqbPLGLMgB6FhLcnw2SiUEavx2aEGM3WQGhtJF" => (Some(20), 1691544598, 1692072000, Some(15), 0, u64::MAX),
        // Solflare
        "HemvJzwxvVpWBjPETpaseAH395WAxb2G73MeUfjVkK1u" => (None, 0, 0, Some(15), 1695942000, u64::MAX),
        // Brave
        "DGpjHo4yYA3NgHvhHTp3XfBFrESsx1DnhfTr8D881ZBM" => (None, 0, 0, Some(20), 0, u64::MAX),
        // 585.eth
        "7vWSqSw1eCXZXXUubuHWssXELNQ8MLaDgAs2ErEfCKxn" => (None, 0, 0, Some(20), 0, u64::MAX),
        // wdotsol
        "5F6gcdzpw7wUjNEugdsD4aLJdEQ4Wt8d6E85vaQXZQSJ" => (None, 0, 0, Some(20), 0, u64::MAX),
        // SuiNS
        "D5cLoAGjNTHKU1UGv2bYwbnyRoGTMe3sbpLtJW3fRq91" => (Some(20), 0, 1730311200, None, 0, 0),
        // Nansen
        "FePcCmrr7vgjeFXcXtJHqShSXydaTrga2wfHRt9RrYvP" => (Some(20), 0, 1731636000, None, 0, 0),
        // SNS Campaign
        "5D2zKog251d6KPCyFyLMt3KroWwXXPWSgTPyhV22K2gR" => (Some(50), 1737266400, 1737608460, None, 0, 0),
    };
    // Fees taken for the reselling of domain names
    // | Tier | Percentage of payout    | Requirements   |
    // | ---- | ----------------------- | -------------- |
    // | 0    | 5%                      | None           |
    // | 1    | 3%                      | 500 FIDA       |
    // | 2    | 2%                      | 5,000 FIDA     |
    // | 3    | 1.5%                    | 20,000 FIDA    |
    // | 4    | 1%                      | 40,000 FIDA    |
}

#[cfg(feature = "devnet")]
pub mod constants {
    use phf::phf_map;
    use solana_program::{pubkey, pubkey::Pubkey};

    pub const VAULT_OWNER_DEPRECATED: Pubkey =
        pubkey!("SNSaTJbEv2iT3CUrCQYa9zpGjbBVWhFCPaSJHkaJX34");
    pub const VAULT_OWNER: Pubkey = pubkey!("SNSaTJbEv2iT3CUrCQYa9zpGjbBVWhFCPaSJHkaJX34");

    pub const ROOT_DOMAIN_ACCOUNT: Pubkey = pubkey!("5eoDkP6vCQBXqDV9YN2NdUs3nmML3dMRNmEYpiyVNBm2");
    // Unused
    pub const PYTH_MAPPING_ACCOUNT: Pubkey =
        pubkey!("BmA9Z6FjioHJPpjT39QazZyhDRUdZy2ezwx4GiDdE2u2");
    pub const REFERRER_WHITELIST: [Pubkey; 5] = [
        pubkey!("3ogYncmMM5CmytsGCqKHydmXmKUZ6sGWvizkzqwT7zb1"), // Test wallet
        pubkey!("DM1jJCkZZEwY5tmWbgvKRxsDFzXCdbfrYCCH1CtwguEs"), // 4Everland
        pubkey!("ADCp4QXFajHrhy4f43pD6GJFtQLkdBY2mjS9DfCk7tNW"), // Bandit network
        pubkey!("2XTgjw8yi1E3Etgj4CUyRD7Zk49gynH2U9gA5N2MY4NP"), // Altoscan
        pubkey!("5oDWj8vr3vbcq9JZTtwXqrkCMZggMsDzNietvbr1BNfe"), // Solscan
    ];
    /// Percentage of domain name creation cost transfered to the referrer if specified
    pub const REFERRER_FEES_PCT: u64 = 5;
    pub static TOKENS_SYM_MINT_DECIMALS: phf::Map<&'static str, (Pubkey, u8)> = phf_map! {
        "USDC" => (pubkey!("4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU"),6),
        "USDT" => (pubkey!("EJwZgeZrdC8TXTQbQBoL6bfuAnFUUy1PVCMB4DYPzVaS"),6), // Saber devnet
        "SOL" => (pubkey!("So11111111111111111111111111111111111111112"),9),
        "FIDA" => (pubkey!("fidaWCioBQjieRrUQDxxS5Uxmq1CLi2VuVRyv4dEBey"),6),
        "INJ" => (pubkey!("DL4ivZm3NVHWk9ZvtcqTchxoKArDK4rT3vbDx2gYVr7P"), 8),
    };
    pub const FEES: &[u64] = &[500, 300, 200, 150, 100]; // Fees for low leverage orders for tiers [0, 1 ,2]
    pub const FEE_TIERS: [u64; 4] = [500_000_000, 5_000_000_000, 20_000_000_000, 40_000_000_000]; // Amount of FIDA tokens (with precision) that the discount account needs to hold
    pub const FIDA_MINT: Pubkey = pubkey!("fidaWCioBQjieRrUQDxxS5Uxmq1CLi2VuVRyv4dEBey");
    pub const ADMIN: Pubkey = pubkey!("SNSaTJbEv2iT3CUrCQYa9zpGjbBVWhFCPaSJHkaJX34");
    pub const AUCTION_PROGRAM_ID: Pubkey = pubkey!("AVWV7vdWbLqXiLKFaP19GhYurhwxaLp2qRBSjT5tR5vT");
    pub const WOLVES_COLLECTION: Pubkey = pubkey!("Dw74YSxTKVXsztPm3TmwbnfLK8KVaCZw69jVu4LE6uJe");
    pub const WOLVES_COLLECTION_METADATA: Pubkey =
        pubkey!("72aLKvXeV4aansAQtxKymeXDevT5ed6sCuz9iN62ugPT");

    // For marketing campaigns with partners
    // referrer public key -> (discount_pct, start_time, end_time, referrer_pct, start_time, end_time)
    // Example (Some(5), 1682864495, 1685060126, Some(10), 1682864495, 1685060126)
    pub type SpecialDiscountFee = (Option<u8>, u64, u64, Option<u8>, u64, u64);
    pub static REFERRER_DISCOUNT_AND_FEE: phf::Map<&'static str, SpecialDiscountFee> = phf_map! {
        // Test wallet
        "3ogYncmMM5CmytsGCqKHydmXmKUZ6sGWvizkzqwT7zb1" => (Some(10), 1682864495, 1779756628, Some(10), 1682864495, 1779756628),
        // Bandit network
        "ADCp4QXFajHrhy4f43pD6GJFtQLkdBY2mjS9DfCk7tNW" => (Some(20), 1686700800, 1687046399, Some(0), 0, u64::MAX),
        // Solscan
        "5oDWj8vr3vbcq9JZTtwXqrkCMZggMsDzNietvbr1BNfe" => (None, 0, 0, Some(20), 0, u64::MAX)
    };
    // Fees taken for the reselling of domain names
    // | Tier | Percentage of payout    | Requirements   |
    // | ---- | ----------------------- | -------------- |
    // | 0    | 5%                      | None           |
    // | 1    | 3%                      | 500 FIDA       |
    // | 2    | 2%                      | 5,000 FIDA     |
    // | 3    | 1.5%                    | 20,000 FIDA    |
    // | 4    | 1%                      | 40,000 FIDA    |
}

#[cfg(not(feature = "no-entrypoint"))]
solana_security_txt::security_txt! {
    name: env!("CARGO_PKG_NAME"),
    project_url: "http://bonfida.org",
    contacts: "email:security@bonfida.com,link:https://twitter.com/bonfida",
    policy: "https://immunefi.com/bounty/bonfida",
    preferred_languages: "en",
    auditors: "Halborn"
}
