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

    #[msg("Invalid zapout parameters")]
    InvalidZapOutParameters,

    #[msg("Type cast error")]
    TypeCastFailed,

    #[msg("Amm program is not supported")]
    AmmIsNotSupported,

    #[msg("Position is not empty")]
    InvalidPosition,

    #[msg("Missing remaining account for transfer hook")]
    MissingRemainingAccountForTransferHook,

    #[msg("Remaining account was passed for transfer hook but there's no hook program")]
    NoTransferHookProgram,

    #[msg("Invalid remaining account slice")]
    InvalidRemainingAccountSlice,

    #[msg("Insufficient remaining accounts")]
    InsufficientRemainingAccounts,

    #[msg("Duplicated remaining account types")]
    DuplicatedRemainingAccountTypes,

    #[msg("Exceeded slippage tolerance")]
    ExceededSlippage,

    #[msg("Invalid base fee mode")]
    InvalidBaseFeeMode,
}
