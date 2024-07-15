use borsh::BorshSerialize;
use sns_registrar::{
    constants::{ROOT_DOMAIN_ACCOUNT, TOKENS_SYM_MINT_DECIMALS, VAULT_OWNER},
    instruction_auto::{create, create_split_v2},
    processor::{create, create_reverse, create_split_v2},
    utils::get_reverse_key,
};

use solana_program::{program_pack::Pack, pubkey::Pubkey, system_program, sysvar};
use solana_program_test::{processor, ProgramTest};
use solana_sdk::{
    account::Account,
    program_option::COption,
    signature::{Keypair, Signer},
};
use spl_associated_token_account::get_associated_token_address;
use spl_name_service::instruction::NameRegistryInstruction;
use spl_name_service::state::get_seeds_and_key;
pub mod common;
use crate::common::utils::{get_vault, random_string, sign_send_instructions};
use crate::common::{ctx::TestContext, utils::parse_price_feed_fp32};
use common::pyth::PythAccounts;

#[tokio::test]
async fn test_state() {
    let TestContext {
        mint,
        mut ctx,
        mint_authority,
        alice,
        bob,
        vault,
        name_without_rev,
        pyth_accounts,
    } = TestContext::new().await;

    let mut expected_vault_amount = 0;

    alice.create_ata(&mut ctx, &mint).await;
    bob.create_ata(&mut ctx, &mint).await;
    bob.mint_into_ata(&mut ctx, 100_000_000_000, &mint, &mint_authority)
        .await;

    // Test: Create domain without referrer
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
            referrer_account_opt: None,
            vault: &vault,
            rent_sysvar: &sysvar::rent::ID,
            state: &Pubkey::find_program_address(&[&domain_key.to_bytes()], &sns_registrar::ID).0,
        },
        create::Params {
            name: domain,
            space: 100,
            referrer_idx_opt: None,
        },
    );

    sign_send_instructions(&mut ctx, vec![ix], vec![&bob.keypair])
        .await
        .unwrap();

    // Verify state
    let vault_acc = get_vault(&mut ctx, &vault).await;
    let fida_usd_price = parse_price_feed_fp32(pyth_accounts.fida_feed_pull.0, 6, 6);
    let fida_price = (usd_price << 32) / fida_usd_price;
    expected_vault_amount += (fida_price * 95) / 100;
    assert_eq!(vault_acc.amount, expected_vault_amount);

    // Test:
    // - Create .sol without rev
    let reverse_look_up = get_reverse_key(&name_without_rev, None).unwrap();
    let ix = sns_registrar::instruction_auto::create_reverse(
        sns_registrar::ID,
        create_reverse::Accounts {
            naming_service_program: &spl_name_service::ID,
            root_domain: &ROOT_DOMAIN_ACCOUNT,
            central_state: &sns_registrar::central_state::KEY,
            parent_name: None,
            parent_name_owner: None,
            system_program: &system_program::ID,
            fee_payer: &bob.keypair.pubkey(),
            reverse_lookup: &reverse_look_up,
            rent_sysvar: &sysvar::rent::ID,
        },
        create_reverse::Params {
            name: "no_rev".to_string(),
        },
    );
    sign_send_instructions(&mut ctx, vec![ix], vec![&bob.keypair])
        .await
        .unwrap();
    let acc = ctx
        .banks_client
        .get_account(reverse_look_up)
        .await
        .unwrap()
        .unwrap();

    let len = &acc.data[96..96 + 4];
    let len = *bytemuck::from_bytes::<u32>(len);
    let des_name = String::from_utf8(acc.data[96 + 4..96 + 4 + len as usize].to_vec()).unwrap();
    assert_eq!(len, 6);
    assert_eq!(des_name, "no_rev");

    ////////////////////////////////////////////////////////////////////////

    // Test:
    // - Create sub without rev

    // First create sub
    let sub = common::utils::random_string();
    let hashed = sns_registrar::utils::get_hashed_name(&sub);
    let (sub_key, _) = get_seeds_and_key(
        &spl_name_service::id(),
        hashed.clone(),
        None,
        Some(&domain_key),
    );
    let reverse_hashed = sns_registrar::utils::get_hashed_name(&sub_key.to_string());
    let (sub_reverse, _) = get_seeds_and_key(
        &spl_name_service::ID,
        reverse_hashed.clone(),
        Some(&sns_registrar::central_state::KEY),
        Some(&domain_key),
    );
    let space = 0;
    let lamports = 2282880;
    let ix = spl_name_service::instruction::create(
        spl_name_service::ID,
        NameRegistryInstruction::Create {
            hashed_name: hashed,
            lamports,
            space: space as u32,
        },
        sub_key,
        bob.keypair.pubkey(),
        bob.keypair.pubkey(),
        None,
        Some(domain_key),
        Some(bob.keypair.pubkey()),
    )
    .unwrap();
    sign_send_instructions(&mut ctx, vec![ix], vec![&bob.keypair])
        .await
        .unwrap();

    let ix = sns_registrar::instruction_auto::create_reverse(
        sns_registrar::ID,
        create_reverse::Accounts {
            naming_service_program: &spl_name_service::ID,
            root_domain: &ROOT_DOMAIN_ACCOUNT,
            reverse_lookup: &sub_reverse,
            system_program: &system_program::ID,
            central_state: &sns_registrar::central_state::KEY,
            fee_payer: &ctx.payer.pubkey(),
            rent_sysvar: &sysvar::rent::ID,
            parent_name: Some(&domain_key),
            parent_name_owner: Some(&bob.keypair.pubkey()),
        },
        create_reverse::Params { name: sub.clone() },
    );
    sign_send_instructions(&mut ctx, vec![ix], vec![&bob.keypair])
        .await
        .unwrap();
    let acc = ctx
        .banks_client
        .get_account(sub_reverse)
        .await
        .unwrap()
        .unwrap();

    let len = &acc.data[96..96 + 4];
    let len = *bytemuck::from_bytes::<u32>(len);
    let des_name = String::from_utf8(acc.data[96 + 4..96 + 4 + len as usize].to_vec()).unwrap();
    assert_eq!(len, sub.len() as u32);
    assert_eq!(des_name, sub);
}

