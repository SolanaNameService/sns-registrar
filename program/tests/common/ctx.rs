use super::{nft, pyth::PythAccounts, utils::sign_send_instructions};
use borsh::BorshSerialize;

use sns_registrar::{
    central_state,
    constants::{
        ROOT_DOMAIN_ACCOUNT, TOKENS_SYM_MINT_DECIMALS, VAULT_OWNER, WOLVES_COLLECTION_METADATA,
    },
    utils::get_name_key,
};
use solana_program::{program_option::COption, program_pack::Pack, pubkey::Pubkey};
use solana_program_test::{processor, ProgramTest, ProgramTestContext};
use solana_sdk::{
    account::Account,
    clock::Clock,
    signature::{Keypair, Signer},
    sysvar::SysvarId,
};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};
use spl_token::state::Mint;

pub struct TestContext {
    pub mint_authority: Keypair,
    pub mint: Pubkey,
    pub ctx: ProgramTestContext,
    pub alice: User,
    pub bob: User,
    pub vault: Pubkey,
    pub name_without_rev: Pubkey,
    pub pyth_accounts: PythAccounts,
}

pub const MAX_NAME_LENGTH: usize = 32;

pub const MAX_SYMBOL_LENGTH: usize = 10;

pub const MAX_URI_LENGTH: usize = 200;

pub const MAX_METADATA_LEN: usize = 1 // key
+ 32             // update auth pubkey
+ 32             // mint pubkey
+ MAX_DATA_SIZE
+ 1              // primary sale
+ 1              // mutable
+ 9              // nonce (pretty sure this only needs to be 2)
+ 2              // token standard
+ 34             // collection
+ 18             // uses
+ 10             // collection details
+ 33             // programmable config
+ 75; // Padding

const MAX_CREATOR_LIMIT: usize = 5;
const MAX_CREATOR_LEN: usize = 32 + 1 + 1;

pub const MAX_DATA_SIZE: usize = 4
    + MAX_NAME_LENGTH
    + 4
    + MAX_SYMBOL_LENGTH
    + 4
    + MAX_URI_LENGTH
    + 2
    + 1
    + 4
    + MAX_CREATOR_LIMIT * MAX_CREATOR_LEN;

// The last byte of the account contains the fee flag, indicating
// if the account has fees available for retrieval.
pub const METADATA_FEE_FLAG_INDEX: usize = MAX_METADATA_LEN - 1;
pub const MAX_MASTER_EDITION_LEN: usize = 1 + 9 + 8 + 264;

