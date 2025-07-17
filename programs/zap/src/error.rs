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

    #[msg("Missing dlmm remaining accounts info parameter")]
    MissingDlmmRemainingAccountInfo,
}
