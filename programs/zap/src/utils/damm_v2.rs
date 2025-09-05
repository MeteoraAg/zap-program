use std::cell::{Ref, RefMut};

use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;
use ruint::aliases::{U256, U512};

use crate::{error::ZapError, safe_math::SafeMath};

use damm_v2_program::{
    activation_handler::ActivationHandler,
    params::swap::TradeDirection,
    state::{fee::FeeMode, pool::Pool, ModifyLiquidityResult},
    token::{
        calculate_transfer_fee_excluded_amount, calculate_transfer_fee_included_amount,
        TransferFeeExcludedAmount, TransferFeeIncludedAmount,
    },
    u128x128_math::Rounding,
    PoolError,
};

// Δa = L * (1 / √P_lower - 1 / √P_upper) => L = Δa / (1 / √P_lower - 1 / √P_upper)
pub fn get_liquidity_delta_from_token_a(
    base_amount: u64,
    sqrt_max_price: u128,
    sqrt_price: u128,
) -> Result<U512> {
    let price_delta = U512::from(sqrt_max_price.safe_sub(sqrt_price)?);
    let prod = U512::from(base_amount)
        .safe_mul(U512::from(sqrt_price))?
        .safe_mul(U512::from(sqrt_max_price))?;
    let liquidity = prod.safe_div(price_delta)?; // round down
    Ok(liquidity)
}

// Δb = L (√P_upper - √P_lower) => L = Δb / (√P_upper - √P_lower)
pub fn get_liquidity_delta_from_token_b(
    quote_amount: u64,
    sqrt_min_price: u128,
    sqrt_price: u128,
) -> Result<u128> {
    let price_delta = U256::from(sqrt_price.safe_sub(sqrt_min_price)?);
    let quote_amount = U256::from(quote_amount).safe_shl(128)?;
    let liquidity = quote_amount.safe_div(price_delta)?; // round down
    return Ok(liquidity.try_into().map_err(|_| ZapError::TypeCastFailed)?);
}

pub fn get_liquidity_for_adding_liquidity(
    amount_a: u64,
    amount_b: u64,
    sqrt_price: u128,
    min_sqrt_price: u128,
    max_sqrt_price: u128,
) -> Result<u128> {
    let liquidity_from_token_a =
        get_liquidity_delta_from_token_a(amount_a, max_sqrt_price, sqrt_price)?;
    let liquidity_from_token_b =
        get_liquidity_delta_from_token_b(amount_b, min_sqrt_price, sqrt_price)?;

    if liquidity_from_token_a > U512::from(liquidity_from_token_b) {
        Ok(liquidity_from_token_b)
    } else {
        Ok(liquidity_from_token_a
            .try_into()
            .map_err(|_| ZapError::TypeCastFailed)?)
    }
}

pub fn simulate_swap<'info>(
    pool: &mut RefMut<'_, Pool>,
    amount_in: u64,
    trade_direction: TradeDirection,
    token_a_mint: &InterfaceAccount<'info, Mint>,
    token_b_mint: &InterfaceAccount<'info, Mint>,
) -> Result<u64> {
    // Parse accounts
    let (token_in_mint, token_out_mint) = if trade_direction == TradeDirection::AtoB {
        (token_a_mint, token_b_mint)
    } else {
        (token_a_mint, token_b_mint)
    };

    let TransferFeeExcludedAmount {
        amount: transfer_fee_excluded_amount_in,
        ..
    } = calculate_transfer_fee_excluded_amount(token_in_mint, amount_in)?;
    require!(transfer_fee_excluded_amount_in > 0, PoolError::AmountIsZero);

    let fee_mode = &FeeMode::get_fee_mode(pool.collect_fee_mode, trade_direction, false)?;
    let current_timestamp = Clock::get()?.unix_timestamp as u64;
    let current_point = ActivationHandler::get_current_point(pool.activation_type)?;

    pool.update_pre_swap(current_timestamp)?;

    let swap_result = pool.get_swap_result(
        transfer_fee_excluded_amount_in,
        fee_mode,
        trade_direction,
        current_point,
    )?;
    // Apply the swap result
    pool.apply_swap_result(&swap_result, fee_mode, current_timestamp)?;

    let TransferFeeExcludedAmount {
        amount: transfer_fee_excluded_amount_out,
        ..
    } = calculate_transfer_fee_excluded_amount(token_out_mint, swap_result.output_amount)?;
    Ok(transfer_fee_excluded_amount_out)
}

pub fn simulate_add_liquidity<'info>(
    pool: &AccountLoader<'info, Pool>,
    liquidity_delta: u128,
    token_a_mint: &InterfaceAccount<'info, Mint>,
    token_b_mint: &InterfaceAccount<'info, Mint>,
) -> Result<SimulateAddLiquidityResult> {
    let pool = pool.load()?;
    let ModifyLiquidityResult {
        token_a_amount,
        token_b_amount,
    } = pool.get_amounts_for_modify_liquidity(liquidity_delta, Rounding::Up)?;

    let TransferFeeIncludedAmount {
        amount: transfer_fee_included_token_a_amount,
        ..
    } = calculate_transfer_fee_included_amount(token_a_mint, token_a_amount)?;
    let TransferFeeIncludedAmount {
        amount: transfer_fee_included_token_b_amount,
        ..
    } = calculate_transfer_fee_included_amount(token_b_mint, token_b_amount)?;

    drop(pool);

    Ok(SimulateAddLiquidityResult {
        token_a_amount: transfer_fee_included_token_a_amount,
        token_b_amount: transfer_fee_included_token_b_amount,
    })
}

#[derive(Debug, PartialEq)]
pub struct SimulateAddLiquidityResult {
    pub token_a_amount: u64,
    pub token_b_amount: u64,
}
