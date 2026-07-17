use num_traits::FromPrimitive;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{error::Error, processor::Processor};

#[cfg(not(feature = "no-entrypoint"))]
use solana_program::entrypoint;
#[cfg(not(feature = "no-entrypoint"))]
entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    msg!("Entrypoint");
    if let Err(error) = Processor::process_instruction(program_id, accounts, instruction_data) {
        match &error {
            ProgramError::Custom(error_id) => {
                if let Some(local_error) = crate::Error::from_u32(*error_id) {
                    msg!("Error: {}", local_error);
                } else {
                    msg!("External custom program error: {:#x}", error_id);
                }
            }
            error => msg!("Error: {}", error),
        }
        return Err(error);
    }
    Ok(())
}

// impl PrintProgramError for Error {
//     fn print<E>(&self)
//     where
//         E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
//     {
//         match self {
//             Error::Overflow => {
//                 msg!("Error: Numerical overflow")
//             }
//             Error::WrongCollection => msg!("Error: Wrong collection"),
//             Error::AlreadyRegistered => {
//                 msg!("Error: The domain name is already registered")
//             }
//             Error::DeprecatedInstruction => {
//                 msg!("Error: The instruction is deprecated")
//             }
//         }
//     }
// }
