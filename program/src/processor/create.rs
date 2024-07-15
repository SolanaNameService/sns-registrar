//! Create a domain name and buy the ownership of a domain name

use bonfida_utils::{BorshSize, InstructionsAccount};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};

use super::create_split_v2;

#[derive(BorshDeserialize, BorshSerialize, BorshSize, Debug)]
/// The required parameters for the `create` instruction
pub struct Params {
    pub name: String,
    pub space: u32,
    pub referrer_idx_opt: Option<u16>,
}

impl From<Params> for create_split_v2::Params {
    fn from(value: Params) -> Self {
        create_split_v2::Params {
            name: value.name,
            space: value.space,
            referrer_idx_opt: value.referrer_idx_opt,
        }
    }
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
    pub buyer_token_source: &'a T,
    /// The Pyth mapping account
    pub pyth_mapping_acc: &'a T,
    /// The Pyth product account
    pub pyth_product_acc: &'a T,
    /// The Pyth price account
    pub pyth_price_acc: &'a T,
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

impl<'a, T> From<Accounts<'a, T>> for create_split_v2::Accounts<'a, T> {
    fn from(value: Accounts<'a, T>) -> Self {
        Self {
            // Only difference
            fee_payer: value.buyer,
            domain_owner: value.buyer,

            // Exact mapping
            naming_service_program: value.naming_service_program,
            root_domain: value.root_domain,
            name: value.name,
            reverse_lookup: value.reverse_lookup,
            system_program: value.system_program,
            central_state: value.central_state,
            buyer: value.buyer,
            buyer_token_source: value.buyer_token_source,
            pyth_feed_account: value.pyth_mapping_acc,
            vault: value.vault,
            spl_token_program: value.spl_token_program,
            rent_sysvar: value.rent_sysvar,
            state: value.state,
            referrer_account_opt: value.referrer_account_opt,
        }
    }
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
            buyer_token_source: next_account_info(accounts_iter)?,
            pyth_mapping_acc: next_account_info(accounts_iter)?,
            pyth_product_acc: next_account_info(accounts_iter)?,
            pyth_price_acc: next_account_info(accounts_iter)?,
            vault: next_account_info(accounts_iter)?,
            spl_token_program: next_account_info(accounts_iter)?,
            rent_sysvar: next_account_info(accounts_iter)?,
            state: next_account_info(accounts_iter)?,
            referrer_account_opt: next_account_info(accounts_iter).ok(),
        })
    }
}

pub fn process_create(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: Params,
) -> ProgramResult {
    let accounts = Accounts::parse(accounts, program_id)?;
    create_split_v2::create(program_id, accounts.into(), params.into())
}
