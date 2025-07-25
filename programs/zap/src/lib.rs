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
pub mod error;

declare_id!("zapvX9M3uf5pvy4wRPAbQgdQsM1xmuiFnkfHKPvwMiz");

#[program]
pub mod zap {
    use super::*;

    pub fn initialize_token_ledger(ctx: Context<InitializeTokenLedgerCtx>) -> Result<()> {
        instructions::handle_initialize_token_ledger(ctx)
    }

    pub fn zap_out<'c: 'info, 'info>(
        ctx: Context<'_, '_, 'c, 'info, ZapOutCtx<'info>>,
        action_type: u8,
        payload_data: Vec<u8>,
    ) -> Result<()> {
        instructions::handle_zap_out(ctx, action_type, payload_data)
    }

    pub fn zap_in(
        ctx: Context<ZapInDammV2Ctx>,
        params: ZapInDammV2Parameters,
    ) -> Result<Vec<ZapInDammV2Result>> {
        instructions::handle_zap_in_damm_v2(ctx, params)
    }
}
