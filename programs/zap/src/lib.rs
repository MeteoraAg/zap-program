#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;

#[macro_use]
pub mod macros;

pub mod instructions;
pub use instructions::*;
pub mod const_pda;
pub mod constants;
pub mod utils;
pub use utils::*;
pub mod parameters;
pub use parameters::*;
pub mod error;
pub mod math;
pub use math::*;

declare_id!("GQc29JqD7njZadikaVgzt5A2hkuLUYCDPX9EjYYi8Y3y");

#[program]
pub mod zap {
    use super::*;

    pub fn initialize_token_ledger(ctx: Context<InitializeTokenLedgerCtx>) -> Result<()> {
        instructions::handle_initialize_token_ledger(ctx)
    }

    pub fn zap_out<'c: 'info, 'info>(
        ctx: Context<'_, '_, 'c, 'info, ZapOutCtx<'info>>,
        data: Vec<u8>,
    ) -> Result<()> {
        instructions::handle_zap_out(ctx, data)
    }
}
