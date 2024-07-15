use std::convert::TryInto;
use std::str::FromStr;

use bonfida_utils::pyth::parse_price_v2;
use pyth_solana_receiver_sdk::price_update::PriceFeedMessage;
use solana_program::clock::Clock;
use solana_program::instruction::Instruction;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program_test::{BanksClientError, ProgramTest, ProgramTestContext, ProgramTestError};
use solana_sdk::account::Account;
use solana_sdk::signature::Signer;
use solana_sdk::{signature::Keypair, transaction::Transaction};
use spl_associated_token_account::get_associated_token_address;
use spl_associated_token_account::instruction::create_associated_token_account;
use spl_token::state::Mint;

/// Functional testing utils

pub async fn sign_send_instructions(
    ctx: &mut ProgramTestContext,
    instructions: Vec<Instruction>,
    signers: Vec<&Keypair>,
) -> Result<(), BanksClientError> {
    let slot = ctx.banks_client.get_root_slot().await?;
    ctx.warp_to_slot(slot + 1).unwrap();
    let mut transaction = Transaction::new_with_payer(&instructions, Some(&ctx.payer.pubkey()));
    let mut payer_signers = vec![&ctx.payer];
    for s in signers {
        payer_signers.push(s);
    }
    transaction.partial_sign(&payer_signers, ctx.last_blockhash);
    ctx.banks_client.process_transaction(transaction).await
}

pub fn mint_bootstrap(
    address: Option<&str>,
    decimals: u8,
    program_test: &mut ProgramTest,
    mint_authority: &Pubkey,
) -> (Pubkey, Mint) {
    let address = address
        .map(|s| Pubkey::from_str(s).unwrap())
        .unwrap_or_else(Pubkey::new_unique);
    let mint_info = Mint {
        mint_authority: Some(*mint_authority).into(),
        supply: u32::MAX.into(),
        decimals,
        is_initialized: true,
        freeze_authority: None.into(),
    };
    let mut data = [0; Mint::LEN];
    mint_info.pack_into_slice(&mut data);
    program_test.add_account(
        address,
        Account {
            lamports: u32::MAX.into(),
            data: data.into(),
            owner: spl_token::ID,
            executable: false,
            ..Account::default()
        },
    );
    (address, mint_info)
}

pub async fn create_and_get_associated_token_address(
    ctx: &mut ProgramTestContext,
    parent_key: &Pubkey,
    mint_key: &Pubkey,
) -> Result<Pubkey, BanksClientError> {
    let instruction =
        create_associated_token_account(&ctx.payer.pubkey(), parent_key, mint_key, &spl_token::ID);
    let asset_key = get_associated_token_address(parent_key, mint_key);
    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&ctx.payer.pubkey()));
    transaction.partial_sign(&[&ctx.payer], ctx.last_blockhash);
    ctx.banks_client.process_transaction(transaction).await?;
    Ok(asset_key)
}

pub async fn warp_to_timestamp(
    ctx: &mut ProgramTestContext,
    timestamp: i64,
) -> Result<(), ProgramTestError> {
    // Set clock
    let mut clock: Clock = ctx.banks_client.get_sysvar().await.unwrap();
    clock.unix_timestamp = timestamp;
    ctx.set_sysvar(&clock);

    // Increase slot by 1
    let slot = ctx.banks_client.get_root_slot().await.unwrap();
    ctx.warp_to_slot(slot + 1).unwrap();

    Ok(())
}

pub async fn advance_clock_by(
    ctx: &mut ProgramTestContext,
    sec: i64,
) -> Result<(), ProgramTestError> {
    // Set clock
    let mut clock: Clock = ctx.banks_client.get_sysvar().await.unwrap();
    clock.unix_timestamp += sec;
    ctx.set_sysvar(&clock);

    // Increase slot by 1
    let slot = ctx.banks_client.get_root_slot().await.unwrap();
    ctx.warp_to_slot(slot + 1).unwrap();

    Ok(())
}

const CHARSET: &str = "1234567890";

pub fn random_string() -> String {
    random_string::generate(10, CHARSET)
}

pub async fn get_vault(ctx: &mut ProgramTestContext, key: &Pubkey) -> spl_token::state::Account {
    let acc = ctx.banks_client.get_account(*key).await.unwrap();
    spl_token::state::Account::unpack(&acc.unwrap().data).unwrap()
}

pub fn parse_price_feed_fp32(account: Account, quote_decimals: u32, base_decimals: u32) -> u64 {
    let PriceFeedMessage {
        price, exponent, ..
    } = parse_price_v2(&account.data).unwrap().price_message;

    let price = if exponent > 0 {
        ((price as u128) << 32) * 10u128.pow(exponent as u32)
    } else {
        ((price as u128) << 32) / 10u128.pow((-exponent) as u32)
    };

    let corrected_price = (price * 10u128.pow(quote_decimals)) / 10u128.pow(base_decimals);

    corrected_price.try_into().unwrap()
}
