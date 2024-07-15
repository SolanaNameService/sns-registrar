use borsh::BorshSerialize;
use common::ctx::TestContext;
use common::pyth::PythAccounts;
use common::utils::{create_and_get_associated_token_address, random_string};
use mpl_token_metadata::accounts::MasterEdition;
use sns_registrar::instruction_auto::create_with_nft;
use sns_registrar::processor::create_with_nft;
use sns_registrar::{
    central_state,
    constants::{
        ROOT_DOMAIN_ACCOUNT, TOKENS_SYM_MINT_DECIMALS, VAULT_OWNER, WOLVES_COLLECTION_METADATA,
    },
    instruction_auto::{create, create_reverse, delete},
    processor::{create, create_reverse, delete},
};
use solana_program::{
    hash::hashv,
    program_option::COption,
    program_pack::Pack,
    pubkey::Pubkey,
    system_program,
    sysvar::{self},
};
use solana_program_test::{processor, ProgramTest};
use solana_sdk::{
    account::Account,
    signature::{Keypair, Signer},
};
use spl_name_service::{
    instruction::NameRegistryInstruction,
    state::{get_seeds_and_key, HASH_PREFIX},
};
use spl_token::{instruction::mint_to, state::Mint};

pub mod common;
use crate::common::utils::sign_send_instructions;

