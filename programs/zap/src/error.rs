//! Error module includes error messages and codes of the program
use anchor_lang::prelude::{thiserror::Error, *};
use pinocchio::program_error;
use pinocchio_log::log;

/// Error messages and codes of the program
#[error_code]
#[derive(PartialEq)]
pub enum ZapError {
    #[msg("Math operation overflow")]
    MathOverflow,

    #[msg("Invalid offset")]
    InvalidOffset,

    #[msg("Invalid zapout parameters")]
    InvalidZapOutParameters,

    #[msg("Type cast error")]
    TypeCastFailed,

    #[msg("Amm program is not supported")]
    AmmIsNotSupported,
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum PinoError {
    #[error("Invalid instruction")]
    InvalidInstruction,

    #[error("Invalid account")]
    InvalidAccount,

    #[error("Invalid offset")]
    InvalidOffset,

    #[error("Invalid zapout parameters")]
    InvalidZapOutParameters,

    #[error("Type cast error")]
    TypeCastFailed,

    #[error("Amm program is not supported")]
    AmmIsNotSupported,
}

impl From<PinoError> for program_error::ProgramError {
    fn from(e: PinoError) -> Self {
        let msg: &str = &e.to_string();
        log!("Error: {}", msg);
        program_error::ProgramError::Custom(e as u32)
    }
}
