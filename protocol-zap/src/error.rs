use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ProtocolZapError {
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

impl ProtocolZapError {
    pub fn name(&self) -> String {
        match self {
            Self::MathOverflow => "MathOverflow",
            Self::InvalidZapOutParameters => "InvalidZapOutParameters",
            Self::TypeCastFailed => "TypeCastFailed",
            Self::MissingZapOutInstruction => "MissingZapOutInstruction",
            Self::InvalidWithdrawProtocolFeeZapAccounts => "InvalidWithdrawProtocolFeeZapAccounts",
            Self::MintRestrictedFromZap => "MintRestrictedFromZap",
            Self::CpiDisabled => "CpiDisabled",
            Self::InvalidZapAccounts => "InvalidZapAccounts",
        }
        .to_string()
    }
}

impl From<ProtocolZapError> for u32 {
    fn from(e: ProtocolZapError) -> Self {
        e as u32
    }
}

impl From<ProtocolZapError> for ProgramError {
    fn from(e: ProtocolZapError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
