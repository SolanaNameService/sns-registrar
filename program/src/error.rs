use num_derive::FromPrimitive;
use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum Error {
    #[error("Overflow")]
    Overflow,
    #[error("Wrong collection")]
    WrongCollection,
    #[error("The domain name is already registered")]
    AlreadyRegistered,
    #[error("The instruction is deprecated")]
    DeprecatedInstruction,
    #[error("The registrar is temporarily closed")]
    ClosedRegistrar,
}

impl From<crate::Error> for ProgramError {
    fn from(e: Error) -> Self {
        ProgramError::Custom(e as u32)
    }
}
