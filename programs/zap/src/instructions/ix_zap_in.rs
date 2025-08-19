use std::cmp::min;

use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{
    constants::MAX_SIMULATE_ZAP_IN, error::ZapError, get_liquidity_for_adding_liquidity,
    safe_math::SafeMath, simulate_add_liquidity, simulate_swap, SimulateAddLiquidityResult,
};

use damm_v2_program::{params::swap::TradeDirection, state::Pool};

use damm_v2::{accounts::Position, types::AddLiquidityParameters};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ZapInParameters {
    pub pre_token_a_balance: u64,
    pub pre_token_b_balance: u64,
    pub amount_a_0: u64, // amount a user want to zap in and deposit
    pub amount_b_0: u64, // amount b user want to zap in and deposit
    pub threshold_amount_a: u64,
    pub threshold_amount_b: u64,
    pub max_deposit_amount_a: u64,
    pub max_deposit_amount_b: u64,
}

#[derive(Accounts)]
pub struct ZapInDammV2Ctx<'info> {
    #[account(mut, has_one = token_a_vault, has_one = token_b_vault, has_one = token_a_mint, has_one = token_b_mint)]
    pub pool: AccountLoader<'info, Pool>,

    #[account(
        mut,
        has_one = pool,
      )]
    pub position: AccountLoader<'info, Position>,

    /// CHECK: The vault token account for token a
    #[account(mut)]
    pub token_a_vault: UncheckedAccount<'info>,

    /// CHECK: The vault token account for token b
    #[account(mut)]
    pub token_b_vault: UncheckedAccount<'info>,

    /// The mint of token a
    pub token_a_mint: Box<InterfaceAccount<'info, Mint>>,

    /// The mint of token b
    pub token_b_mint: Box<InterfaceAccount<'info, Mint>>,

    /// The token account for nft
    #[account(
        constraint = position_nft_account.mint == position.load()?.nft_mint,
        constraint = position_nft_account.amount == 1,
        token::authority = owner
)]
    pub position_nft_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut)]
    pub user_token_a_account: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub user_token_b_account: InterfaceAccount<'info, TokenAccount>,

    /// owner of position
    pub owner: Signer<'info>,

    /// CHECK:
    pub amm_program: UncheckedAccount<'info>,

    /// CHECK: damm v2 event authority
    pub damm_event_authority: UncheckedAccount<'info>,

    /// Token a program
    pub token_a_program: Interface<'info, TokenInterface>,

    /// Token b program
    pub token_b_program: Interface<'info, TokenInterface>,
}

#[derive(Debug, PartialEq)]
pub struct CalculateZapInResult {
    pub liquidity_delta: u128,
    pub amount_in: u64,
    pub trade_direction: TradeDirection,
}

impl<'info> ZapInDammV2Ctx<'info> {
    pub fn calculate_zap_in(
        &self,
        token_a_amount: u64,
        token_b_amount: u64,
        max_deposit_amount_a: u64,
        max_deposit_amount_b: u64,
        threshold_token_a: u64,
        threshold_token_b: u64,
    ) -> Result<CalculateZapInResult> {
        let pool = self.pool.load()?;
        for _ in 0..MAX_SIMULATE_ZAP_IN {
            let (remain_amount, trade_direction) = if token_a_amount > threshold_token_a {
                (
                    token_a_amount.safe_sub(threshold_token_a)?,
                    TradeDirection::AtoB,
                )
            } else {
                (
                    token_b_amount.safe_sub(threshold_token_b)?,
                    TradeDirection::BtoA,
                )
            };

            let amount_in = remain_amount.safe_div(2)?;

            let amount_out = simulate_swap(
                &self.pool,
                amount_in,
                trade_direction,
                &self.token_a_mint,
                &self.token_b_mint,
            )?;

            let (max_deposit_amount_a, max_deposit_amount_b) =
                if trade_direction == TradeDirection::AtoB {
                    (
                        max_deposit_amount_a.safe_sub(amount_in)?,
                        max_deposit_amount_b.safe_add(amount_out)?,
                    )
                } else {
                    (
                        max_deposit_amount_a.safe_add(amount_in)?,
                        max_deposit_amount_b.safe_sub(amount_out)?,
                    )
                };

            let liquidity_delta = get_liquidity_for_adding_liquidity(
                max_deposit_amount_a,
                max_deposit_amount_b,
                pool.sqrt_price,
                pool.sqrt_min_price,
                pool.sqrt_max_price,
            )?;

            let SimulateAddLiquidityResult {
                token_a_amount,
                token_b_amount,
            } = simulate_add_liquidity(
                &self.pool,
                liquidity_delta,
                &self.token_a_mint,
                &self.token_b_mint,
            )?;

            if token_a_amount <= threshold_token_a && token_b_amount <= threshold_token_b {
                return Ok(CalculateZapInResult {
                    liquidity_delta,
                    amount_in,
                    trade_direction,
                });
            }
        }

        return Err(ZapError::LiquidityDeltaIsZero.into());
    }
}

