//! Error module includes error messages and codes of the program
use anchor_lang::prelude::*;

/// Error messages and codes of the program
#[error_code]
#[derive(PartialEq)]
pub enum ZapError {
    #[msg("Math operation overflow")]
    MathOverflow,

    #[msg("Invalid amm program id")]
    InvalidAmmProgramId,

    #[msg("Missing dlmm remaining accounts info parameter")]
    MissingDlmmRemainingAccountInfo,

    #[msg("Invalid action type")]
    InvalidActionType,

    #[msg("Invalid data length")]
    InvalidDataLength,

    #[msg("Type cast error")]
    TypeCastFailed,
}
