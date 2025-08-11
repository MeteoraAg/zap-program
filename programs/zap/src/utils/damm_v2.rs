use anchor_lang::prelude::*;
use ruint::aliases::{U256, U512};

use crate::{error::ZapError, safe_math::SafeMath};

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
