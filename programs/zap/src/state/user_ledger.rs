use crate::{
    damm_v2_ultils::{get_liquidity_from_amount_a, get_liquidity_from_amount_b},
    math::safe_math::SafeMath,
};
use anchor_lang::prelude::*;
use anchor_spl::token_interface::Mint;
use damm_v2::{params::swap::TradeDirection, token::calculate_transfer_fee_excluded_amount};

#[account(zero_copy)]
#[derive(InitSpace, Debug, Default)]
pub struct UserLedger {
    pub owner: Pubkey,
    pub amount_a: u64, // amount_x in DLMM
    pub amount_b: u64, // amount_y in DLMM
}

impl UserLedger {
    pub fn update_ledger_balances(
        &mut self,
        pre_amount_a: u64,
        post_amount_a: u64,
        pre_amount_b: u64,
        post_amount_b: u64,
    ) -> Result<()> {
        self.amount_a = self
            .amount_a
            .safe_add(post_amount_a)?
            .safe_sub(pre_amount_a)?;
        self.amount_b = self
            .amount_b
            .safe_add(post_amount_b)?
            .safe_sub(pre_amount_b)?;
        Ok(())
    }
    // only needed for damm v2 function
    pub fn get_liquidity_from_amounts_and_trade_direction<'info>(
        &self,
        token_a_mint: &InterfaceAccount<'info, Mint>,
        token_b_mint: &InterfaceAccount<'info, Mint>,
        sqrt_price: u128,
        min_sqrt_price: u128,
        max_sqrt_price: u128,
    ) -> Result<(u128, TradeDirection)> {
        let amount_a = calculate_transfer_fee_excluded_amount(token_a_mint, self.amount_a)?.amount;
        let amount_b = calculate_transfer_fee_excluded_amount(token_b_mint, self.amount_b)?.amount;
        let liquidity_from_a = get_liquidity_from_amount_a(amount_a, max_sqrt_price, sqrt_price)?;
        let liquidity_from_b = get_liquidity_from_amount_b(amount_b, min_sqrt_price, sqrt_price)?;
        if liquidity_from_a > liquidity_from_b {
            // a is surplus, so we need to swap AtoB
            Ok((liquidity_from_b, TradeDirection::AtoB))
        } else {
            // b is surplus, so we need to swap BtoA
            Ok((liquidity_from_b, TradeDirection::BtoA))
        }
    }
}