#[tokio::test]
async fn test_state_create_split_v1_and_v2() {
    let program_id = sns_registrar::ID;
    let mut program_test = ProgramTest::new(
        "sns_registrar",
        program_id,
        processor!(sns_registrar::entrypoint::process_instruction),
    );
    program_test.add_program(
        "spl_name_service",
        spl_name_service::id(),
        processor!(spl_name_service::processor::Processor::process_instruction),
    );

    let token_acc_rent = 2039280;

    // Load Pyth accounts
    let PythAccounts {
        mapping,
        sol_feed_pull,
        sol_price,
        sol_product,
        ..
    } = common::pyth::load_pyth_accounts(true);

    program_test.add_account(mapping.1, mapping.0);
    program_test.add_account(sol_feed_pull.1, sol_feed_pull.0);
    program_test.add_account(sol_price.1, sol_price.0);
    program_test.add_account(sol_product.1, sol_product.0);

    // Set up root account
    let root_domain_data = spl_name_service::state::NameRecordHeader {
        parent_name: Pubkey::default(),
        owner: sns_registrar::central_state::KEY,
        class: Pubkey::default(),
    }
    .try_to_vec()
    .unwrap();

    program_test.add_account(
        ROOT_DOMAIN_ACCOUNT,
        Account {
            lamports: 1_000_000,
            data: root_domain_data,
            owner: spl_name_service::id(),
            ..Account::default()
        },
    );

    // Set up user: Alice

    let alice = Keypair::new();
    let sol_mint = TOKENS_SYM_MINT_DECIMALS.get("SOL").unwrap().0;
    let alice_sol_ata = get_associated_token_address(&alice.pubkey(), &sol_mint);
    let alice_sol_amount = 100 * 1_000_000_000;

    let mut data = vec![0u8; spl_token::state::Account::LEN];
    spl_token::state::Account {
        mint: sol_mint,
        owner: alice.pubkey(),
        amount: alice_sol_amount,
        delegate: COption::None,
        state: spl_token::state::AccountState::Initialized,
        is_native: COption::Some(token_acc_rent),
        delegated_amount: 0,
        close_authority: COption::None,
    }
    .pack_into_slice(&mut data);
    program_test.add_account(
        alice_sol_ata,
        Account {
            lamports: alice_sol_amount + token_acc_rent,
            data,
            owner: spl_token::id(),
            executable: false,
            ..Account::default()
        },
    );

    // Set up vault account
    let vault_ata = get_associated_token_address(&VAULT_OWNER, &sol_mint);
    let mut data = vec![0u8; spl_token::state::Account::LEN];
    spl_token::state::Account {
        mint: sol_mint,
        owner: VAULT_OWNER,
        amount: 0,
        delegate: COption::None,
        state: spl_token::state::AccountState::Initialized,
        is_native: COption::Some(token_acc_rent),
        delegated_amount: 0,
        close_authority: COption::None,
    }
    .pack_into_slice(&mut data);
    program_test.add_account(
        vault_ata,
        Account {
            lamports: token_acc_rent,
            data,
            owner: spl_token::id(),
            executable: false,
            ..Account::default()
        },
    );

    let mut ctx = program_test.start_with_context().await;
    let payer_pubkey = ctx.payer.pubkey();

    ////////////////////////////////////////////////////////////////////////////////////////////////
    // Create split v2
    ////////////////////////////////////////////////////////////////////////////////////////////////

    let name = common::utils::random_string();
    let domain_key = sns_registrar::utils::get_name_key(&name, None).unwrap();
    let reverse_look_up = get_reverse_key(&domain_key, None).unwrap();

    let ix: solana_sdk::instruction::Instruction = create_split_v2(
        program_id,
        create_split_v2::Accounts {
            domain_owner: &alice.pubkey(),
            fee_payer: &payer_pubkey,
            naming_service_program: &spl_name_service::ID,
            root_domain: &ROOT_DOMAIN_ACCOUNT,
            name: &domain_key,
            reverse_lookup: &reverse_look_up,
            system_program: &system_program::id(),
            central_state: &sns_registrar::central_state::KEY,
            buyer: &alice.pubkey(),
            buyer_token_source: &alice_sol_ata,
            pyth_feed_account: &sol_feed_pull.1,
            vault: &vault_ata,
            spl_token_program: &spl_token::ID,
            referrer_account_opt: None,
            rent_sysvar: &sysvar::rent::ID,
            state: &Pubkey::find_program_address(&[&domain_key.to_bytes()], &program_id).0,
        },
        create_split_v2::Params {
            name,
            space: 0,
            referrer_idx_opt: None,
        },
    );
    sign_send_instructions(&mut ctx, vec![ix], vec![&alice])
        .await
        .unwrap();

    ////////////////////////////////////////////////////////////////////////////////////////////////
    // Check state
    // The dumped state of Pyth Push system has the following prices ~ 156.04 SOL (snapshot taken 22/04/24 @1713846180)
    // i.e domain price 0.12817 SOL
    ////////////////////////////////////////////////////////////////////////////////////////////////
    let vault = get_vault(&mut ctx, &vault_ata).await;
    let balances_2 = 128170275;
    assert_eq!(vault.amount, balances_2);
}
