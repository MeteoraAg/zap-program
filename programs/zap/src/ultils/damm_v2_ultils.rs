use anchor_lang::prelude::*;
use damm_v2::{
    base_fee::{BaseFeeHandler, FeeRateLimiter},
    constants::fee::get_max_fee_numerator,
    curve::{
        get_delta_amount_a_unsigned, get_delta_amount_b_unsigned, get_next_sqrt_price_from_input,
    },
    params::swap::TradeDirection,
    state::{
        fee::{BaseFeeMode, FeeMode, FeeOnAmountResult},
        Pool,
    },
    u128x128_math::Rounding,
    PoolError,
};
use ruint::aliases::{U192, U256, U512};

use crate::{
    constants::MAX_BASIS_POINT, error::ZapError, safe_math::SafeMath, TransferFeeCalculator,
};

struct SwapAmountFromInput {
    output_amount: u64,
}

pub fn get_swap_result_status(
    token_a_transfer_fee_calculator: &TransferFeeCalculator,
    token_b_transfer_fee_calculator: &TransferFeeCalculator,
    token_a_amount: u64,
    token_b_amount: u64,
    total_amount_a: u64,
    total_amount_b: u64,
) -> Result<SwapResultStatus> {
    let exclude_transfer_fee_token_a_amount = token_a_transfer_fee_calculator
        .calculate_transfer_fee_excluded_amount(token_a_amount)?
        .amount;

    let exclude_transfer_fee_token_b_amount = token_b_transfer_fee_calculator
        .calculate_transfer_fee_excluded_amount(token_b_amount)?
        .amount;
    // if total_amount_a and total_amount_b is zero, it will return error, but outside function will skip that
    let r1 = u128::from(exclude_transfer_fee_token_a_amount)
        .safe_shl(64)?
        .safe_div(u128::from(total_amount_a))?;
    let r2 = u128::from(exclude_transfer_fee_token_b_amount)
        .safe_shl(64)?
        .safe_div(u128::from(total_amount_b))?;

    // compare a / Ta with b / Tb
    // if a / Ta > b / Tb => Exceed A
    // if a / Ta <= b / Tb => Exceed B
    let diff = if r1 > r2 {
        r1.safe_sub(r2)?
    } else {
        r2.safe_sub(r1)?
    };

    // if ratio (r1-r2) * 2 / (r1+r2) is less than 0.1%, we can stop
    let prod = U192::from(diff)
        .safe_mul(U192::from(2000))?
        .safe_div(U192::from(r1).safe_add(U192::from(r2))?)?;

    if prod.is_zero() {
        Ok(SwapResultStatus::Done)
    } else {
        if r1 > r2 {
            Ok(SwapResultStatus::ExceededA)
        } else {
            Ok(SwapResultStatus::ExceededB)
        }
    }
}

fn calculate_a_to_b_from_amount_in(pool: &Pool, amount_in: u64) -> Result<SwapAmountFromInput> {
    // finding new target price
    let next_sqrt_price =
        get_next_sqrt_price_from_input(pool.sqrt_price, pool.liquidity, amount_in, true)?;

    if next_sqrt_price < pool.sqrt_min_price {
        return Err(PoolError::PriceRangeViolation.into());
    }

    // finding output amount
    let output_amount = get_delta_amount_b_unsigned(
        next_sqrt_price,
        pool.sqrt_price,
        pool.liquidity,
        Rounding::Down,
    )?;

    Ok(SwapAmountFromInput { output_amount })
}

fn calculate_b_to_a_from_amount_in(pool: &Pool, amount_in: u64) -> Result<SwapAmountFromInput> {
    // finding new target price
    let next_sqrt_price =
        get_next_sqrt_price_from_input(pool.sqrt_price, pool.liquidity, amount_in, false)?;

    if next_sqrt_price > pool.sqrt_max_price {
        return Err(PoolError::PriceRangeViolation.into());
    }
    // finding output amount
    let output_amount = get_delta_amount_a_unsigned(
        pool.sqrt_price,
        next_sqrt_price,
        pool.liquidity,
        Rounding::Down,
    )?;

    Ok(SwapAmountFromInput { output_amount })
}

struct SimulateSwapResult {
    user_amount_in: u64,
    user_amount_out: u64,
    pool_amount_in: u64,
    pool_amount_out: u64,
}

