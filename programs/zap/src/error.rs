//! Error module includes error messages and codes of the program
use anchor_lang::prelude::*;

/// Error messages and codes of the program
#[error_code]
#[derive(PartialEq)]
pub enum ZapError {
    #[msg("Math operation overflow")]
    MathOverflow,

    #[msg("Invalid offset")]
    InvalidOffset,

    #[msg("Math operation overflow")]
    InvalidZapOutParameters,

    #[msg("Missing remaining account for transfer hook")]
    MissingRemainingAccountForTransferHook,

    #[msg("Missing memo program")]
    MissingMemoProgram,

    #[msg("Type cast error")]
    TypeCastFailed,

    #[msg("Amm program is not supported")]
    AmmIsNotSupported,
}
