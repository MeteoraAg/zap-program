use anchor_lang::prelude::*;

// Anchor custom user error codes start at 6000. Adding 1000 to avoid conflict with calling program error codes
#[error_code(offset = 7000)]
#[derive(PartialEq)]
pub enum ProtocolZapError {
    #[msg("Math operation overflow")]
    MathOverflow,

    #[msg("Invalid zapout parameters")]
    InvalidZapOutParameters,

    #[msg("Type cast error")]
    TypeCastFailed,

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
