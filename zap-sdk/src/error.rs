use num_enum::{IntoPrimitive, TryFromPrimitive};
use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum ZapSdkError {
    #[error("Math operation overflow")]
    MathOverflow = 0,

    #[error("Invalid zapout parameters")]
    InvalidZapOutParameters = 1,

    #[error("Type cast error")]
    TypeCastFailed = 2,

    #[error("Missing zap out instruction")]
    MissingZapOutInstruction = 3,

    #[error("Invalid withdraw protocol fee zap accounts")]
    InvalidWithdrawProtocolFeeZapAccounts = 4,

    #[error("SOL,USDC protocol fee cannot be withdrawn via zap")]
    MintRestrictedFromZap = 5,

    #[error("CPI disabled")]
    CpiDisabled = 6,

    #[error("Invalid zap accounts")]
    InvalidZapAccounts = 7,
}

impl From<ZapSdkError> for ProgramError {
    fn from(e: ZapSdkError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
