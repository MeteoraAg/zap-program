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

    #[msg("Remaining amount is over threshold")]
    RemainingAmountIsOverThreshold,

    #[msg("Liquidity delta is zero")]
    LiquidityDeltaIsZero,
}
