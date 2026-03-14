use crate::{
    error::ZapError, liquidity_handler::LiquidityHandler, math::safe_math::SafeMath,
    TransferFeeCalculator,
};
use anchor_lang::prelude::*;
use damm_v2::params::swap::TradeDirection;

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
        self.amount_a = u128::from(self.amount_a)
            .safe_add(post_amount_a.into())?
            .safe_sub(pre_amount_a.into())?
            .try_into()
            .map_err(|_| ZapError::MathOverflow)?;

        self.amount_b = u128::from(self.amount_b)
            .safe_add(post_amount_b.into())?
            .safe_sub(pre_amount_b.into())?
            .try_into()
            .map_err(|_| ZapError::MathOverflow)?;
        Ok(())
    }
    // only needed for damm v2 function
    pub fn get_liquidity_from_amounts_and_trade_direction(
        &self,
        token_a_transfer_fee_calculator: &TransferFeeCalculator,
        token_b_transfer_fee_calculator: &TransferFeeCalculator,
        handler: &dyn LiquidityHandler,
    ) -> Result<(u128, TradeDirection)> {
        let amount_a = token_a_transfer_fee_calculator
            .calculate_transfer_fee_excluded_amount(self.amount_a)?
            .amount;
        let amount_b = token_b_transfer_fee_calculator
            .calculate_transfer_fee_excluded_amount(self.amount_b)?
            .amount;
        let liquidity_from_a = handler.get_liquidity_delta_from_amount_a(amount_a)?;
        let liquidity_from_b = handler.get_liquidity_delta_from_amount_b(amount_b)?;
        if liquidity_from_a > liquidity_from_b {
            // a is surplus, so we need to swap AtoB
            Ok((liquidity_from_b, TradeDirection::AtoB))
        } else {
            // b is surplus, so we need to swap BtoA
            Ok((liquidity_from_a, TradeDirection::BtoA))
        }
    }
}