/// Replicate exactly how swap_exact_in work
fn calculate_swap_result(
    pool: &Pool,
    token_a_transfer_fee_calculator: &TransferFeeCalculator,
    token_b_transfer_fee_calculator: &TransferFeeCalculator,
    current_point: u64,
    amount_in: u64,
    trade_direction: TradeDirection,
    fee_handler: &FeeHandler,
    fee_mode: &FeeMode,
) -> Result<SimulateSwapResult> {
    let excluded_fee_amount_in = if trade_direction == TradeDirection::AtoB {
        token_a_transfer_fee_calculator
            .calculate_transfer_fee_excluded_amount(amount_in)?
            .amount
    } else {
        token_b_transfer_fee_calculator
            .calculate_transfer_fee_excluded_amount(amount_in)?
            .amount
    };
    let trade_fee_numerator = fee_handler.get_trade_fee_numerator(
        excluded_fee_amount_in,
        current_point,
        pool.activation_point,
        trade_direction,
    )?;
    let actual_amount_in = if fee_mode.fees_on_input {
        let FeeOnAmountResult { amount, .. } = pool.pool_fees.get_fee_on_amount(
            excluded_fee_amount_in,
            trade_fee_numerator,
            fee_mode.has_referral,
            false,
        )?;

        amount
    } else {
        excluded_fee_amount_in
    };
    let SwapAmountFromInput { output_amount, .. } = match trade_direction {
        TradeDirection::AtoB => calculate_a_to_b_from_amount_in(pool, actual_amount_in),
        TradeDirection::BtoA => calculate_b_to_a_from_amount_in(pool, actual_amount_in),
    }?;

    let actual_amount_out = if fee_mode.fees_on_input {
        output_amount
    } else {
        let FeeOnAmountResult { amount, .. } = pool.pool_fees.get_fee_on_amount(
            output_amount,
            trade_fee_numerator,
            fee_mode.has_referral,
            false,
        )?;
        amount
    };

    let excluded_fee_amount_out = if trade_direction == TradeDirection::AtoB {
        token_b_transfer_fee_calculator
            .calculate_transfer_fee_excluded_amount(actual_amount_out)?
            .amount
    } else {
        token_a_transfer_fee_calculator
            .calculate_transfer_fee_excluded_amount(actual_amount_out)?
            .amount
    };

    Ok(SimulateSwapResult {
        user_amount_in: excluded_fee_amount_out,
        user_amount_out: amount_in,
        pool_amount_in: actual_amount_in,
        pool_amount_out: output_amount,
    })
}

/// swap result status
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, AnchorDeserialize, AnchorSerialize)]
pub enum SwapResultStatus {
    Done,
    ExceededA,
    ExceededB,
}

fn validate_swap_result(
    swap_result: &SimulateSwapResult,
    token_a_transfer_fee_calculator: &TransferFeeCalculator,
    token_b_transfer_fee_calculator: &TransferFeeCalculator,
    remaining_amount: u64,
    total_amount_a: u64,
    total_amount_b: u64,
    trade_direction: TradeDirection,
) -> Result<SwapResultStatus> {
    let &SimulateSwapResult {
        user_amount_in,
        user_amount_out,
        pool_amount_in,
        pool_amount_out,
    } = swap_result;
    // apply swap result
    if trade_direction == TradeDirection::AtoB {
        let user_amount_a = remaining_amount.safe_sub(user_amount_out)?;
        let user_amount_b = user_amount_in;
        let pool_amount_a = total_amount_a.safe_add(pool_amount_in)?;
        let pool_amount_b = total_amount_b.safe_sub(pool_amount_out)?;
        get_swap_result_status(
            token_a_transfer_fee_calculator,
            token_b_transfer_fee_calculator,
            user_amount_a,
            user_amount_b,
            pool_amount_a,
            pool_amount_b,
        )
    } else {
        let user_amount_a = user_amount_in;
        let user_amount_b = remaining_amount.safe_sub(user_amount_out)?;
        let pool_amount_a = total_amount_a.safe_sub(pool_amount_out)?;
        let pool_amount_b = total_amount_b.safe_add(pool_amount_in)?;
        get_swap_result_status(
            token_a_transfer_fee_calculator,
            token_b_transfer_fee_calculator,
            user_amount_a,
            user_amount_b,
            pool_amount_a,
            pool_amount_b,
        )
    }
}

