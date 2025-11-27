use anchor_lang::prelude::*;

use std::ops::Neg;

use damm_v2::safe_math::SafeMath;
use ruint::aliases::U256;

use crate::{error::ZapError, price_math::get_price_from_id};
#[derive(AnchorSerialize, AnchorDeserialize, Eq, PartialEq, Clone, Debug)]
pub enum StrategyType {
    // spot
    Spot,
    // curve
    Curve,
    // bidAsk
    BidAsk,
}

pub struct ParsedAddLiquidityParams {
    pub x0: i128,
    pub y0: i128,
    pub delta_x: i128,
    pub delta_y: i128,
}

pub struct UnparsedAddLiquidityParams {
    pub x0: u64,
    pub y0: u64,
    pub delta_x: u64,
    pub delta_y: u64,
    pub bit_flag: u8,
}

pub const X0_NEG_FLAG: u8 = 0b1;
pub const Y0_NEG_FLAG: u8 = 0b10;
pub const DELTA_X_NEG_FLAG: u8 = 0b100;
pub const DELTA_Y_NEG_FLAG: u8 = 0b1000;

impl ParsedAddLiquidityParams {
    pub fn unparse(&self) -> Result<UnparsedAddLiquidityParams> {
        let &ParsedAddLiquidityParams {
            x0,
            y0,
            delta_x,
            delta_y,
        } = self;

        let mut bit_flag = 0;

        if x0 < 0 {
            bit_flag |= X0_NEG_FLAG;
        }
        if y0 < 0 {
            bit_flag |= Y0_NEG_FLAG;
        }
        if delta_x < 0 {
            bit_flag |= DELTA_X_NEG_FLAG;
        }
        if delta_y < 0 {
            bit_flag |= DELTA_Y_NEG_FLAG;
        }

        Ok(UnparsedAddLiquidityParams {
            x0: u64::try_from(x0.abs()).map_err(|_| ZapError::TypeCastFailed)?,
            y0: u64::try_from(y0.abs()).map_err(|_| ZapError::TypeCastFailed)?,
            delta_x: u64::try_from(delta_x.abs()).map_err(|_| ZapError::TypeCastFailed)?,
            delta_y: u64::try_from(delta_y.abs()).map_err(|_| ZapError::TypeCastFailed)?,
            bit_flag,
        })
    }
}

pub struct ZapInRebalancingParams {
    pub amount_x: u64,
    pub amount_y: u64,
    pub min_delta_id: i32,
    pub max_delta_id: i32,
    pub strategy: StrategyType,
    pub favor_x_in_active_id: bool,
    pub bin_step: u16,
    pub active_id: i32,
}

impl ZapInRebalancingParams {
    pub fn get_rebalancing_params(&self) -> Result<UnparsedAddLiquidityParams> {
        let strategy_handler: Box<dyn StrategyHandler> = match self.strategy {
            StrategyType::Spot => Box::new(SpotHandler),
            StrategyType::Curve => Box::new(CurveHandler),
            StrategyType::BidAsk => Box::new(BidAskHandler),
        };
        let parsed_params = self.get_parsed_rebalancing_params(&strategy_handler)?;
        parsed_params.unparse()
    }

    pub fn get_max_delta_id_both_side(&self) -> (i32, i32) {
        let (bid_side_end_delta_id, ask_side_start_delta_id) = if self.favor_x_in_active_id {
            (-1, 0)
        } else {
            (0, 1)
        };
        (bid_side_end_delta_id, ask_side_start_delta_id)
    }

    pub fn is_only_deposit_y(&self) -> bool {
        let (bid_side_end_delta_id, _ask_side_start_delta_id) = self.get_max_delta_id_both_side();
        self.max_delta_id <= bid_side_end_delta_id
    }

    pub fn is_only_deposit_x(&self) -> bool {
        let (_bid_side_end_delta_id, ask_side_start_delta_id) = self.get_max_delta_id_both_side();
        self.min_delta_id >= ask_side_start_delta_id
    }