impl<'a> TestContext {
    pub async fn new() -> Self {
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

        program_test.add_program("mpl_token_metadata", mpl_token_metadata::ID, None);
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

        // Create user
        let bob = User {
            keypair: Keypair::new(),
        };
        let alice = User {
            keypair: Keypair::new(),
        };
        program_test.add_account(
            bob.keypair.pubkey(),
            Account {
                lamports: 1_000_000_000,
                ..Account::default()
            },
        );
        program_test.add_account(
            alice.keypair.pubkey(),
            Account {
                lamports: 1_000_000_000,
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
                lamports: 100_000_000_000,
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
                owner: VAULT_OWNER,
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

        // Create .sol domain without reverse
        let name_without_rev = get_name_key("no_rev", None).unwrap();
        let name_data = spl_name_service::state::NameRecordHeader {
            parent_name: ROOT_DOMAIN_ACCOUNT,
            owner: bob.keypair.pubkey(),
            class: Pubkey::default(),
        }
        .try_to_vec()
        .unwrap();
        program_test.add_account(
            name_without_rev,
            Account {
                lamports: 1_000_000,
                data: name_data,
                owner: spl_name_service::id(),
                ..Account::default()
            },
        );

        // Add mock NFT & collection & Master edition
        let data = super::nft::get_metadata().try_to_vec().unwrap();
        let padding = vec![0u8; MAX_METADATA_LEN - data.len()];
        let data = [data, padding].concat();
        program_test.add_account(
            super::nft::NFT_METADATA_KEY,
            Account {
                owner: mpl_token_metadata::ID,
                lamports: 100_000_000_000,
                data,
                ..Account::default()
            },
        );
        let mut data: Vec<u8> = vec![];
        super::nft::get_master_edition()
            .serialize(&mut data)
            .unwrap();
        let padding = vec![0u8; MAX_MASTER_EDITION_LEN - data.len()];
        let data = [data, padding].concat();
        program_test.add_account(
            super::nft::MASTER_EDITION,
            Account {
                owner: mpl_token_metadata::ID,
                lamports: 100_000_000_000,
                data,
                ..Account::default()
            },
        );
        let mut data: Vec<u8> = vec![];
        super::nft::get_collection().serialize(&mut data).unwrap();
        let padding = vec![0u8; MAX_METADATA_LEN - data.len()];
        let data = [data, padding].concat();
        program_test.add_account(
            super::nft::COLLECTION_KEY,
            Account {
                owner: spl_token::ID,
                lamports: 100_000_000_000,
                data,
                ..Account::default()
            },
        );

        let mut data = [0; spl_token::state::Account::LEN];
        super::nft::get_nft_account(&bob.keypair.pubkey()).pack_into_slice(&mut data);
        let bob_nft_account = bob.get_ata(&nft::NFT_MINT);
        program_test.add_account(
            bob_nft_account,
            Account {
                owner: spl_token::ID,
                lamports: 100_000_000_000,
                data: data.into(),
                ..Account::default()
            },
        );

        let nft_mint_authority = Keypair::new();
        let mut mint_data = vec![0u8; Mint::LEN];
        Mint {
            mint_authority: COption::Some(nft_mint_authority.pubkey()),
            supply: 1_000_000_000,
            decimals: 6,
            is_initialized: true,
            freeze_authority: COption::None,
        }
        .pack_into_slice(&mut mint_data);

        program_test.add_account(
            nft::NFT_MINT,
            Account {
                lamports: 1_000_000,
                data: mint_data,
                owner: spl_token::id(),
                executable: false,
                ..Account::default()
            },
        );

        // Collection metadata
        let data = vec![
            4, 205, 156, 159, 49, 217, 56, 46, 198, 183, 248, 140, 217, 96, 251, 22, 30, 109, 112,
            247, 174, 231, 117, 92, 64, 53, 80, 181, 126, 130, 217, 247, 157, 192, 40, 117, 199,
            250, 129, 156, 113, 33, 232, 189, 29, 203, 48, 234, 35, 31, 138, 180, 111, 8, 21, 120,
            39, 143, 226, 12, 6, 75, 143, 147, 135, 32, 0, 0, 0, 66, 111, 110, 102, 105, 100, 97,
            32, 87, 111, 108, 118, 101, 115, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            10, 0, 0, 0, 66, 111, 110, 102, 105, 100, 97, 0, 0, 0, 200, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 112,
            232, 187, 189, 236, 153, 89, 12, 101, 44, 161, 168, 215, 21, 119, 40, 121, 255, 90, 26,
            7, 141, 216, 97, 212, 133, 170, 44, 140, 208, 177, 33, 1, 100, 0, 1, 1, 254, 1, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        program_test.add_account(
            WOLVES_COLLECTION_METADATA,
            Account {
                lamports: 1_000_000,
                data,
                owner: mpl_token_metadata::ID,
                executable: false,
                ..Account::default()
            },
        );

        // Load Pyth accounts
        let pyth_accounts = super::pyth::load_pyth_accounts(true);

        program_test.add_account(pyth_accounts.mapping.1, pyth_accounts.mapping.0.clone());
        program_test.add_account(pyth_accounts.sol_price.1, pyth_accounts.sol_price.0.clone());
        program_test.add_account(
            pyth_accounts.sol_product.1,
            pyth_accounts.sol_product.0.clone(),
        );
        program_test.add_account(
            pyth_accounts.sol_feed_pull.1,
            pyth_accounts.sol_feed_pull.0.clone(),
        );
        program_test.add_account(
            pyth_accounts.fida_feed_pull.1,
            pyth_accounts.fida_feed_pull.0.clone(),
        );

        let clock: Clock = Clock {
            unix_timestamp: 1682864495,
            ..Default::default()
        };
        program_test.add_sysvar_account(Clock::id(), &clock);

        let ctx = program_test.start_with_context().await;

        Self {
            mint,
            mint_authority,
            ctx,
            alice,
            bob,
            vault: token_vault_key,
            name_without_rev,
            pyth_accounts,
        }
    }
}

pub struct User {
    pub keypair: Keypair,
}

impl User {
    pub async fn create_ata(&self, ctx: &mut ProgramTestContext, mint: &Pubkey) {
        let ix = create_associated_token_account(
            &ctx.payer.pubkey(),
            &self.keypair.pubkey(),
            mint,
            &spl_token::ID,
        );
        sign_send_instructions(ctx, vec![ix], vec![]).await.unwrap();
    }
    pub fn get_ata(&self, mint: &Pubkey) -> Pubkey {
        get_associated_token_address(&self.keypair.pubkey(), mint)
    }
    pub async fn mint_into_ata(
        &self,
        ctx: &mut ProgramTestContext,
        amount: u64,
        mint: &Pubkey,
        mint_authority: &Keypair,
    ) {
        let ix = spl_token::instruction::mint_to(
            &spl_token::ID,
            mint,
            &self.get_ata(mint),
            &mint_authority.pubkey(),
            &[],
            amount,
        )
        .unwrap();
        sign_send_instructions(ctx, vec![ix], vec![mint_authority])
            .await
            .unwrap();
    }
}
