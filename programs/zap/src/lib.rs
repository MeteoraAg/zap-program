#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;

pub mod instructions;
pub use instructions::*;
pub mod constants;
pub mod error;
pub mod math;
pub use math::*;
pub mod state;
#[cfg(test)]
pub mod tests;
pub use state::*;
pub mod utils;
use dlmm::types::RemainingAccountsInfo;
use protocol_zap::ZapOutParameters;
pub use utils::*;

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

    pub fn zap_in_damm_v2<'c: 'info, 'info>(
        ctx: Context<'_, '_, 'c, 'info, ZapInDammv2Ctx<'info>>,
        pre_sqrt_price: u128,
        max_sqrt_price_change_bps: u32,
    ) -> Result<()> {
        instructions::handle_zap_in_damm_v2(ctx, pre_sqrt_price, max_sqrt_price_change_bps)
    }

    pub fn zap_in_dlmm_for_initialized_position<'c: 'info, 'info>(
        ctx: Context<'_, '_, 'c, 'info, ZapInDlmmForInitializedPositionCtx<'info>>,
        active_id: i32,
        min_delta_id: i32,
        max_delta_id: i32,
        max_active_bin_slippage: u16,
        favor_x_in_active_id: bool,
        strategy: StrategyType,
        remaining_accounts_info: RemainingAccountsInfo,
    ) -> Result<()> {
        instructions::handle_zap_in_dlmm_for_initialized_position(
            ctx,
            active_id,
            max_active_bin_slippage,
            min_delta_id,
            max_delta_id,
            favor_x_in_active_id,
            strategy,
            remaining_accounts_info,
        )
    }

    pub fn zap_in_dlmm_for_uninitialized_position<'c: 'info, 'info>(
        ctx: Context<'_, '_, 'c, 'info, ZapInDlmmForUnintializedPositionCtx<'info>>,
        min_delta_id: i32,
        max_delta_id: i32,
        active_id: i32,
        max_active_bin_slippage: u16,
        favor_x_in_active_id: bool,
        strategy: StrategyType,
        remaining_accounts_info: RemainingAccountsInfo,
    ) -> Result<()> {
        instructions::handle_zap_in_dlmm_for_uninitialized_position(
            ctx,
            min_delta_id,
            max_delta_id,
            active_id,
            max_active_bin_slippage,
            favor_x_in_active_id,
            strategy,
            remaining_accounts_info,
        )
    }
}