    pub fn get_parsed_rebalancing_params(
        &self,
        strategy_handler: &Box<dyn StrategyHandler>,
    ) -> Result<ParsedAddLiquidityParams> {
        // only deposit y
        if self.is_only_deposit_y() {
            let (y0, delta_y) = strategy_handler.find_y0_and_delta_y(
                self.amount_y,
                self.min_delta_id,
                self.max_delta_id,
            )?;
            return Ok(ParsedAddLiquidityParams {
                x0: 0,
                y0,
                delta_x: 0,
                delta_y,
            });
        }
        // only deposit x
        if self.is_only_deposit_x() {
            let (x0, delta_x) = strategy_handler.find_x0_and_delta_x(
                self.amount_x,
                self.min_delta_id,
                self.max_delta_id,
                self.bin_step,
                self.active_id,
            )?;
            return Ok(ParsedAddLiquidityParams {
                x0,
                y0: 0,
                delta_x,
                delta_y: 0,
            });
        }
        // deposit both x and y,
        let (bid_side_end_delta_id, ask_side_start_delta_id) = self.get_max_delta_id_both_side();
        let (y0, delta_y) = strategy_handler.find_y0_and_delta_y(
            self.amount_y,
            self.min_delta_id,
            bid_side_end_delta_id,
        )?;
        let (x0, delta_x) = strategy_handler.find_x0_and_delta_x(
            self.amount_x,
            ask_side_start_delta_id,
            self.max_delta_id,
            self.bin_step,
            self.active_id,
        )?;
        Ok(ParsedAddLiquidityParams {
            x0,
            y0,
            delta_x,
            delta_y,
        })
    }
}

pub trait StrategyHandler {
    fn find_y0_and_delta_y(
        &self,
        amount_y: u64,
        min_delta_id: i32,
        max_delta_id: i32,
    ) -> Result<(i128, i128)>;

    fn find_x0_and_delta_x(
        &self,
        amount_x: u64,
        min_delta_id: i32,
        max_delta_id: i32,
        bin_step: u16,
        active_id: i32,
    ) -> Result<(i128, i128)>;
}

struct SpotHandler;
struct CurveHandler;
struct BidAskHandler;
impl StrategyHandler for SpotHandler {
    fn find_y0_and_delta_y(
        &self,
        amount_y: u64,
        min_delta_id: i32,
        max_delta_id: i32,
    ) -> Result<(i128, i128)> {
        let total_bin = max_delta_id.safe_sub(min_delta_id)?.safe_add(1)?;
        let y0 =
            amount_y.safe_div(u64::try_from(total_bin).map_err(|_| ZapError::TypeCastFailed)?)?;
        Ok((y0.into(), 0))
    }
    // in spot delta_x == 0, so
    // active_id + min_delta_id = x0 * (1+b)^-(active_id + min_delta_id)
    // ...
    // active_id + max_delta_id = x0 * (1+b)^-(active_id + max_delta_id)
    fn find_x0_and_delta_x(
        &self,
        amount_x: u64,
        min_delta_id: i32,
        max_delta_id: i32,
        bin_step: u16,
        active_id: i32,
    ) -> Result<(i128, i128)> {
        if amount_x == 0 || max_delta_id <= 0 {
            return Ok((0, 0));
        }
        let mut total_weight = U256::ZERO;
        let min_bin_id = active_id.safe_add(min_delta_id)?;
        let max_bin_id = active_id.safe_add(max_delta_id)?;

        for bin_id in min_bin_id..=max_bin_id {
            let base_price = U256::from(get_price_from_id(bin_id.neg(), bin_step)?);
            total_weight = total_weight.safe_add(base_price)?;
        }
        let amount_x = U256::from(amount_x);
        let x0 = amount_x.safe_shl(64)?.safe_div(total_weight)?;
        let x0 = i128::try_from(x0).map_err(|_| ZapError::TypeCastFailed)?;

        Ok((x0, 0))
    }
}

