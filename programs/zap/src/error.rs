//! Error module includes error messages and codes of the program
use anchor_lang::prelude::*;

/// Error messages and codes of the program
#[error_code]
#[derive(PartialEq)]
pub enum ZapError {
    #[msg("Amount is zero")]
    AmountIsZero,

    #[msg("Unsupported program")]
    UnsupportedAmmProgram,

    #[msg("Missing damm v2 remaining accounts")]
    MissingDammV2RemainingAccount,
    #[msg("Invalid accounts for damm v2")]
    InvalidDammV2Accounts,
}