pub fn handle_zap_in(ctx: Context<ZapInDammV2Ctx>, params: &ZapInParameters) -> Result<()> {
    let token_a_balance = ctx.accounts.user_token_a_account.amount;
    let token_b_balance = ctx.accounts.user_token_b_account.amount;

    let amount_a_to_deposit = params
        .amount_a_0
        .safe_add(token_a_balance)?
        .safe_sub(params.pre_token_a_balance)?;

    let amount_b_to_deposit = params
        .amount_b_0
        .safe_add(token_b_balance)?
        .safe_sub(params.pre_token_b_balance)?;

    let max_deposit_amount_a = min(params.max_deposit_amount_a, amount_a_to_deposit);
    let max_deposit_amount_b = min(params.max_deposit_amount_b, amount_b_to_deposit);

    let pool = ctx.accounts.pool.load()?;

    let liquidity_delta = get_liquidity_for_adding_liquidity(
        max_deposit_amount_a,
        max_deposit_amount_b,
        pool.sqrt_price,
        pool.sqrt_min_price,
        pool.sqrt_max_price,
    )?;

    require!(liquidity_delta > 0, ZapError::LiquidityDeltaIsZero);

    let SimulateAddLiquidityResult {
        token_a_amount,
        token_b_amount,
    } = simulate_add_liquidity(
        &ctx.accounts.pool,
        liquidity_delta,
        &ctx.accounts.token_a_mint,
        &ctx.accounts.token_b_mint,
    )?;

    // let CalculateZapInResult {
    //     liquidity_delta,
    //     amount_in,
    //     trade_direction,
    // } = if token_a_amount > params.threshold_amount_b || token_b_amount > params.threshold_amount_b
    // {
    //     ctx.accounts.calculate_zap_in(
    //         token_a_amount,
    //         token_b_amount,
    //         max_deposit_amount_a,
    //         max_deposit_amount_b,
    //         params.threshold_amount_a,
    //         params.threshold_amount_b,
    //     )?
    // } else {

    // }

    drop(pool);

    damm_v2::cpi::add_liquidity(
        CpiContext::new(
            ctx.accounts.amm_program.to_account_info(),
            damm_v2::cpi::accounts::AddLiquidity {
                pool: ctx.accounts.pool.to_account_info(),
                position: ctx.accounts.position.to_account_info(),
                token_a_account: ctx.accounts.user_token_a_account.to_account_info(),
                token_b_account: ctx.accounts.user_token_b_account.to_account_info(),
                token_a_vault: ctx.accounts.token_a_vault.to_account_info(),
                token_b_vault: ctx.accounts.token_b_vault.to_account_info(),
                token_a_mint: ctx.accounts.token_a_mint.to_account_info(),
                token_b_mint: ctx.accounts.token_b_mint.to_account_info(),
                position_nft_account: ctx.accounts.position_nft_account.to_account_info(),
                owner: ctx.accounts.owner.to_account_info(),
                token_a_program: ctx.accounts.token_a_program.to_account_info(),
                token_b_program: ctx.accounts.token_b_program.to_account_info(),
                event_authority: ctx.accounts.damm_event_authority.to_account_info(),
                program: ctx.accounts.amm_program.to_account_info(),
            },
        ),
        AddLiquidityParameters {
            liquidity_delta,
            token_a_amount_threshold: u64::MAX,
            token_b_amount_threshold: u64::MAX,
        },
    )?;

    ctx.accounts.user_token_a_account.reload()?;
    ctx.accounts.user_token_b_account.reload()?;

    let post_add_liquidity_token_a_balance = ctx.accounts.user_token_a_account.amount;
    let post_add_liquidity_token_b_balance = ctx.accounts.user_token_b_account.amount;

    require!(
        max_deposit_amount_a.safe_sub(post_add_liquidity_token_a_balance)?
            <= params.threshold_amount_a,
        ZapError::RemainingAmountIsOverThreshold
    );

    require!(
        max_deposit_amount_b.safe_sub(post_add_liquidity_token_b_balance)?
            < params.threshold_amount_b,
        ZapError::RemainingAmountIsOverThreshold
    );

    Ok(())
}