struct FeeHandler {
    pub rate_limiter_handler: FeeRateLimiter, // avoid copy
    pub variable_fee_numerator: u128,
    pub max_fee_numerator: u64,
    pub total_fee_numerator: u64,
    pub is_rate_limiter: bool,
}

impl FeeHandler {
    pub fn get_trade_fee_numerator(
        &self,
        input_amount: u64,
        current_point: u64,
        activation_point: u64,
        trade_direction: TradeDirection,
    ) -> Result<u64> {
        if self.is_rate_limiter {
            let base_fee_numerator = self
                .rate_limiter_handler
                .get_base_fee_numerator_from_included_fee_amount(
                    current_point,
                    activation_point,
                    trade_direction,
                    input_amount,
                )?;

            get_total_fee_numerator(
                base_fee_numerator,
                self.variable_fee_numerator,
                self.max_fee_numerator,
            )
        } else {
            Ok(self.total_fee_numerator)
        }
    }
}

fn get_fee_handler(
    pool: &Pool,
    current_point: u64,
    trade_direction: TradeDirection,
) -> Result<FeeHandler> {
    let variable_fee_numerator = pool.pool_fees.dynamic_fee.get_variable_fee()?;
    let max_fee_numerator = get_max_fee_numerator(pool.version)?;

    let base_fee_mode = pool.pool_fees.base_fee.base_fee_mode;
    match BaseFeeMode::try_from(base_fee_mode) {
        Ok(value) => {
            match value {
                BaseFeeMode::FeeSchedulerLinear | BaseFeeMode::FeeSchedulerExponential => {
                    let base_fee_handler = pool.pool_fees.base_fee.get_base_fee_handler()?;
                    // fee scheduler doesn't care for amount
                    let base_fee_numerator = base_fee_handler
                        .get_base_fee_numerator_from_included_fee_amount(
                            current_point,
                            pool.activation_point,
                            trade_direction,
                            0,
                        )?;

                    let total_fee_numerator = get_total_fee_numerator(
                        base_fee_numerator,
                        variable_fee_numerator,
                        max_fee_numerator,
                    )?;
                    Ok(FeeHandler {
                        rate_limiter_handler: FeeRateLimiter::default(),
                        variable_fee_numerator,
                        max_fee_numerator,
                        total_fee_numerator,
                        is_rate_limiter: false,
                    })
                }
                BaseFeeMode::RateLimiter => {
                    let rate_limiter_handler = pool.pool_fees.base_fee.get_fee_rate_limiter()?;
                    Ok(FeeHandler {
                        rate_limiter_handler,
                        total_fee_numerator: 0,
                        variable_fee_numerator,
                        max_fee_numerator,
                        is_rate_limiter: true,
                    })
                }
            }
        }
        _ => {
            // otherwise, we just use cliff_fee_numerator
            // that is in case the damm v2 update for the new base fee function
            let base_fee_numerator = pool.pool_fees.base_fee.cliff_fee_numerator;
            let total_fee_numerator = get_total_fee_numerator(
                base_fee_numerator,
                variable_fee_numerator,
                max_fee_numerator,
            )?;
            Ok(FeeHandler {
                rate_limiter_handler: FeeRateLimiter::default(),
                variable_fee_numerator,
                max_fee_numerator,
                total_fee_numerator,
                is_rate_limiter: false,
            })
        }
    }
}