impl StrategyHandler for CurveHandler {
    fn find_y0_and_delta_y(
        &self,
        amount_y: u64,
        min_delta_id: i32,
        max_delta_id: i32,
    ) -> Result<(i128, i128)> {
        // min_delta_id = -m1, max_delta_id = -m2
        //
        // active_id - m2 = y0 + delta_y * m2
        // active_id - (m2 + 1) = y0 + delta_y * (m2-1)
        // ...
        // active_id - m1 = y0 + delta_y * m1
        //
        // sum(amounts) = y0 * (m1-m2+1) + delta_y * (m1 * (m1+1)/2 - m2 * (m2-1)/2)
        // set delta_y = -y0 / m1
        // sum(amounts) = y0 * (m1-m2+1) - y0 * (m1 * (m1+1)/2 - m2 * (m2-1)/2) / m1
        // A = (m1-m2+1) - (m1 * (m1+1)/2 - m2 * (m2-1)/2) / m1
        // y0 = sum(amounts) / A
        // avoid precision loss:
        // y0 = sum(amounts) * m1 / ((m1-m2+1) * m1 - (m1 * (m1+1)/2 - m2 * (m2-1)/2))
        // noted: y0 > 0 and delta_y < 0 in curve strategy

        // min_delta_id and min_delta_id <= 0
        if min_delta_id == max_delta_id {
            // quick return
            return Ok((amount_y.into(), 0));
        }

        let m1: i128 = min_delta_id.neg().into();
        let m2: i128 = max_delta_id.neg().into();

        let a = (m1 - m2 + 1) * m1 - (m1 * (m1 + 1) / 2 - m2 * (m2 - 1) / 2);
        let y0 = i128::from(amount_y) * m1 / a;
        // we round down delta_y firstly
        // m1 can't be zero because we've checked for min_delta_id  <= max_delta_id, and both delta id is smaller than or equal 0
        let delta_y = -(y0 / m1);

        // then we update y0 to ensure the first amount (active_id - m1 = y0 + delta_y * m1) > 0
        // delta_y is negative and round up, while y0 is positive and round down
        // it will ensure sum(amounts) <= amount_y
        // sum(amounts) = y0 * (m1-m2+1) + delta_y * (m1 * (m1+1)/2 - m2 * (m2-1)/2)
        // sum(amounts) = -(delta_y * m1) * (m1-m2+1) + delta_y * (m1 * (m1+1)/2 - m2 * (m2-1)/2)
        let y0 = -(delta_y * m1);

        Ok((y0, delta_y))
    }

    fn find_x0_and_delta_x(
        &self,
        amount_x: u64,
        min_delta_id: i32,
        max_delta_id: i32,
        bin_step: u16,
        active_id: i32,
    ) -> Result<(i128, i128)> {
        // min_delta_id = m1, max_delta_id = m2
        // pm = (1+b)^-(active_id + m)
        //
        // active_id + m1 = (x0 + m1 * delta_x) * p(m1)
        // active_id + m1 + 1 = (x0 + (m1 + 1) * delta_x) * p(m1+1)
        // ...
        // active_id + m2 =  (x0 + m2 * delta_x) * p(m2)
        //
        // sum(amounts) = x0 * (p(m1)+..+p(m2)) + delta_x * (m1 * p(m1) + ... + m2 * p(m2))
        // set delta_x = -x0 / m2

        // sum(amounts) = x0 * (p(m1)+..+p(m2)) - x0 * (m1 * p(m1) + ... + m2 * p(m2)) / m2
        // A = (p(m1)+..+p(m2)) - (m1 * p(m1) + ... + m2 * p(m2)) / m2
        // B = (p(m1)+..+p(m2))
        // C = (m1 * p(m1) + ... + m2 * p(m2)) / m2
        // x0 = sum(amounts) / (B-C)
        // noted: x0 > 0 and delta_x < 0 in curve strategy

        let mut b = U256::ZERO;
        let mut c = U256::ZERO;
        let m1 = min_delta_id;
        let m2 = max_delta_id;

        for m in m1..=m2 {
            let bin_id = active_id.safe_add(m)?;
            let pm = U256::from(get_price_from_id(bin_id.neg(), bin_step)?);

            b = b.safe_add(pm)?;

            let c_delta = U256::from(m).safe_mul(pm)?.safe_div(U256::from(m2))?;

            c = c.safe_add(c_delta)?;
        }

        let x0 = U256::from(amount_x)
            .safe_shl(64)?
            .safe_div(b.safe_sub(c)?)?;
        let x0: i128 = x0.try_into().map_err(|_| ZapError::TypeCastFailed)?;
        let m2: i128 = max_delta_id.into();
        let delta_x = if m2 != 0 { -x0 / m2 } else { 0 };

        // same handle as get y0, delta_y
        let x0 = -(delta_x * m2);

        Ok((x0, delta_x))
    }
}

