use anchor_lang::prelude::*;
use damm_v2::u128x128_math::Rounding;

use crate::{
    math::utils_math::{safe_mul_div_cast_u128, safe_mul_div_cast_u64},
    safe_math::SafeMath,
};

use super::{LiquidityHandler, SwapAmountFromInput};

pub struct CompoundingLiquidity {
    pub token_a_amount: u64,
    pub token_b_amount: u64,
    pub liquidity: u128,
}

impl LiquidityHandler for CompoundingLiquidity {
    // ref: https://github.com/MeteoraAg/damm-v2/blob/6e9aee4e549bd792c8f5b82b88be04459e644f3c/programs/cp-amm/src/liquidity_handler/compounding_liquidity.rs#L67-L82
    // a * b = (a + amount_in) * (b - output_amount)
    // => output_amount = b - a * b / (a + amount_in) = b * amount_in / (a + amount_in)
    fn calculate_a_to_b_from_amount_in(&self, amount_in: u64) -> Result<SwapAmountFromInput> {
        let output_amount = safe_mul_div_cast_u64(
            self.token_b_amount,
            amount_in,
            self.token_a_amount.safe_add(amount_in)?,
            Rounding::Down,
        )?;
        Ok(SwapAmountFromInput { output_amount })
    }

    // ref: https://github.com/MeteoraAg/damm-v2/blob/6e9aee4e549bd792c8f5b82b88be04459e644f3c/programs/cp-amm/src/liquidity_handler/compounding_liquidity.rs#L84-L99
    // a * b = (b + amount_in) * (a - output_amount)
    // => output_amount = a - a * b / (b + amount_in) = a * amount_in / (b + amount_in)
    fn calculate_b_to_a_from_amount_in(&self, amount_in: u64) -> Result<SwapAmountFromInput> {
        let output_amount = safe_mul_div_cast_u64(
            self.token_a_amount,
            amount_in,
            self.token_b_amount.safe_add(amount_in)?,
            Rounding::Down,
        )?;
        Ok(SwapAmountFromInput { output_amount })
    }

    // inverse formula of get_amounts_for_modify_liquidity
    // ref: https://github.com/MeteoraAg/damm-v2/blob/6e9aee4e549bd792c8f5b82b88be04459e644f3c/programs/cp-amm/src/liquidity_handler/compounding_liquidity.rs#L46-L65
    // liquidity_delta / pool_liquidity = amount_a / pool_reserve_a
    // => liquidity_delta = amount_a * pool_liquidity / pool_reserve_a
    fn get_liquidity_delta_from_amount_a(&self, amount_a: u64) -> Result<u128> {
        safe_mul_div_cast_u128(
            amount_a.into(),
            self.liquidity,
            self.token_a_amount.into(),
            Rounding::Down,
        )
    }

    // inverse formula of get_amounts_for_modify_liquidity
    // ref: https://github.com/MeteoraAg/damm-v2/blob/6e9aee4e549bd792c8f5b82b88be04459e644f3c/programs/cp-amm/src/liquidity_handler/compounding_liquidity.rs#L46-L65
    fn get_liquidity_delta_from_amount_b(&self, amount_b: u64) -> Result<u128> {
        // liquidity_delta / pool_liquidity = amount_b / pool_reserve_b
        // => liquidity_delta = amount_b * pool_liquidity / pool_reserve_b
        safe_mul_div_cast_u128(
            amount_b.into(),
            self.liquidity,
            self.token_b_amount.into(),
            Rounding::Down,
        )
    }
}
