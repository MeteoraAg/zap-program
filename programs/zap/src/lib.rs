#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;

pub mod instructions;
pub use instructions::*;
pub mod constants;
pub mod error;
pub mod math;
pub use math::*;

pub mod tests;

mod entrypoint;
pub use entrypoint::entrypoint;

declare_id!("zapvX9M3uf5pvy4wRPAbQgdQsM1xmuiFnkfHKPvwMiz");

#[program]
pub mod zap {
    use super::*;

    pub fn zap_out<'c: 'info, 'info>(
        ctx: Context<'_, '_, 'c, 'info, ZapOutCtx<'info>>,
        params: ZapOutParameters,
    ) -> Result<()> {
        // instructions::handle_zap_out(ctx, &params)
        Ok(())
    }
}