impl StrategyHandler for BidAskHandler {
    fn find_y0_and_delta_y(
        &self,
        amount_y: u64,
        min_delta_id: i32,
        max_delta_id: i32,
    ) -> Result<(i128, i128)> {
        // min_delta_id = -m1, max_delta_id = -m2
        //
        // active_id - m2 = y0 + delta_y * m2
        // active_id - (m2 + 1) = y0 + delta_y * (m2-1)
        // ...
        // active_id - m1 = y0 + delta_y * m1
        //
        // sum(amounts) = y0 * (m1-m2+1) + delta_y * (m1 * (m1+1)/2 - m2 * (m2-1)/2)
        // set y0 = -delta_y * m2
        // sum(amounts) = -delta_y * m2 * (m1-m2+1) + delta_y * (m1 * (m1+1)/2 - m2 * (m2-1)/2)
        // A = -m2 * (m1-m2+1) + (m1 * (m1+1)/2 - m2 * (m2-1)/2)
        // delta_y = sum(amounts) / A
        // note: in bid ask strategy: y0 < 0 and delta_y > 0

        if min_delta_id == max_delta_id {
            return Ok((amount_y.into(), 0));
        }
        let m1: i128 = min_delta_id.neg().into();
        let m2: i128 = max_delta_id.neg().into();

        let a = -m2 * (m1 - m2 + 1) + (m1 * (m1 + 1) / 2 - m2 * (m2 - 1) / 2);
        let delta_y = i128::from(amount_y) / a;
        let y0 = delta_y.neg() * m2;
        Ok((y0, delta_y))
    }

    fn find_x0_and_delta_x(
        &self,
        amount_x: u64,
        min_delta_id: i32,
        max_delta_id: i32,
        bin_step: u16,
        active_id: i32,
    ) -> Result<(i128, i128)> {
        // min_delta_id = m1, max_delta_id = m2
        // pm = (1+b)^-(active_id + m)
        //
        // active_id + m1 = (x0 + m1 * delta_x) * p(m1)
        // active_id + m1 + 1 = (x0 + (m1 + 1) * delta_x) * p(m1+1)
        // ...
        // active_id + m2 =  (x0 + m2 * delta_x) * p(m2)
        //
        // sum(amounts) = x0 * (p(m1)+..+p(m2)) + delta_x * (m1 * p(m1) + ... + m2 * p(m2))
        // set x0 = -m1 * delta_x

        // sum(amounts) = -m1 * delta_x * (p(m1)+..+p(m2)) + delta_x * (m1 * p(m1) + ... + m2 * p(m2))
        // A = -m1 * (p(m1)+..+p(m2)) + (m1 * p(m1) + ... + m2 * p(m2))
        // B = m1 * (p(m1)+..+p(m2))
        // C = (m1 * p(m1) + ... + m2 * p(m2))
        // delta_x = sum(amounts) / (C-B)
        // note: in bid ask strategy: x0 < 0 and delta_x > 0

        let mut b = U256::ZERO;
        let mut c = U256::ZERO;
        let m1 = U256::try_from(min_delta_id).map_err(|_| ZapError::TypeCastFailed)?;

        for m in min_delta_id..=max_delta_id {
            let bin_id = active_id.safe_add(m)?;
            let pm = U256::from(get_price_from_id(bin_id.neg(), bin_step)?);

            let b_delta = m1.safe_mul(pm)?;

            b = b.safe_add(b_delta)?;

            let c_delta = U256::try_from(m)
                .map_err(|_| ZapError::TypeCastFailed)?
                .safe_mul(pm)?;

            c = c.safe_add(c_delta)?;
        }

        //reverse b to c
        let delta_x = U256::from(amount_x)
            .safe_shl(64)?
            .safe_div(c.safe_sub(b)?)?;
        let delta_x: i128 = delta_x.try_into().map_err(|_| ZapError::TypeCastFailed)?;
        let x0 = delta_x * i128::from(min_delta_id).neg();
        Ok((x0, delta_x))
    }
}
