#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;

pub mod constants;
pub mod entrypoint;
pub mod error;
pub mod instructions;
pub mod math;
pub mod tests;

pub use instructions::*;
pub use math::*;

declare_id!("zapvX9M3uf5pvy4wRPAbQgdQsM1xmuiFnkfHKPvwMiz");

#[program]
pub mod zap {
    use super::*;

    pub fn zap_out<'c: 'info, 'info>(
        ctx: Context<'_, '_, 'c, 'info, ZapOutCtx<'info>>,
        params: ZapOutParameters,
    ) -> Result<()> {
        instructions::handle_zap_out(ctx, &params)
    }

    pub fn zap_out_2(_ctx: Context<ZapOut2Ctx>, _params: ZapOut2Parameters) -> Result<()> {
        Ok(())
    }
}
