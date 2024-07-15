//! Create a reverse lookup registry for a name registry

use crate::{
    central_state,
    cpi::Cpi,
    utils::{get_hashed_name, get_name_key},
};

use crate::constants::ROOT_DOMAIN_ACCOUNT;
use bonfida_utils::{
    checks::{check_account_key, check_account_owner, check_signer},
    BorshSize, InstructionsAccount,
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    system_program, sysvar,
};
use spl_name_service::state::{get_seeds_and_key, NameRecordHeader};

#[derive(BorshDeserialize, BorshSerialize, BorshSize, Debug)]
/// The required parameters for the `create_reverse` instruction
pub struct Params {
    pub name: String,
}

#[derive(InstructionsAccount)]
/// The required accounts for the `create_reverse` instruction
pub struct Accounts<'a, T> {
    /// The name service program account
    pub naming_service_program: &'a T,

    /// The root domain account
    pub root_domain: &'a T,

    /// The reverse lookup account
    #[cons(writable)]
    pub reverse_lookup: &'a T,

    /// The system program account
    pub system_program: &'a T,

    /// The central state account
    pub central_state: &'a T,

    /// The fee payer account
    #[cons(signer, writable)]
    pub fee_payer: &'a T,

    pub rent_sysvar: &'a T,

    /// The optional parent name account for subdomains
    #[cons(writable)]
    pub parent_name: Option<&'a T>,

    /// The optional parent name owner
    #[cons(signer, writable)]
    pub parent_name_owner: Option<&'a T>,
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
            reverse_lookup: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
            central_state: next_account_info(accounts_iter)?,
            fee_payer: next_account_info(accounts_iter)?,
            rent_sysvar: next_account_info(accounts_iter)?,
            parent_name: next_account_info(accounts_iter).ok(),
            parent_name_owner: next_account_info(accounts_iter).ok(),
        };

        // Check keys
        check_account_key(accounts.naming_service_program, &spl_name_service::id()).unwrap();
        check_account_key(accounts.root_domain, &ROOT_DOMAIN_ACCOUNT).unwrap();
        check_account_key(accounts.system_program, &system_program::id()).unwrap();
        check_account_key(accounts.central_state, &central_state::KEY).unwrap();
        check_account_key(accounts.rent_sysvar, &sysvar::rent::id()).unwrap();

        // Check owners
        check_account_owner(accounts.reverse_lookup, &system_program::ID).unwrap();

        // Check signer
        check_signer(accounts.fee_payer).unwrap();

        Ok(accounts)
    }
}

pub fn process_create_reverse(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;

    let mut parent = None;
    match (accounts.parent_name, accounts.parent_name_owner) {
        (Some(_), None) | (None, Some(_)) => return Err(ProgramError::InvalidArgument),
        (Some(parent_name), Some(parent_name_owner)) => {
            check_account_owner(parent_name, &spl_name_service::ID).unwrap();
            check_signer(parent_name_owner).unwrap();
            let parent_hd =
                NameRecordHeader::unpack_from_slice(&parent_name.data.borrow()).unwrap();
            if parent_hd.parent_name != ROOT_DOMAIN_ACCOUNT {
                msg!("Invalid parent name");
                return Err(ProgramError::InvalidArgument);
            }
            parent = Some(parent_name.key);
        }
        _ => (),
    }

    let name_account_key = get_name_key(&params.name, parent).unwrap();

    let hashed_reverse_lookup = get_hashed_name(&name_account_key.to_string());

    let (reverse_lookup_account_key, _) = get_seeds_and_key(
        accounts.naming_service_program.key,
        hashed_reverse_lookup.clone(),
        Some(accounts.central_state.key),
        parent,
    );

    if &reverse_lookup_account_key != accounts.reverse_lookup.key {
        msg!("Provided wrong reverse lookup account");
        return Err(ProgramError::InvalidArgument);
    }

    let central_state_signer_seeds: &[&[u8]] = &[&program_id.to_bytes(), &[central_state::NONCE]];

    if accounts.reverse_lookup.data_len() == 0 {
        Cpi::create_reverse_lookup_account(
            accounts.naming_service_program,
            accounts.system_program,
            accounts.reverse_lookup,
            accounts.fee_payer,
            params.name,
            hashed_reverse_lookup,
            accounts.central_state,
            accounts.rent_sysvar,
            central_state_signer_seeds,
            accounts.parent_name,
            accounts.parent_name_owner,
        )?;
    } else {
        msg!("Reverse lookup already exists. No-op");
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    Ok(())
}