#[tokio::test]
async fn test_functional_0() {
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

    let (derived_central_state_key, _) =
        Pubkey::find_program_address(&[&program_id.to_bytes()], &program_id);
    program_test.add_account(
        derived_central_state_key,
        Account {
            lamports: 1_000_000,
            data: vec![central_state::NONCE],
            owner: program_id,
            executable: false,
            ..Account::default()
        },
    );

    let mint_authority = Keypair::new();
    let mint = TOKENS_SYM_MINT_DECIMALS.get("FIDA").unwrap().0;
    let mut mint_data = vec![0u8; Mint::LEN];
    Mint {
        mint_authority: COption::Some(mint_authority.pubkey()),
        supply: 1_000_000_000,
        decimals: 6,
        is_initialized: true,
        freeze_authority: COption::None,
    }
    .pack_into_slice(&mut mint_data);

    program_test.add_account(
        mint,
        Account {
            lamports: 1_000_000,
            data: mint_data,
            owner: spl_token::id(),
            executable: false,
            ..Account::default()
        },
    );

    let mut vault_data = vec![0u8; spl_token::state::Account::LEN];
    spl_token::state::Account {
        mint,
        owner: VAULT_OWNER,
        amount: 0,
        delegate: COption::None,
        state: spl_token::state::AccountState::Initialized,
        is_native: COption::None,
        delegated_amount: 0,
        close_authority: COption::None,
    }
    .pack_into_slice(&mut vault_data);

    let token_vault_key = Pubkey::new_unique();
    program_test.add_account(
        token_vault_key,
        Account {
            lamports: 1_000_000,
            data: vault_data,
            owner: spl_token::id(),
            executable: false,
            ..Account::default()
        },
    );
    program_test.add_account(
        VAULT_OWNER,
        Account {
            lamports: 1_000_000,
            data: vec![],
            executable: false,
            ..Account::default()
        },
    );

    let root_domain_data = spl_name_service::state::NameRecordHeader {
        parent_name: Pubkey::default(),
        owner: derived_central_state_key,
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

    // Load Pyth accounts
    let PythAccounts {
        mapping,
        sol_feed_pull,
        sol_price,
        sol_product,
        fida_feed_pull,
    } = common::pyth::load_pyth_accounts(true);

    program_test.add_account(mapping.1, mapping.0);
    program_test.add_account(sol_feed_pull.1, sol_feed_pull.0);
    program_test.add_account(sol_price.1, sol_price.0);
    program_test.add_account(sol_product.1, sol_product.0);
    program_test.add_account(fida_feed_pull.1, fida_feed_pull.0);

    let mut ctx = program_test.start_with_context().await;
    let payer_pubkey = ctx.payer.pubkey();

    let name = "megosiani";
    let hashed_name: Vec<u8> = hashv(&[(HASH_PREFIX.to_owned() + name).as_bytes()])
        .as_ref()
        .to_vec();
    let (name_account_key, _) = get_seeds_and_key(
        &spl_name_service::id(),
        hashed_name.clone(),
        None,
        Some(&ROOT_DOMAIN_ACCOUNT),
    );
    let hashed_reverse_lookup =
        hashv(&[(HASH_PREFIX.to_owned() + &name_account_key.to_string()).as_bytes()])
            .as_ref()
            .to_vec();

    let (reverse_lookup_account_key, _) = get_seeds_and_key(
        &spl_name_service::id(),
        hashed_reverse_lookup.clone(),
        Some(&derived_central_state_key),
        None,
    );

    let create_reverse_naming_auction_instruction = create_reverse(
        program_id,
        create_reverse::Accounts {
            naming_service_program: &spl_name_service::ID,
            root_domain: &ROOT_DOMAIN_ACCOUNT,
            reverse_lookup: &reverse_lookup_account_key,
            system_program: &system_program::ID,
            central_state: &derived_central_state_key,
            fee_payer: &payer_pubkey,
            parent_name: None,
            rent_sysvar: &sysvar::rent::ID,
            parent_name_owner: None,
        },
        create_reverse::Params {
            name: name.to_owned(),
        },
    );

    sign_send_instructions(
        &mut ctx,
        vec![create_reverse_naming_auction_instruction],
        vec![],
    )
    .await
    .unwrap();

    // Create a user token acc
    let buyer_token_source =
        create_and_get_associated_token_address(&mut ctx, &payer_pubkey, &mint)
            .await
            .unwrap();
    let mint_to_instr = mint_to(
        &spl_token::ID,
        &mint,
        &buyer_token_source,
        &mint_authority.pubkey(),
        &[],
        1_000_000_000,
    )
    .unwrap();
    sign_send_instructions(&mut ctx, vec![mint_to_instr], vec![&mint_authority])
        .await
        .unwrap();

    let create_name_instruction = create(
        sns_registrar::id(),
        create::Accounts {
            naming_service_program: &spl_name_service::ID,
            root_domain: &ROOT_DOMAIN_ACCOUNT,
            name: &name_account_key,
            reverse_lookup: &reverse_lookup_account_key,
            system_program: &system_program::id(),
            central_state: &derived_central_state_key,
            buyer: &payer_pubkey,
            buyer_token_source: &buyer_token_source,
            pyth_mapping_acc: &fida_feed_pull.1,
            //Pyth account derivation is tested in bo&nfida pyth utils //TODO
            pyth_product_acc: &Pubkey::default(),
            pyth_price_acc: &Pubkey::default(),
            vault: &token_vault_key,
            spl_token_program: &spl_token::ID,
            referrer_account_opt: None,
            rent_sysvar: &sysvar::rent::ID,
            state: &Pubkey::find_program_address(&[&name_account_key.to_bytes()], &program_id).0,
        },
        create::Params {
            name: name.to_string(),
            space: 10_000,
            referrer_idx_opt: Some(0),
        },
    );
    sign_send_instructions(&mut ctx, vec![create_name_instruction], vec![])
        .await
        .unwrap();

    // Create an auction via the deprecated instructions
    let (derived_state_key, _) =
        Pubkey::find_program_address(&[&name_account_key.to_bytes()], &program_id);
    let (derived_reselling_state_key, _) =
        Pubkey::find_program_address(&[&name_account_key.to_bytes(), &[1u8, 1u8]], &program_id);

    // Create reverse for subs
    let sub = common::utils::random_string();
    let hashed = sns_registrar::utils::get_hashed_name(&sub);
    let (sub_key, _) = get_seeds_and_key(
        &spl_name_service::id(),
        hashed.clone(),
        None,
        Some(&name_account_key),
    );
    let reverse_hashed = sns_registrar::utils::get_hashed_name(&sub_key.to_string());
    let (sub_reverse, _) = get_seeds_and_key(
        &spl_name_service::ID,
        reverse_hashed.clone(),
        Some(&sns_registrar::central_state::KEY),
        Some(&name_account_key),
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
        ctx.payer.pubkey(),
        ctx.payer.pubkey(),
        None,
        Some(name_account_key),
        Some(ctx.payer.pubkey()),
    )
    .unwrap();
    sign_send_instructions(&mut ctx, vec![ix], vec![])
        .await
        .unwrap();

    let ix = create_reverse(
        program_id,
        create_reverse::Accounts {
            naming_service_program: &spl_name_service::ID,
            root_domain: &ROOT_DOMAIN_ACCOUNT,
            reverse_lookup: &sub_reverse,
            system_program: &system_program::ID,
            central_state: &sns_registrar::central_state::KEY,
            fee_payer: &ctx.payer.pubkey(),
            rent_sysvar: &sysvar::rent::ID,
            parent_name: Some(&name_account_key),
            parent_name_owner: Some(&ctx.payer.pubkey()),
        },
        create_reverse::Params { name: sub },
    );
    sign_send_instructions(&mut ctx, vec![ix], vec![])
        .await
        .unwrap();

    // Delete
    let ix = delete(
        program_id,
        delete::Accounts {
            name_service_id: &spl_name_service::ID,
            system_program: &system_program::ID,
            domain: &name_account_key,
            reverse: &reverse_lookup_account_key,
            reselling_state: &derived_reselling_state_key,
            state: &derived_state_key,
            owner: &ctx.payer.pubkey(),
            central_state: &sns_registrar::central_state::KEY,
            target: &ctx.payer.pubkey(),
        },
        delete::Params {},
    );
    sign_send_instructions(&mut ctx, vec![ix], vec![])
        .await
        .unwrap();
}

#[tokio::test]
async fn test_functional_2() {
    let TestContext { mut ctx, bob, .. } = TestContext::new().await;

    let domain = random_string();
    let domain_key = sns_registrar::utils::get_name_key(&domain, None).unwrap();
    let reverse_key = sns_registrar::utils::get_reverse_key(&domain_key, None).unwrap();

    let source = ctx
        .banks_client
        .get_account(bob.get_ata(&common::nft::NFT_MINT))
        .await
        .unwrap()
        .unwrap();

    let des = spl_token::state::Account::unpack(&source.data).unwrap();
    assert_eq!(des.amount, 1);

    let ix = create_with_nft(
        sns_registrar::ID,
        create_with_nft::Accounts {
            naming_service_program: &spl_name_service::ID,
            root_domain: &ROOT_DOMAIN_ACCOUNT,
            rent_sysvar: &sysvar::rent::ID,
            name: &domain_key,
            reverse_lookup: &reverse_key,
            system_program: &system_program::ID,
            central_state: &sns_registrar::central_state::KEY,
            buyer: &bob.keypair.pubkey(),
            nft_source: &bob.get_ata(&common::nft::NFT_MINT),
            nft_metadata: &common::nft::NFT_METADATA_KEY,
            collection_metadata: &WOLVES_COLLECTION_METADATA,
            spl_token_program: &spl_token::ID,
            state: &Pubkey::find_program_address(&[&domain_key.to_bytes()], &sns_registrar::ID).0,
            nft_mint: &common::nft::NFT_MINT,
            master_edition: &MasterEdition::find_pda(&common::nft::NFT_MINT).0,
            mpl_token_metadata: &mpl_token_metadata::ID,
        },
        create_with_nft::Params {
            name: domain,
            space: 100,
        },
    );
    sign_send_instructions(&mut ctx, vec![ix], vec![&bob.keypair])
        .await
        .unwrap();

    let source = ctx
        .banks_client
        .get_account(bob.get_ata(&common::nft::NFT_MINT))
        .await
        .unwrap();

    assert!(source.is_none())
}

#[tokio::test]
async fn test_create_split_v2() {}
