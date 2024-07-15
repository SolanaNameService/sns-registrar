use crate::instruction_auto::ProgramInstruction;
use borsh::BorshDeserialize;
use num_traits::FromPrimitive;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

pub mod create;
pub mod create_reverse;
pub mod create_split_v2;
pub mod create_with_nft;
pub mod delete;
pub struct Processor {}

impl Processor {
    pub fn process_instruction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        msg!("Beginning processing");

        let instruction = FromPrimitive::from_u8(instruction_data[0])
            .ok_or(ProgramError::InvalidInstructionData)?;
        let instruction_data = &instruction_data[1..];
        msg!("Instruction unpacked");

        match instruction {
            ProgramInstruction::Create => {
                msg!("Instruction: Create v3");
                let params = create::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                create::process_create(program_id, accounts, params)?;
            }

            ProgramInstruction::CreateReverse => {
                msg!("Instruction: CreateReverse");
                let params = create_reverse::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                create_reverse::process_create_reverse(program_id, accounts, params)?;
            }
            ProgramInstruction::Delete => {
                msg!("Instruction: Delete");
                let params = delete::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                delete::process_delete(program_id, accounts, params)?
            }
            ProgramInstruction::CreateWithNft => {
                msg!("Instruction: Create with NFT");
                let params = create_with_nft::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                create_with_nft::process_create_with_nft(program_id, accounts, params)?
            }
            ProgramInstruction::CreateSplitV2 => {
                msg!("Instruction: Create with split V2");
                let params = create_split_v2::Params::try_from_slice(instruction_data)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                create_split_v2::process_create(program_id, accounts, params)?
            }
            _ => {
                msg!("Instruction: Deprecated");
                return Err(ProgramError::InvalidInstructionData);
            }
        }

        Ok(())
    }
}
