use anchor_lang::prelude::*;
use damm_v2::{
    base_fee::{
        fee_rate_limiter::PodAlignedFeeRateLimiter, BaseFeeEnumReader, BaseFeeHandler,
        BaseFeeHandlerBuilder,
    },
    constants::fee::get_max_fee_numerator,
    params::swap::TradeDirection,
    state::{
        fee::{BaseFeeMode, FeeMode},
        CollectFeeMode, Pool,
    },
};
use ruint::aliases::U192;

use crate::{
    constants::MAX_BASIS_POINT, error::ZapError, get_liquidity_handler, safe_math::SafeMath,
    LiquidityHandler, SwapAmountFromInput, TransferFeeCalculator,
};

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

struct SimulateSwapResult {
    user_amount_in: u64,
    user_amount_out: u64,
    pool_amount_in: u64,
    pool_amount_out: u64,
    compounding_fee: u64,
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
    handler: &dyn LiquidityHandler,
) -> Result<SimulateSwapResult> {
    let (input_transfer_fee_calculator, output_transfer_fee_calculator) =
        if trade_direction == TradeDirection::AtoB {
            (
                token_a_transfer_fee_calculator,
                token_b_transfer_fee_calculator,
            )
        } else {
            (
                token_b_transfer_fee_calculator,
                token_a_transfer_fee_calculator,
            )
        };

    let excluded_fee_amount_in = input_transfer_fee_calculator
        .calculate_transfer_fee_excluded_amount(amount_in)?
        .amount;

    let trade_fee_numerator = fee_handler.get_trade_fee_numerator(
        excluded_fee_amount_in,
        current_point,
        pool.activation_point,
        trade_direction,
        pool.pool_fees.init_sqrt_price,
        pool.sqrt_price,
    )?;

    let (actual_amount_in, input_compounding_fee) = if fee_mode.fees_on_input {
        let fee_result = pool.pool_fees.get_fee_on_amount(
            excluded_fee_amount_in,
            trade_fee_numerator,
            fee_mode.has_referral,
        )?;
        (fee_result.amount, fee_result.compounding_fee)
    } else {
        (excluded_fee_amount_in, 0)
    };

    let SwapAmountFromInput { output_amount, .. } = match trade_direction {
        TradeDirection::AtoB => handler.calculate_a_to_b_from_amount_in(actual_amount_in),
        TradeDirection::BtoA => handler.calculate_b_to_a_from_amount_in(actual_amount_in),
    }?;

    let (actual_amount_out, output_compounding_fee) = if fee_mode.fees_on_input {
        (output_amount, 0)
    } else {
        let fee_result = pool.pool_fees.get_fee_on_amount(
            output_amount,
            trade_fee_numerator,
            fee_mode.has_referral,
        )?;
        (fee_result.amount, fee_result.compounding_fee)
    };

    let compounding_fee = input_compounding_fee.safe_add(output_compounding_fee)?;

    let excluded_fee_amount_out = output_transfer_fee_calculator
        .calculate_transfer_fee_excluded_amount(actual_amount_out)?
        .amount;

    Ok(SimulateSwapResult {
        user_amount_in: excluded_fee_amount_out,
        user_amount_out: amount_in,
        pool_amount_in: actual_amount_in,
        pool_amount_out: output_amount,
        compounding_fee,
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
        compounding_fee,
    } = swap_result;
    // apply swap result
    if trade_direction == TradeDirection::AtoB {
        let user_amount_a = remaining_amount.safe_sub(user_amount_out)?;
        let user_amount_b = user_amount_in;
        let pool_amount_a = total_amount_a.safe_add(pool_amount_in)?;
        let pool_amount_b = total_amount_b
            .safe_sub(pool_amount_out)?
            .safe_add(compounding_fee)?;
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
        let pool_amount_b = total_amount_b
            .safe_add(pool_amount_in)?
            .safe_add(compounding_fee)?;
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
    pub rate_limiter_handler: PodAlignedFeeRateLimiter, // avoid copy
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
        init_sqrt_price: u128,
        current_sqrt_price: u128,
    ) -> Result<u64> {
        if self.is_rate_limiter {
            let base_fee_numerator = self
                .rate_limiter_handler
                .get_base_fee_numerator_from_included_fee_amount(
                    current_point,
                    activation_point,
                    trade_direction,
                    input_amount,
                    init_sqrt_price,
                    current_sqrt_price,
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
    let max_fee_numerator = get_max_fee_numerator(pool.fee_version)?;

    let base_fee_mode = pool.pool_fees.base_fee.base_fee_info.get_base_fee_mode()?;
    match BaseFeeMode::try_from(base_fee_mode) {
        Ok(value) => {
            match value {
                BaseFeeMode::FeeTimeSchedulerLinear
                | BaseFeeMode::FeeTimeSchedulerExponential
                | BaseFeeMode::FeeMarketCapSchedulerLinear
                | BaseFeeMode::FeeMarketCapSchedulerExponential => {
                    let base_fee_handler = pool
                        .pool_fees
                        .base_fee
                        .base_fee_info
                        .get_base_fee_handler()?;
                    // fee scheduler doesn't care for amount
                    let base_fee_numerator = base_fee_handler
                        .get_base_fee_numerator_from_included_fee_amount(
                            current_point,
                            pool.activation_point,
                            trade_direction,
                            0,
                            pool.pool_fees.init_sqrt_price,
                            pool.sqrt_price,
                        )?;

                    let total_fee_numerator = get_total_fee_numerator(
                        base_fee_numerator,
                        variable_fee_numerator,
                        max_fee_numerator,
                    )?;
                    Ok(FeeHandler {
                        rate_limiter_handler: PodAlignedFeeRateLimiter::default(),
                        variable_fee_numerator,
                        max_fee_numerator,
                        total_fee_numerator,
                        is_rate_limiter: false,
                    })
                }
                BaseFeeMode::RateLimiter => {
                    let rate_limiter_handler = pool.pool_fees.base_fee.to_fee_rate_limiter()?;
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
        _ => Err(ZapError::UnsupportedFeeMode.into()),
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
) -> Result<(u64, u64)> {
    let mut max_swap_amount = remaining_amount;
    let mut min_swap_amount = 0;
    let mut swap_in_amount = 0;
    let mut swap_out_amount = 0;

    let fee_handler = get_fee_handler(pool, current_point, trade_direction)?;

    let collect_fee_mode = CollectFeeMode::try_from(pool.collect_fee_mode)
        .map_err(|_| ZapError::UnsupportedFeeMode)?;
    let fee_mode = FeeMode::get_fee_mode(collect_fee_mode, trade_direction, false);

    let (pool_amount_a, pool_amount_b) = pool.get_liquidity_handler()?.get_reserves_amount()?;

    let handler = get_liquidity_handler(pool)?;

    // max 20 loops
    // For each loop program consumed ~ 5394.3 -> 5,395 CUs
    // So the 20 loops will consume maximum ~ 107,900 CUs
    for _i in 0..20 {
        let delta_half = max_swap_amount.safe_sub(min_swap_amount)? >> 1;
        let amount_in = min_swap_amount.safe_add(delta_half)?;

        if amount_in == swap_in_amount {
            break;
        }

        let swap_result = calculate_swap_result(
            pool,
            token_a_transfer_fee_calculator,
            token_b_transfer_fee_calculator,
            current_point,
            amount_in,
            trade_direction,
            &fee_handler,
            &fee_mode,
            handler.as_ref(),
        )?;

        // update swap amount
        swap_in_amount = amount_in;
        swap_out_amount = swap_result.user_amount_in;

        let status = validate_swap_result(
            &swap_result,
            token_a_transfer_fee_calculator,
            token_b_transfer_fee_calculator,
            remaining_amount,
            pool_amount_a,
            pool_amount_b,
            trade_direction,
        )?;

        match status {
            SwapResultStatus::Done => {
                #[cfg(test)]
                println!("Done calculate swap result {}", _i);
                break;
            }
            SwapResultStatus::ExceededA => {
                if trade_direction == TradeDirection::AtoB {
                    // need to increase swap amount
                    min_swap_amount = swap_in_amount;
                } else {
                    // need to decrease swap amount
                    max_swap_amount = swap_in_amount;
                }
            }
            SwapResultStatus::ExceededB => {
                if trade_direction == TradeDirection::AtoB {
                    // need to decrease swap amount
                    max_swap_amount = swap_in_amount;
                } else {
                    // need to increase swap amount
                    min_swap_amount = swap_in_amount;
                }
            }
        }
    }

    Ok((swap_in_amount, swap_out_amount))
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
