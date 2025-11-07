#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;

pub mod instructions;
pub use instructions::*;
pub mod constants;
pub mod error;
pub mod math;
pub use math::*;
pub mod tests;
pub mod utils;
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

    /// helpful when user swap firstly, and want to transfer the delta of amount to the new token account
    pub fn transfer_delta_balance<'c: 'info, 'info>(
        ctx: Context<'_, '_, 'c, 'info, TransferBalanceCtx<'info>>,
        pre_source_token_balance: u64,
        max_transfer_amount: u64,
        remaining_accounts_info: RemainingAccountsInfo,
    ) -> Result<()> {
        instructions::handle_transfer_delta_balance(
            ctx,
            pre_source_token_balance,
            max_transfer_amount,
            remaining_accounts_info,
        )
    }

    /// helpful when user want to transfer full amount of a token account
    pub fn transfer_full_balance<'c: 'info, 'info>(
        ctx: Context<'_, '_, 'c, 'info, TransferBalanceCtx<'info>>,
        remaining_accounts_info: RemainingAccountsInfo,
    ) -> Result<()> {
        instructions::handle_transfer_full_balance(ctx, remaining_accounts_info)
    }

    /// same as full balance, but get capped by max_amount
    pub fn transfer_max_balance<'c: 'info, 'info>(
        ctx: Context<'_, '_, 'c, 'info, TransferBalanceCtx<'info>>,
        max_amount: u64,
        remaining_accounts_info: RemainingAccountsInfo,
    ) -> Result<()> {
        instructions::handle_transfer_max_balance(ctx, max_amount, remaining_accounts_info)
    }

    pub fn zap_in_damm_v2(
        ctx: Context<ZapInDammv2Ctx>,
        max_sqrt_price_change_bps: u32,
    ) -> Result<()> {
        instructions::handle_zap_in_damm_v2(ctx, max_sqrt_price_change_bps)
    }
}
