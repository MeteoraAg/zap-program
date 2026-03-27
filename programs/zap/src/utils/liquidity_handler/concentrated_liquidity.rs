use anchor_lang::prelude::*;
use damm_v2::{
    get_delta_amount_a_unsigned, get_delta_amount_b_unsigned, get_next_sqrt_price_from_input,
    u128x128_math::Rounding, PoolError,
};
use ruint::aliases::{U256, U512};

use crate::{error::ZapError, safe_math::SafeMath, LiquidityHandler, SwapAmountFromInput};

pub struct ConcentratedLiquidity {
    pub sqrt_price: u128,
    pub sqrt_min_price: u128,
    pub sqrt_max_price: u128,
    pub liquidity: u128,
}

impl LiquidityHandler for ConcentratedLiquidity {
    // ref: https://github.com/MeteoraAg/damm-v2/blob/6e9aee4e549bd792c8f5b82b88be04459e644f3c/programs/cp-amm/src/liquidity_handler/concentrated_liquidity.rs#L66-L87
    fn calculate_a_to_b_from_amount_in(&self, amount_in: u64) -> Result<SwapAmountFromInput> {
        let next_sqrt_price =
            get_next_sqrt_price_from_input(self.sqrt_price, self.liquidity, amount_in, true)?;

        if next_sqrt_price < self.sqrt_min_price {
            return Err(PoolError::PriceRangeViolation.into());
        }

        let output_amount = get_delta_amount_b_unsigned(
            next_sqrt_price,
            self.sqrt_price,
            self.liquidity,
            Rounding::Down,
        )?;

        Ok(SwapAmountFromInput { output_amount })
    }

    // ref: https://github.com/MeteoraAg/damm-v2/blob/6e9aee4e549bd792c8f5b82b88be04459e644f3c/programs/cp-amm/src/liquidity_handler/concentrated_liquidity.rs#L90-L110
    fn calculate_b_to_a_from_amount_in(&self, amount_in: u64) -> Result<SwapAmountFromInput> {
        let next_sqrt_price =
            get_next_sqrt_price_from_input(self.sqrt_price, self.liquidity, amount_in, false)?;

        if next_sqrt_price > self.sqrt_max_price {
            return Err(PoolError::PriceRangeViolation.into());
        }

        let output_amount = get_delta_amount_a_unsigned(
            self.sqrt_price,
            next_sqrt_price,
            self.liquidity,
            Rounding::Down,
        )?;

        Ok(SwapAmountFromInput { output_amount })
    }

    // ref: https://github.com/MeteoraAg/damm-v2/blob/6e9aee4e549bd792c8f5b82b88be04459e644f3c/rust-sdk/src/tests/test_calculate_concentrated_initial_sqrt_price.rs#L15-L28
    // liquidity delta has the same calculation as initial liquidity for concentrated liquidity
    // Δa = L * (1/√P_lower - 1/√P_upper) => L = Δa / (1/√P_lower - 1/√P_upper)
    fn get_liquidity_delta_from_amount_a(&self, amount_a: u64) -> Result<u128> {
        if self.sqrt_price == self.sqrt_max_price {
            // Single-sided B position: no token A needed, return max so A is always surplus
            return Ok(u128::MAX);
        }
        let price_delta = U512::from(self.sqrt_max_price.safe_sub(self.sqrt_price)?);
        let prod = U512::from(amount_a)
            .safe_mul(U512::from(self.sqrt_price))?
            .safe_mul(U512::from(self.sqrt_max_price))?;
        let liquidity = prod.safe_div(price_delta)?;
        Ok(liquidity.try_into().map_err(|_| ZapError::TypeCastFailed)?)
    }

    // ref: https://github.com/MeteoraAg/damm-v2/blob/6e9aee4e549bd792c8f5b82b88be04459e644f3c/rust-sdk/src/tests/test_calculate_concentrated_initial_sqrt_price.rs#L31-L42
    // liquidity delta has the same calculation as initial liquidity for concentrated liquidity
    // Δb = L * (√P_upper - √P_lower) => L = Δb / (√P_upper - √P_lower)
    fn get_liquidity_delta_from_amount_b(&self, amount_b: u64) -> Result<u128> {
        if self.sqrt_price == self.sqrt_min_price {
            // Single-sided A position: no token B needed, return max so B is always surplus
            return Ok(u128::MAX);
        }
        let price_delta = U256::from(self.sqrt_price.safe_sub(self.sqrt_min_price)?);
        let quote_amount = U256::from(amount_b).safe_shl(128)?;
        let liquidity = quote_amount.safe_div(price_delta)?;
        Ok(liquidity.try_into().map_err(|_| ZapError::TypeCastFailed)?)
    }
}
