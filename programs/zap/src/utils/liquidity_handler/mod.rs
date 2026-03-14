pub mod compounding_liquidity;
pub use compounding_liquidity::*;

pub mod concentrated_liquidity;
pub use concentrated_liquidity::*;

use anchor_lang::prelude::*;
use damm_v2::state::{CollectFeeMode, Pool};

use crate::error::ZapError;

pub struct SwapAmountFromInput {
    pub output_amount: u64,
}

pub trait LiquidityHandler {
    fn calculate_a_to_b_from_amount_in(&self, amount_in: u64) -> Result<SwapAmountFromInput>;
    fn calculate_b_to_a_from_amount_in(&self, amount_in: u64) -> Result<SwapAmountFromInput>;
    fn get_liquidity_delta_from_amount_a(&self, amount_a: u64) -> Result<u128>;
    fn get_liquidity_delta_from_amount_b(&self, amount_b: u64) -> Result<u128>;
}

pub fn get_liquidity_handler(pool: &Pool) -> Result<Box<dyn LiquidityHandler>> {
    let collect_fee_mode = CollectFeeMode::try_from(pool.collect_fee_mode)
        .map_err(|_| ZapError::UnsupportedFeeMode)?;
    if collect_fee_mode == CollectFeeMode::Compounding {
        Ok(Box::new(CompoundingLiquidity {
            token_a_amount: pool.token_a_amount,
            token_b_amount: pool.token_b_amount,
            liquidity: pool.liquidity,
        }))
    } else {
        Ok(Box::new(ConcentratedLiquidity {
            sqrt_price: pool.sqrt_price,
            sqrt_min_price: pool.sqrt_min_price,
            sqrt_max_price: pool.sqrt_max_price,
            liquidity: pool.liquidity,
        }))
    }
}