fn get_total_fee_numerator(
    base_fee_numerator: u64,
    variable_fee_numerator: u128,
    max_fee_numerator: u64,
) -> Result<u64> {
    let total_fee_numerator = variable_fee_numerator.safe_add(base_fee_numerator.into())?;
    let total_fee_numerator: u64 = total_fee_numerator
        .try_into()
        .map_err(|_| ZapError::TypeCastFailed)?;

    if total_fee_numerator > max_fee_numerator {
        Ok(max_fee_numerator)
    } else {
        Ok(total_fee_numerator)
    }
}
// we will use binary search
pub fn calculate_swap_amount(
    pool: &Pool,
    token_a_transfer_fee_calculator: &TransferFeeCalculator,
    token_b_transfer_fee_calculator: &TransferFeeCalculator,
    remaining_amount: u64,
    trade_direction: TradeDirection,
    current_point: u64,
) -> Result<u64> {
    let mut max_swap_amount = remaining_amount;
    let mut min_swap_amount = 0;
    let mut swap_amount = 0;

    let fee_handler = get_fee_handler(pool, current_point, trade_direction)?;

    let fee_mode = FeeMode::get_fee_mode(pool.collect_fee_mode, trade_direction, false)?;

    let (pool_amount_a, pool_amount_b) = pool.get_reserves_amount()?;

    // max 20 loops
    // For each loop program consumed ~ 5.19 CUs
    // So the 20 loops will consume maximum ~ 100 CUs
    for _i in 0..20 {
        let amount_in = max_swap_amount.safe_add(min_swap_amount)?.safe_div(2)?;

        if let Ok(swap_result) = calculate_swap_result(
            pool,
            token_a_transfer_fee_calculator,
            token_b_transfer_fee_calculator,
            current_point,
            amount_in,
            trade_direction,
            &fee_handler,
            &fee_mode,
        ) {
            // update swap amount
            swap_amount = amount_in;
            if let Ok(status) = validate_swap_result(
                &swap_result,
                token_a_transfer_fee_calculator,
                token_b_transfer_fee_calculator,
                remaining_amount,
                pool_amount_a,
                pool_amount_b,
                trade_direction,
            ) {
                match status {
                    SwapResultStatus::Done => {
                        #[cfg(test)]
                        println!("Done calculate swap result {}", _i);
                        break;
                    }
                    SwapResultStatus::ExceededA => {
                        if trade_direction == TradeDirection::AtoB {
                            // need to increase swap amount
                            min_swap_amount = swap_amount;
                        } else {
                            // need to decrease swap amount
                            max_swap_amount = swap_amount;
                        }
                    }
                    SwapResultStatus::ExceededB => {
                        if trade_direction == TradeDirection::AtoB {
                            // need to decrease swap amount
                            max_swap_amount = swap_amount;
                        } else {
                            // need to increase swap amount
                            min_swap_amount = swap_amount;
                        }
                    }
                }
            } else {
                #[cfg(test)]
                println!("can't validate swap result {}", _i);

                break; // if we can't validate swap result, then just break
            }
        } else {
            #[cfg(test)]
            println!("can't simulate swap result {}", _i);

            break; // if we can't simulate swap result, then just break
        }
    }

    Ok(swap_amount)
}

// Δa = L * (1 / √P_lower - 1 / √P_upper) => L = Δa / (1 / √P_lower - 1 / √P_upper)
pub fn get_liquidity_from_amount_a(
    amount_a: u64,
    sqrt_max_price: u128,
    sqrt_price: u128,
) -> Result<u128> {
    let price_delta = U512::from(sqrt_max_price.safe_sub(sqrt_price)?);
    let prod = U512::from(amount_a)
        .safe_mul(U512::from(sqrt_price))?
        .safe_mul(U512::from(sqrt_max_price))?;
    let liquidity = prod.safe_div(price_delta)?; // round down
    Ok(liquidity.try_into().map_err(|_| ZapError::TypeCastFailed)?)
}

// Δb = L (√P_upper - √P_lower) => L = Δb / (√P_upper - √P_lower)
pub fn get_liquidity_from_amount_b(
    amount_b: u64,
    sqrt_min_price: u128,
    sqrt_price: u128,
) -> Result<u128> {
    let price_delta = U256::from(sqrt_price.safe_sub(sqrt_min_price)?);
    let quote_amount = U256::from(amount_b).safe_shl(128)?;
    let liquidity = quote_amount.safe_div(price_delta)?; // round down
    return Ok(liquidity.try_into().map_err(|_| ZapError::TypeCastFailed)?);
}

// u32::MAX == 4_294_967_295, so we dont allow price change to go over 4_294_967_295 * 100 / 10_000 = 42_949_672 (%)
pub fn get_price_change_bps(pre_sqrt_price: u128, post_sqrt_price: u128) -> Result<u32> {
    let price_diff = if pre_sqrt_price > post_sqrt_price {
        pre_sqrt_price.safe_sub(post_sqrt_price)?
    } else {
        post_sqrt_price.safe_sub(pre_sqrt_price)?
    };

    let price_diff_prod = U192::from(price_diff).safe_mul(U192::from(MAX_BASIS_POINT))?;

    let price_diff_bps = price_diff_prod.div_ceil(U192::from(pre_sqrt_price));
    Ok(price_diff_bps
        .try_into()
        .map_err(|_| ZapError::TypeCastFailed)?)
}
