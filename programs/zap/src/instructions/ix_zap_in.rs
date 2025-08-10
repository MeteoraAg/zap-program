use std::cmp::min;

use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use ruint::aliases::{U256, U512};

use crate::{error::ZapError, safe_math::SafeMath};

use damm_v2::{
    accounts::{Pool, Position},
    types::AddLiquidityParameters,
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ZapInParameters {
    pub pre_token_a_balance: u64,
    pub pre_token_b_balance: u64,
    pub theshold_amount_a: u64,
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

    /// The vault token account for input token
    #[account(mut, token::token_program = token_a_program, token::mint = token_a_mint)]
    pub token_a_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The vault token account for output token
    #[account(mut, token::token_program = token_b_program, token::mint = token_b_mint)]
    pub token_b_vault: Box<InterfaceAccount<'info, TokenAccount>>,

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
// Δa = L * (1 / √P_lower - 1 / √P_upper)
// Δa = L * (√P_upper - √P_lower) / (√P_upper * √P_lower)
// L = Δa * √P_upper * √P_lower / (√P_upper - √P_lower)
fn calculate_liquidity_delta_from_amount_a(
    max_amount_a: u64,
    lower_sqrt_price: u128,
    upper_sqrt_price: u128,
) -> Result<u128> {
    let numerator_1 = U512::from(max_amount_a);
    let numerator_2 = U512::from(upper_sqrt_price);
    let numerator_3 = U512::from(lower_sqrt_price);
    let product = numerator_1.safe_mul(numerator_2)?.safe_mul(numerator_3)?;
    let denominator = U512::from(upper_sqrt_price.safe_sub(lower_sqrt_price)?);

    assert!(denominator > U512::ZERO);

    let result = product.safe_div(denominator)?;
    Ok(result.try_into().map_err(|_| ZapError::TypeCastFailed)?)
}

// Δb = L (√P_upper - √P_lower)
// L = Δb / (√P_upper - √P_lower)
fn calculate_liquidity_delta_from_amount_b(
    max_amount_b: u64,
    lower_sqrt_price: u128,
    upper_sqrt_price: u128,
) -> Result<u128> {
    let denominator = upper_sqrt_price.safe_sub(lower_sqrt_price)?;
    assert!(denominator > 0);

    let product = U256::from(max_amount_b).safe_shl(128)?;
    let result = product.safe_div(U256::from(denominator))?;
    Ok(result.try_into().map_err(|_| ZapError::TypeCastFailed)?)
}

fn calculate_liquidity_delta(
    max_amount_a: u64,
    max_amount_b: u64,
    current_sqrt_price: u128,
    min_sqrt_price: u128,
    max_sqrt_price: u128,
) -> Result<u128> {
    let liquidity_delta_a =
        calculate_liquidity_delta_from_amount_a(max_amount_a, current_sqrt_price, max_sqrt_price)?;

    let liquidity_delta_b =
        calculate_liquidity_delta_from_amount_b(max_amount_b, min_sqrt_price, current_sqrt_price)?;

    Ok(min(liquidity_delta_a, liquidity_delta_b))
}

pub fn handle_zap_in(ctx: Context<ZapInDammV2Ctx>, params: &ZapInParameters) -> Result<()> {
    let token_a_balance = ctx.accounts.user_token_a_account.amount;
    let token_b_balance = ctx.accounts.user_token_b_account.amount;

    let token_a_changed = if params.pre_token_a_balance > token_a_balance {
        params.pre_token_a_balance.safe_sub(token_a_balance)?
    } else {
        token_a_balance.safe_sub(params.pre_token_a_balance)?
    };

    let token_b_changed = if params.pre_token_b_balance > token_b_balance {
        params.pre_token_b_balance.safe_sub(token_b_balance)?
    } else {
        token_b_balance.safe_sub(params.pre_token_b_balance)?
    };
    let pool = ctx.accounts.pool.load()?;

    let max_deposit_amount_a = min(params.max_deposit_amount_a, token_a_changed);
    let max_deposit_amount_b = min(params.max_deposit_amount_b, token_b_changed);

    let liquidity_delta = calculate_liquidity_delta(
        max_deposit_amount_a,
        max_deposit_amount_b,
        pool.sqrt_price,
        pool.sqrt_min_price,
        pool.sqrt_max_price,
    )?;

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
        token_a_balance.safe_sub(post_add_liquidity_token_a_balance)? <= params.theshold_amount_a,
        ZapError::RemainingAmountIsOverThreshold
    );

    require!(
        token_b_balance.safe_sub(post_add_liquidity_token_b_balance)? < params.threshold_amount_b,
        ZapError::RemainingAmountIsOverThreshold
    );

    Ok(())
}
