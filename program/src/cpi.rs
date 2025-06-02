use crate::state::ReverseLookup;
use borsh::BorshSerialize;
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};

use spl_name_service::{instruction::NameRegistryInstruction, state::NameRecordHeader};

pub struct Cpi {}

impl Cpi {
    pub fn create_account<'a>(
        program_id: &Pubkey,
        system_program: &AccountInfo<'a>,
        fee_payer: &AccountInfo<'a>,
        account_to_create: &AccountInfo<'a>,
        signer_seeds: &[&[u8]],
        space: usize,
    ) -> ProgramResult {
        let rent = Rent::get()?.minimum_balance(space);
        let account_lamports = account_to_create.lamports();
        if account_lamports != 0 && account_to_create.data_is_empty() {
            let defund_created_account = system_instruction::transfer(
                account_to_create.key,
                fee_payer.key,
                account_lamports,
            );
            invoke_signed(
                &defund_created_account,
                &[
                    system_program.clone(),
                    fee_payer.clone(),
                    account_to_create.clone(),
                ],
                &[signer_seeds],
            )?;
        }

        let create_state_instruction = system_instruction::create_account(
            fee_payer.key,
            account_to_create.key,
            rent,
            space as u64,
            program_id,
        );

        invoke_signed(
            &create_state_instruction,
            &[
                system_program.clone(),
                fee_payer.clone(),
                account_to_create.clone(),
            ],
            &[signer_seeds],
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_name_account<'a>(
        name_service_program: &AccountInfo<'a>,
        system_program_account: &AccountInfo<'a>,
        name_account: &AccountInfo<'a>,
        fee_payer: &AccountInfo<'a>,
        new_owner_account: &AccountInfo<'a>,
        root_name_account: &AccountInfo<'a>,
        authority: &AccountInfo<'a>,
        hashed_name: Vec<u8>,
        lamports: u64,
        space: u32,
        signer_seeds: &[&[u8]],
    ) -> ProgramResult {
        let create_name_instruction = spl_name_service::instruction::create(
            *name_service_program.key,
            NameRegistryInstruction::Create {
                hashed_name,
                lamports,
                space,
            },
            *name_account.key,
            *fee_payer.key,
            *new_owner_account.key,
            None,
            Some(*root_name_account.key),
            Some(*authority.key),
        )?;

        invoke_signed(
            &create_name_instruction,
            &[
                name_service_program.clone(),
                fee_payer.clone(),
                name_account.clone(),
                new_owner_account.clone(),
                system_program_account.clone(),
                root_name_account.clone(),
                authority.clone(),
            ],
            &[signer_seeds],
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn create_reverse_lookup_account<'a>(
        name_service_program: &AccountInfo<'a>,
        system_program_account: &AccountInfo<'a>,
        reverse_lookup_account: &AccountInfo<'a>,
        fee_payer: &AccountInfo<'a>,
        name: String,
        hashed_reverse_lookup: Vec<u8>,
        authority: &AccountInfo<'a>,
        rent_sysvar_account: &AccountInfo<'a>,
        signer_seeds: &[&[u8]],
        parent_name_opt: Option<&AccountInfo<'a>>,
        parent_name_owner_opt: Option<&AccountInfo<'a>>,
    ) -> ProgramResult {
        let name_bytes = ReverseLookup { name }.try_to_vec().unwrap();
        let rent = Rent::from_account_info(rent_sysvar_account)?;
        let lamports = rent.minimum_balance(name_bytes.len() + NameRecordHeader::LEN);

        let create_name_instruction = spl_name_service::instruction::create(
            *name_service_program.key,
            NameRegistryInstruction::Create {
                hashed_name: hashed_reverse_lookup,
                lamports,
                space: name_bytes.len() as u32,
            },
            *reverse_lookup_account.key,
            *fee_payer.key,
            *authority.key,
            Some(*authority.key),
            parent_name_opt.map(|a| *a.key),
            parent_name_owner_opt.map(|a| *a.key),
        )?;

        let mut accounts_create = vec![
            name_service_program.clone(),
            fee_payer.clone(),
            authority.clone(),
            reverse_lookup_account.clone(),
            system_program_account.clone(),
        ];

        let mut accounts_update = vec![
            name_service_program.clone(),
            reverse_lookup_account.clone(),
            authority.clone(),
        ];

        if let Some(parent_name) = parent_name_opt {
            accounts_create.push(parent_name.clone());
            accounts_create.push(parent_name_owner_opt.unwrap().clone());
            accounts_update.push(parent_name.clone());
        }

        invoke_signed(&create_name_instruction, &accounts_create, &[signer_seeds])?;

        let write_name_instruction = spl_name_service::instruction::update(
            *name_service_program.key,
            0,
            name_bytes,
            *reverse_lookup_account.key,
            *authority.key,
            parent_name_opt.map(|a| *a.key),
        )?;

        invoke_signed(&write_name_instruction, &accounts_update, &[signer_seeds])?;
        Ok(())
    }

    pub fn transfer_name_account<'a>(
        name_service_program: &AccountInfo<'a>,
        old_owner_account: &AccountInfo<'a>,
        name_account: &AccountInfo<'a>,
        new_owner_key: &Pubkey,
        signer_seeds: Option<&[&[u8]]>,
    ) -> ProgramResult {
        let transfer_name_instruction = spl_name_service::instruction::transfer(
            *name_service_program.key,
            *new_owner_key,
            *name_account.key,
            *old_owner_account.key,
            None,
        )?;

        if let Some(seeds) = signer_seeds {
            invoke_signed(
                &transfer_name_instruction,
                &[
                    name_service_program.clone(),
                    old_owner_account.clone(),
                    name_account.clone(),
                ],
                &[seeds],
            )
        } else {
            invoke(
                &transfer_name_instruction,
                &[
                    name_service_program.clone(),
                    old_owner_account.clone(),
                    name_account.clone(),
                ],
            )
        }
    }
}
