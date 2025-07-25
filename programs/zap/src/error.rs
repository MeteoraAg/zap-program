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

    #[msg("Invalid action type")]
    InvalidActionType,

    #[msg("Type cast error")]
    TypeCastFailed,

    #[msg("Amount is zero")]
    AmountIsZero,

    #[msg("Cannot converge to optimal value")]
    CannotConvergeToOptimalValue,
}
