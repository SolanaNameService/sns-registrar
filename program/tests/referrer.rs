use std::str::FromStr;

use sns_registrar::{constants::ROOT_DOMAIN_ACCOUNT, instruction_auto::create, processor::create};

use solana_program::{pubkey::Pubkey, system_program, sysvar};
use solana_sdk::signature::Signer;
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};
pub mod common;
use crate::common::utils::{get_vault, random_string, sign_send_instructions};
use crate::common::{ctx::TestContext, utils::parse_price_feed_fp32};

#[tokio::test]
async fn test_state() {
    let TestContext {
        mint,
        mut ctx,
        mint_authority,
        alice,
        bob,
        vault,
        pyth_accounts,
        ..
    } = TestContext::new().await;

    let mut expected_vault_amount = 0;
    let mut expected_ref_fees = 0;
    let referrer = Pubkey::from_str("3ogYncmMM5CmytsGCqKHydmXmKUZ6sGWvizkzqwT7zb1").unwrap();
    let referrer_ata = get_associated_token_address(&referrer, &mint);

    alice.create_ata(&mut ctx, &mint).await;
    bob.create_ata(&mut ctx, &mint).await;
    bob.mint_into_ata(&mut ctx, 100_000_000_000, &mint, &mint_authority)
        .await;

    // Create ref ATA
    let ix = create_associated_token_account(&ctx.payer.pubkey(), &referrer, &mint, &spl_token::ID);
    sign_send_instructions(&mut ctx, vec![ix], vec![])
        .await
        .unwrap();

    crate::common::utils::warp_to_timestamp(&mut ctx, 1682864495 + 1)
        .await
        .unwrap();

    // Test: Create domain with referrer
    let domain = random_string();
    let usd_price = sns_registrar::utils::get_usd_price(domain.len());
    let domain_key = sns_registrar::utils::get_name_key(&domain, None).unwrap();
    let reverse_key = sns_registrar::utils::get_reverse_key(&domain_key, None).unwrap();
    let ix = create(
        sns_registrar::ID,
        create::Accounts {
            naming_service_program: &spl_name_service::ID,
            root_domain: &ROOT_DOMAIN_ACCOUNT,
            reverse_lookup: &reverse_key,
            name: &domain_key,
            system_program: &system_program::ID,
            central_state: &sns_registrar::central_state::KEY,
            buyer: &bob.keypair.pubkey(),
            buyer_token_source: &bob.get_ata(&mint),
            pyth_product_acc: &Pubkey::default(),
            pyth_price_acc: &Pubkey::default(),
            pyth_mapping_acc: &pyth_accounts.fida_feed_pull.1,
            spl_token_program: &spl_token::ID,
            referrer_account_opt: Some(&referrer_ata),
            vault: &vault,
            rent_sysvar: &sysvar::rent::ID,
            state: &Pubkey::find_program_address(&[&domain_key.to_bytes()], &sns_registrar::ID).0,
        },
        create::Params {
            name: domain,
            space: 100,
            referrer_idx_opt: Some(0),
        },
    );

    sign_send_instructions(&mut ctx, vec![ix], vec![&bob.keypair])
        .await
        .unwrap();

    #[cfg(not(feature = "no-special-discount-fee"))]
    let (discount, fee) = (10, 10);
    #[cfg(feature = "no-special-discount-fee")]
    let (discount, fee) = (13, 7);

    // Verify state
    let vault_acc = get_vault(&mut ctx, &vault).await;
    let referrer_ata = get_vault(&mut ctx, &referrer_ata).await;
    let fida_usd_price_fp32 = parse_price_feed_fp32(pyth_accounts.fida_feed_pull.0, 6, 6);
    let fida_price = (usd_price << 32) / fida_usd_price_fp32;
    let fida_price = (fida_price * 95) / 100;
    let fida_price = (fida_price * (100 - discount)) / 100;
    let ref_fees = fida_price * fee / 100;
    expected_ref_fees += ref_fees;
    expected_vault_amount += fida_price - ref_fees;

    assert_eq!(vault_acc.amount, expected_vault_amount);
    assert_eq!(referrer_ata.amount, expected_ref_fees);
}
