use crate::{
    damm_v2_ultils::{get_liquidity_from_amount_a, get_liquidity_from_amount_b},
    math::safe_math::SafeMath,
};
use anchor_lang::prelude::*;
use damm_v2::params::swap::TradeDirection;

#[account(zero_copy)]
#[derive(InitSpace, Debug, Default)]
pub struct UserLedger {
    pub owner: Pubkey,
    pub amount_a: u64,
    pub amount_b: u64,
}

impl UserLedger {
    pub fn update_ledger_balance(
        &mut self,
        pre_amount: u64,
        post_amount: u64,
        is_token_a: bool,
    ) -> Result<()> {
        if is_token_a {
            self.amount_a = self.amount_a.safe_add(post_amount)?.safe_sub(pre_amount)?;
        } else {
            self.amount_b = self.amount_b.safe_add(post_amount)?.safe_sub(pre_amount)?;
        }
        Ok(())
    }
    pub fn get_liquidity_from_amounts_and_trade_direction(
        &self,
        sqrt_price: u128,
        min_sqrt_price: u128,
        max_sqrt_price: u128,
    ) -> Result<(u128, TradeDirection)> {
        let liquidity_from_a =
            get_liquidity_from_amount_a(self.amount_a, max_sqrt_price, sqrt_price)?;
        let liquidity_from_b =
            get_liquidity_from_amount_b(self.amount_b, min_sqrt_price, sqrt_price)?;
        if liquidity_from_a > liquidity_from_b {
            // a is surplus, so we need to swap AtoB
            Ok((liquidity_from_b, TradeDirection::AtoB))
        } else {
            // b is surplus, so we need to swap BtoA
            Ok((liquidity_from_b, TradeDirection::BtoA))
        }
    }
}
