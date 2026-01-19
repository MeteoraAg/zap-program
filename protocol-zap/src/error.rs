use anchor_lang::prelude::*;

// Anchor custom user error codes start at 6000. Adding 1000 to avoid conflict with calling program error codes
#[error_code(offset = 7000)]
#[derive(PartialEq)]
pub enum ProtocolZapError {
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

    #[msg("Exceeded slippage tolerance")]
    ExceededSlippage,

    #[msg("Invalid dlmm zap in parameters")]
    InvalidDlmmZapInParameters,

    #[msg("Unsupported fee mode")]
    UnsupportedFeeMode,

    #[msg("Missing zap out instruction")]
    MissingZapOutInstruction,

    #[msg("Invalid withdraw protocol fee zap accounts")]
    InvalidWithdrawProtocolFeeZapAccounts,

    #[msg("SOL,USDC protocol fee cannot be withdrawn via zap")]
    MintRestrictedFromZap,

    #[msg("CPI disabled")]
    CpiDisabled,

    #[msg("Invalid zap accounts")]
    InvalidZapAccounts,
}
