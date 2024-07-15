//! Create a domain name with a wolf

use std::convert::TryFrom;

use crate::{
    central_state,
    constants::{ROOT_DOMAIN_ACCOUNT, WOLVES_COLLECTION, WOLVES_COLLECTION_METADATA},
    cpi::Cpi,
    error::Error,
    utils::get_hashed_name,
    utils::get_name_key,
};
use bonfida_utils::{
    checks::{check_account_key, check_account_owner, check_signer},
    BorshSize, InstructionsAccount,
};
use borsh::{BorshDeserialize, BorshSerialize};
use mpl_token_metadata::{
    accounts::{MasterEdition, Metadata},
    instructions::{BurnNftCpi, BurnNftCpiAccounts},
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_program, sysvar,
    sysvar::Sysvar,
};
use spl_name_service::state::{get_seeds_and_key, NameRecordHeader};

#[derive(BorshDeserialize, BorshSerialize, BorshSize, Debug)]
/// The required parameters for the `create` instruction
pub struct Params {
    pub name: String,
    pub space: u32,
}

#[derive(InstructionsAccount)]
/// The required accounts for the `create` instruction
pub struct Accounts<'a, T> {
    /// The naming service program ID
    pub naming_service_program: &'a T,
    /// The root domain account       
    pub root_domain: &'a T,
    /// The name account
    #[cons(writable)]
    pub name: &'a T,
    /// The reverse look up account   
    #[cons(writable)]
    pub reverse_lookup: &'a T,
    /// The system program account
    pub system_program: &'a T,
    /// The central state account
    pub central_state: &'a T,
    /// The buyer account     
    #[cons(writable, signer)]
    pub buyer: &'a T,
    /// The buyer token account       
    #[cons(writable)]
    pub nft_source: &'a T,
    /// The NFT metadata account
    #[cons(writable)]
    pub nft_metadata: &'a T,
    /// The NFT mint account
    #[cons(writable)]
    pub nft_mint: &'a T,
    /// The NFT master edition account
    #[cons(writable)]
    pub master_edition: &'a T,
    /// The NFT collection metadata account
    #[cons(writable)]
    pub collection_metadata: &'a T,
    /// The SPL token program
    pub spl_token_program: &'a T,
    /// The rent sysvar account
    pub rent_sysvar: &'a T,
    /// The state auction account
    pub state: &'a T,
    /// Metaplex token metadata program account
    pub mpl_token_metadata: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        _program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            naming_service_program: next_account_info(accounts_iter)?,
            root_domain: next_account_info(accounts_iter)?,
            name: next_account_info(accounts_iter)?,
            reverse_lookup: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
            central_state: next_account_info(accounts_iter)?,
            buyer: next_account_info(accounts_iter)?,
            nft_source: next_account_info(accounts_iter)?,
            nft_metadata: next_account_info(accounts_iter)?,
            nft_mint: next_account_info(accounts_iter)?,
            master_edition: next_account_info(accounts_iter)?,
            collection_metadata: next_account_info(accounts_iter)?,
            spl_token_program: next_account_info(accounts_iter)?,
            rent_sysvar: next_account_info(accounts_iter)?,
            state: next_account_info(accounts_iter)?,
            mpl_token_metadata: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(accounts.naming_service_program, &spl_name_service::ID).unwrap();
        check_account_key(accounts.root_domain, &ROOT_DOMAIN_ACCOUNT).unwrap();
        check_account_key(accounts.system_program, &system_program::ID).unwrap();
        check_account_key(accounts.central_state, &central_state::KEY).unwrap();
        check_account_key(accounts.collection_metadata, &WOLVES_COLLECTION_METADATA).unwrap();
        check_account_key(accounts.spl_token_program, &spl_token::ID).unwrap();
        check_account_key(accounts.rent_sysvar, &sysvar::rent::ID).unwrap();
        check_account_key(accounts.mpl_token_metadata, &mpl_token_metadata::ID).unwrap();

        // Check ownership
        check_account_owner(accounts.name, &system_program::ID)
            .map_err(|_| crate::Error::AlreadyRegistered)?;
        check_account_owner(accounts.state, &system_program::ID).unwrap();
        check_account_owner(accounts.nft_source, &spl_token::ID).unwrap();
        check_account_owner(accounts.nft_metadata, &mpl_token_metadata::ID).unwrap();
        check_account_owner(accounts.nft_mint, &spl_token::ID).unwrap();
        check_account_owner(accounts.master_edition, &mpl_token_metadata::ID).unwrap();
        check_account_owner(accounts.collection_metadata, &mpl_token_metadata::ID).unwrap();

        // Check signer
        check_signer(accounts.buyer).unwrap();

        Ok(accounts)
    }
}

