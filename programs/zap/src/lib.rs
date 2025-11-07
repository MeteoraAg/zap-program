#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;

pub mod instructions;
pub use instructions::*;
pub mod constants;
pub mod error;
pub mod math;
pub use math::*;
pub mod state;
pub mod tests;
pub use state::*;
pub mod damm_v2_ultils;
pub use damm_v2_ultils::*;
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

    pub fn initialize_ledger_account(ctx: Context<InitializeLedgerAccountCtx>) -> Result<()> {
        instructions::handle_initialize_ledger_account(ctx)
    }

    pub fn close_ledger_account(ctx: Context<CloseLedgerAccountCtx>) -> Result<()> {
        instructions::handle_close_ledger_account(ctx)
    }

    pub fn set_ledger_balance(
        ctx: Context<SetLedgerBalanceCtx>,
        amount: u64,
        is_token_a: bool,
    ) -> Result<()> {
        instructions::handle_set_ledger_balance(ctx, amount, is_token_a)
    }

    pub fn update_ledger_balance_after_swap(
        ctx: Context<UpdateLedgerBalanceAfterSwapCtx>,
        pre_source_token_balance: u64,
        max_transfer_amount: u64,
        is_token_a: bool,
    ) -> Result<()> {
        instructions::handle_update_ledger_balance_after_swap(
            ctx,
            pre_source_token_balance,
            max_transfer_amount,
            is_token_a,
        )
    }

    pub fn zap_in_damm_v2(
        ctx: Context<ZapInDammv2Ctx>,
        max_sqrt_price_change_bps: u32,
    ) -> Result<()> {
        instructions::handle_zap_in_damm_v2(ctx, max_sqrt_price_change_bps)
    }
}
