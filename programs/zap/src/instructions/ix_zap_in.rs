use std::cmp::min;

use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{
    error::ZapError, get_liquidity_for_adding_liquidity, safe_math::SafeMath,
    simulate_add_liquidity, simulate_swap, SimulateAddLiquidityResult,
};

use damm_v2_program::{
    params::swap::TradeDirection,
    state::{Pool, Position},
};

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

    /// CHECK: damm v2 pool authority
    pub damm_pool_authority: UncheckedAccount<'info>,

    /// Token a program
    pub token_a_program: Interface<'info, TokenInterface>,

    /// Token b program
    pub token_b_program: Interface<'info, TokenInterface>,
}

#[derive(Debug, PartialEq)]
pub struct SwapAction {
    pub amount_in: u64,
    pub trade_direction: TradeDirection,
}

#[derive(Debug, PartialEq)]
pub struct CalculateZapInResult {
    pub liquidity_delta: u128,
    pub swap_action: Option<SwapAction>,
}

impl<'info> ZapInDammV2Ctx<'info> {
    pub fn calculate_zap_in(
        &self,
        max_deposit_amount_a: u64,
        max_deposit_amount_b: u64,
        threshold_token_a: u64,
        threshold_token_b: u64,
    ) -> Result<CalculateZapInResult> {
        let pool = self.pool.load()?;
        let sqrt_price = pool.sqrt_price;
        let min_sqrt_price = pool.sqrt_min_price;
        let max_sqrt_price = pool.sqrt_max_price;

        drop(pool);

        let liquidity_delta = get_liquidity_for_adding_liquidity(
            max_deposit_amount_a,
            max_deposit_amount_b,
            sqrt_price,
            min_sqrt_price,
            max_sqrt_price,
        )?;

        require!(liquidity_delta > 0, ZapError::LiquidityDeltaIsZero);

        let SimulateAddLiquidityResult {
            token_a_amount,
            token_b_amount,
        } = simulate_add_liquidity(
            &self.pool,
            liquidity_delta,
            &self.token_a_mint,
            &self.token_b_mint,
        )?;

        msg!("max_deposit_amount_a 1: {:?}", max_deposit_amount_a);
        msg!("max_deposit_amount_b 1: {:?}", max_deposit_amount_b);
        msg!("token_a_amount 1: {:?}", token_a_amount);
        msg!("token_b_amount 1: {:?}", token_b_amount);

        if token_a_amount <= threshold_token_a && token_b_amount <= threshold_token_b {
            return Ok(CalculateZapInResult {
                liquidity_delta,
                swap_action: None,
            });
        }

        let remain_token_a_amount = max_deposit_amount_a.safe_sub(token_a_amount)?;
        let remain_token_b_amount = max_deposit_amount_b.safe_sub(token_b_amount)?;

        msg!("remain_token_a_amount 1: {:?}", remain_token_a_amount);
        msg!("remain_token_b_amount 1: {:?}", remain_token_b_amount);

        let (remain_amount, trade_direction) = if remain_token_a_amount > threshold_token_a {
            (remain_token_a_amount, TradeDirection::AtoB)
        } else {
            (remain_token_b_amount, TradeDirection::BtoA)
        };

        let amount_in = remain_amount.safe_div(2)?; // TODO fix this

        msg!("remain_amount: {:?}", remain_amount);
        msg!("trade_direction: {:?}", trade_direction);

        let amount_out = simulate_swap(
            &mut self.pool.load_mut()?,
            amount_in,
            trade_direction,
            &self.token_a_mint,
            &self.token_b_mint,
        )?;

        msg!("swap out: {:?}", amount_out);

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
            sqrt_price,
            min_sqrt_price,
            max_sqrt_price,
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

        msg!("token_a_amount: {:?}", token_a_amount);
        msg!("token_b_amount: {:?}", token_b_amount);

        Ok(CalculateZapInResult {
            liquidity_delta,
            swap_action: Some(SwapAction {
                amount_in,
                trade_direction,
            }),
        })
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

    let CalculateZapInResult {
        liquidity_delta,
        swap_action,
    } = ctx.accounts.calculate_zap_in(
        max_deposit_amount_a,
        max_deposit_amount_b,
        params.threshold_amount_a,
        params.threshold_amount_b,
    )?;

    if let Some(swap_action) = swap_action {
        let (input_token_account, output_token_account) =
            if swap_action.trade_direction == TradeDirection::AtoB {
                (
                    ctx.accounts.user_token_a_account.to_account_info(),
                    ctx.accounts.user_token_b_account.to_account_info(),
                )
            } else {
                (
                    ctx.accounts.user_token_b_account.to_account_info(),
                    ctx.accounts.user_token_a_account.to_account_info(),
                )
            };

        damm_v2::cpi::swap(
            CpiContext::new(
                ctx.accounts.amm_program.to_account_info(),
                damm_v2::cpi::accounts::Swap {
                    pool_authority: ctx.accounts.damm_pool_authority.to_account_info(),
                    pool: ctx.accounts.pool.to_account_info(),
                    input_token_account,
                    output_token_account,
                    token_a_vault: ctx.accounts.token_a_vault.to_account_info(),
                    token_b_vault: ctx.accounts.token_b_vault.to_account_info(),
                    token_a_mint: ctx.accounts.token_a_mint.to_account_info(),
                    token_b_mint: ctx.accounts.token_b_mint.to_account_info(),
                    payer: ctx.accounts.owner.to_account_info(),
                    token_a_program: ctx.accounts.token_a_program.to_account_info(),
                    token_b_program: ctx.accounts.token_b_program.to_account_info(),
                    referral_token_account: None,
                    event_authority: ctx.accounts.damm_event_authority.to_account_info(),
                    program: ctx.accounts.amm_program.to_account_info(),
                },
            ),
            damm_v2::types::SwapParameters {
                amount_in: swap_action.amount_in,
                minimum_amount_out: 0, // TODO check this
            },
        )?;
    }

    // TODO use invoke
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
        damm_v2::types::AddLiquidityParameters {
            liquidity_delta,
            token_a_amount_threshold: u64::MAX,
            token_b_amount_threshold: u64::MAX,
        },
    )?;

    ctx.accounts.user_token_a_account.reload()?;
    ctx.accounts.user_token_b_account.reload()?;

    let post_add_liquidity_token_a_balance = ctx.accounts.user_token_a_account.amount;
    let post_add_liquidity_token_b_balance = ctx.accounts.user_token_b_account.amount;
    let deposited_token_a = token_a_balance.safe_sub(post_add_liquidity_token_a_balance)?;
    let deposited_token_b = token_b_balance.safe_sub(post_add_liquidity_token_b_balance)?;

    msg!("deposited_token_a: {:?}", deposited_token_a);
    msg!("deposited_token_b: {:?}", deposited_token_b);

    require!(
        max_deposit_amount_a.safe_sub(deposited_token_a)? <= params.threshold_amount_a,
        ZapError::RemainingAmountIsOverThreshold
    );

    require!(
        max_deposit_amount_b.safe_sub(deposited_token_b)? < params.threshold_amount_b,
        ZapError::RemainingAmountIsOverThreshold
    );

    Ok(())
}