pub fn process_create_with_nft(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;

    if params.name != params.name.trim().to_lowercase() {
        msg!("Domain names must be lower case and have no space");
        return Err(ProgramError::InvalidArgument);
    }
    if params.name.contains('.') {
        return Err(ProgramError::InvalidArgument);
    }
    let name_account_key = get_name_key(&params.name, None).unwrap();

    if &name_account_key != accounts.name.key {
        msg!("Provided wrong name account");
        return Err(ProgramError::InvalidArgument);
    }

    let state_key = Pubkey::find_program_address(&[name_account_key.as_ref()], program_id).0;
    if &state_key != accounts.state.key {
        msg!("An invalid name auctioning state account was provided");
        return Err(ProgramError::InvalidArgument);
    }

    if !accounts.state.data_is_empty() {
        msg!("The name auctioning state account is not empty.");
        return Err(ProgramError::InvalidArgument);
    }

    let hashed_reverse_lookup = get_hashed_name(&name_account_key.to_string());

    let (reverse_lookup_account_key, _) = get_seeds_and_key(
        accounts.naming_service_program.key,
        hashed_reverse_lookup.clone(),
        Some(&central_state::KEY),
        None,
    );

    if &reverse_lookup_account_key != accounts.reverse_lookup.key {
        msg!("Provided wrong reverse lookup account");
        return Err(ProgramError::InvalidArgument);
    }

    let central_state_signer_seeds: &[&[u8]] = &[program_id.as_ref(), &[central_state::NONCE]];

    // Verify NFT
    let (metadata_key, _) = Metadata::find_pda(accounts.nft_mint.key);

    let (master_edition_key, _) = MasterEdition::find_pda(accounts.nft_mint.key);

    check_account_key(accounts.master_edition, &master_edition_key)?;
    check_account_key(accounts.nft_metadata, &metadata_key)?;

    let metadata = Metadata::try_from(accounts.nft_metadata)?;

    if let Some(collection) = metadata.collection {
        if !collection.verified || collection.key != WOLVES_COLLECTION {
            return Err(Error::WrongCollection.into());
        }
    } else {
        return Err(Error::WrongCollection.into());
    }

    BurnNftCpi::new(
        accounts.mpl_token_metadata,
        BurnNftCpiAccounts {
            spl_token_program: accounts.spl_token_program,
            metadata: accounts.nft_metadata,
            owner: accounts.buyer,
            master_edition_account: accounts.master_edition,
            mint: accounts.nft_mint,
            token_account: accounts.nft_source,
            collection_metadata: Some(accounts.collection_metadata),
        },
    )
    .invoke()?;

    // Create domain name
    let rent = Rent::get()?;
    let hashed_name = get_hashed_name(&params.name);
    Cpi::create_name_account(
        accounts.naming_service_program,
        accounts.system_program,
        accounts.name,
        accounts.buyer,
        accounts.buyer,
        accounts.root_domain,
        accounts.central_state,
        hashed_name,
        rent.minimum_balance(NameRecordHeader::LEN + params.space as usize),
        params.space,
        central_state_signer_seeds,
    )?;

    // Reverse look up
    if accounts.reverse_lookup.data_len() == 0 {
        Cpi::create_reverse_lookup_account(
            accounts.naming_service_program,
            accounts.system_program,
            accounts.reverse_lookup,
            accounts.buyer,
            params.name,
            hashed_reverse_lookup,
            accounts.central_state,
            accounts.rent_sysvar,
            central_state_signer_seeds,
            None,
            None,
        )?;
    }
    Ok(())
}
