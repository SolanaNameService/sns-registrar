//! Delete a domain and clean up related accounts

use solana_program::program::{invoke, invoke_signed};

use crate::{central_state, utils::get_reverse_key};
use {
    bonfida_utils::{
        checks::{check_account_key, check_account_owner, check_signer},
        BorshSize, InstructionsAccount,
    },
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        msg,
        program_error::ProgramError,
        pubkey::Pubkey,
        system_program,
    },
    spl_name_service::instruction::delete,
};

#[derive(BorshDeserialize, BorshSerialize, BorshSize, Debug)]
/// The required parameters for the `delete` instruction
pub struct Params {}

#[derive(InstructionsAccount)]
/// The required accounts for the `delete` instruction
pub struct Accounts<'a, T> {
    pub name_service_id: &'a T,
    pub system_program: &'a T,
    #[cons(writable)]
    pub domain: &'a T,
    #[cons(writable)]
    pub reverse: &'a T,
    #[cons(writable)]
    pub reselling_state: &'a T,
    #[cons(writable)]
    pub state: &'a T,
    pub central_state: &'a T,
    #[cons(signer)]
    pub owner: &'a T,
    #[cons(writable)]
    pub target: &'a T,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        let accounts = Accounts {
            name_service_id: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
            domain: next_account_info(accounts_iter)?,
            reverse: next_account_info(accounts_iter)?,
            reselling_state: next_account_info(accounts_iter)?,
            state: next_account_info(accounts_iter)?,
            central_state: next_account_info(accounts_iter)?,
            owner: next_account_info(accounts_iter)?,
            target: next_account_info(accounts_iter)?,
        };

        // Check keys
        check_account_key(accounts.name_service_id, &spl_name_service::ID)?;
        check_account_key(accounts.system_program, &system_program::ID)?;
        check_account_key(accounts.central_state, &central_state::KEY)?;

        // Check ownership
        check_account_owner(accounts.domain, &spl_name_service::ID)
            .or_else(|_| check_account_owner(accounts.domain, program_id))?;
        check_account_owner(accounts.reverse, &spl_name_service::ID)
            .or_else(|_| check_account_owner(accounts.reverse, program_id))?;
        check_account_owner(accounts.reselling_state, &system_program::id())
            .or_else(|_| check_account_owner(accounts.reselling_state, program_id))?;
        check_account_owner(accounts.state, &system_program::id())
            .or_else(|_| check_account_owner(accounts.state, program_id))?;

        // Check signer
        check_signer(accounts.owner)?;

        Ok(accounts)
    }
}

pub fn process_delete(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _params: Params,
) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;

    let reverse_key = get_reverse_key(accounts.domain.key, None)?;
    check_account_key(accounts.reverse, &reverse_key)?;

    let seeds = accounts.domain.key.to_bytes();
    let (state, _) = Pubkey::find_program_address(&[&seeds], program_id);
    check_account_key(accounts.state, &state)?;

    let (reselling_state, _) = Pubkey::find_program_address(&[&seeds, &[1u8, 1u8]], program_id);
    check_account_key(accounts.reselling_state, &reselling_state)?;

    msg!("[+] Deleting domain");
    let ix = delete(
        spl_name_service::ID,
        *accounts.domain.key,
        *accounts.owner.key,
        *accounts.target.key,
    )?;
    invoke(
        &ix,
        &[
            accounts.system_program.clone(),
            accounts.owner.clone(),
            accounts.domain.clone(),
            accounts.target.clone(),
        ],
    )?;

    msg!("[+] Deleting reverse");
    let cs_seeds: &[&[u8]] = &[&program_id.to_bytes(), &[crate::central_state::NONCE]];
    let ix = delete(
        spl_name_service::ID,
        *accounts.reverse.key,
        *accounts.central_state.key,
        *accounts.target.key,
    )?;
    invoke_signed(
        &ix,
        &[
            accounts.system_program.clone(),
            accounts.central_state.clone(),
            accounts.reverse.clone(),
            accounts.target.clone(),
        ],
        &[cs_seeds],
    )?;

    if !accounts.reselling_state.data_is_empty() {
        msg!("[+] Deleting reselling state");
        let mut source = accounts.reselling_state.lamports.borrow_mut();
        let mut target = accounts.target.lamports.borrow_mut();

        **target += **source;
        **source = 0;
    }

    if !accounts.state.data_is_empty() {
        msg!("[+] Deleting state");
        let mut source = accounts.state.lamports.borrow_mut();
        let mut target = accounts.target.lamports.borrow_mut();

        **target += **source;
        **source = 0;
    }

    Ok(())
}
