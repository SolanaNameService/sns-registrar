//! Create a domain name and buy the ownership of a domain name

use bonfida_utils::{
    checks::{check_account_key, check_account_owner, check_signer},
    BorshSize, InstructionsAccount,
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_program, sysvar,
    sysvar::Sysvar,
};
use spl_name_service::state::{get_seeds_and_key, NameRecordHeader};
use spl_token::instruction::transfer;

use crate::{
    central_state,
    constants::{FIDA_MINT, REFERRER_FEES_PCT, REFERRER_WHITELIST, ROOT_DOMAIN_ACCOUNT},
    cpi::Cpi,
    utils::{
        check_vault_token_account_owner, get_domain_price_checked, get_hashed_name, get_name_key,
        get_special_discount_and_fee,
    },
    Error,
};

#[derive(BorshDeserialize, BorshSerialize, BorshSize, Debug)]
/// The required parameters for the `create` instruction
pub struct Params {
    pub name: String,
    pub space: u32,
    pub referrer_idx_opt: Option<u16>,
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
    /// The registered domain owner     
    pub domain_owner: &'a T,
    /// The solana fee payer account     
    #[cons(writable, signer)]
    pub fee_payer: &'a T,
    /// The buyer token account       
    #[cons(writable)]
    pub buyer_token_source: &'a T,
    /// The Pyth feed account
    pub pyth_feed_account: &'a T,
    /// The vault account     
    #[cons(writable)]
    pub vault: &'a T,
    /// The SPL token program
    pub spl_token_program: &'a T,
    /// The rent sysvar account
    pub rent_sysvar: &'a T,
    /// The state auction account
    pub state: &'a T,
    /// The *optional* referrer token account to receive a portion of fees.
    /// The token account owner has to be whitelisted.
    #[cons(writable)]
    pub referrer_account_opt: Option<&'a T>,
}

impl<'a, 'b: 'a> Accounts<'a, AccountInfo<'b>> {
    pub fn parse(
        accounts: &'a [AccountInfo<'b>],
        _program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        let accounts_iter = &mut accounts.iter();
        Ok(Accounts {
            naming_service_program: next_account_info(accounts_iter)?,
            root_domain: next_account_info(accounts_iter)?,
            name: next_account_info(accounts_iter)?,
            reverse_lookup: next_account_info(accounts_iter)?,
            system_program: next_account_info(accounts_iter)?,
            central_state: next_account_info(accounts_iter)?,
            buyer: next_account_info(accounts_iter)?,
            domain_owner: next_account_info(accounts_iter)?,
            fee_payer: next_account_info(accounts_iter)?,
            buyer_token_source: next_account_info(accounts_iter)?,
            pyth_feed_account: next_account_info(accounts_iter)?,
            vault: next_account_info(accounts_iter)?,
            spl_token_program: next_account_info(accounts_iter)?,
            rent_sysvar: next_account_info(accounts_iter)?,
            state: next_account_info(accounts_iter)?,
            referrer_account_opt: next_account_info(accounts_iter).ok(),
        })
    }

    pub fn check(&self) -> Result<(), ProgramError> {
        // Check keys
        check_account_key(self.naming_service_program, &spl_name_service::ID).unwrap();
        check_account_key(self.root_domain, &ROOT_DOMAIN_ACCOUNT).unwrap();
        check_account_key(self.system_program, &system_program::ID).unwrap();
        check_account_key(self.central_state, &central_state::KEY).unwrap();
        check_account_key(self.spl_token_program, &spl_token::ID).unwrap();
        check_account_key(self.rent_sysvar, &sysvar::rent::ID).unwrap();

        // Check ownership
        check_account_owner(self.name, &system_program::ID)
            .map_err(|_| crate::Error::AlreadyRegistered)?;
        check_account_owner(self.vault, &spl_token::ID).unwrap();
        check_account_owner(self.state, &system_program::ID).unwrap();

        // Check signer
        check_signer(self.buyer).unwrap();
        check_signer(self.fee_payer).unwrap();

        Ok(())
    }
}

pub fn process_create(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;
    create(program_id, accounts, params)
}

pub fn create<'a, 'b: 'a>(
    program_id: &Pubkey,
    accounts: Accounts<'a, AccountInfo<'b>>,
    params: Params,
) -> ProgramResult {
    accounts.check()?;
    check_vault_token_account_owner(accounts.vault).unwrap();

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

    let (state_key, _) = Pubkey::find_program_address(&[&name_account_key.to_bytes()], program_id);
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

    let central_state_signer_seeds: &[&[u8]] = &[&program_id.to_bytes(), &[central_state::NONCE]];

    let mut domain_token_price = get_domain_price_checked(&params.name, &accounts)?;

    // 5% discount if paid in FIDA
    let token_acc = spl_token::state::Account::unpack(&accounts.buyer_token_source.data.borrow())?;
    if token_acc.mint == FIDA_MINT {
        domain_token_price = domain_token_price.checked_mul(95).ok_or(Error::Overflow)? / 100;
    }

    //Check referrer token account owner and transfer fees
    let referrer_fees = if let Some(referrer_account) = accounts.referrer_account_opt {
        check_account_owner(referrer_account, &spl_token::ID)?;
        let mut referrer_fee_pct = REFERRER_FEES_PCT;
        let referrer_token_acc =
            spl_token::state::Account::unpack(&referrer_account.data.borrow()).unwrap();

        #[cfg(not(feature = "no-referrer-check"))]
        if *REFERRER_WHITELIST
            .get(params.referrer_idx_opt.unwrap() as usize)
            .unwrap()
            != referrer_token_acc.owner
        {
            msg!("Referrer token account owner is not whitelisted.");
            return Err(ProgramError::IllegalOwner);
        }

        let (discount, special_fee) = get_special_discount_and_fee(&referrer_token_acc.owner);
        if let Some(discount) = discount {
            domain_token_price = 100u64
                .checked_sub(discount as u64)
                .ok_or(Error::Overflow)?
                .checked_mul(domain_token_price)
                .ok_or(Error::Overflow)?
                / 100;
        }
        if let Some(special_fee) = special_fee {
            referrer_fee_pct = special_fee as u64
        }

        let referrer_fees_amount = domain_token_price.checked_mul(referrer_fee_pct).unwrap() / 100;
        let transfer_ix = transfer(
            &spl_token::ID,
            accounts.buyer_token_source.key,
            referrer_account.key,
            accounts.buyer.key,
            &[],
            referrer_fees_amount,
        )?;

        invoke(
            &transfer_ix,
            &[
                accounts.spl_token_program.clone(),
                accounts.buyer_token_source.clone(),
                referrer_account.clone(),
                accounts.buyer.clone(),
            ],
        )?;

        referrer_fees_amount
    } else {
        0
    };

    // Transfer tokens to vault
    let transfer_ix = transfer(
        &spl_token::ID,
        accounts.buyer_token_source.key,
        accounts.vault.key,
        accounts.buyer.key,
        &[],
        domain_token_price.checked_sub(referrer_fees).unwrap(),
    )?;

    invoke(
        &transfer_ix,
        &[
            accounts.spl_token_program.clone(),
            accounts.buyer_token_source.clone(),
            accounts.vault.clone(),
            accounts.buyer.clone(),
        ],
    )?;

    // Create domain name
    let rent = Rent::get()?;
    let hashed_name = get_hashed_name(&params.name);
    Cpi::create_name_account(
        accounts.naming_service_program,
        accounts.system_program,
        accounts.name,
        accounts.fee_payer,
        accounts.domain_owner,
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
            accounts.fee_payer,
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
