use anchor_lang::prelude::*;
use dlmm::types::{AddLiquidityParams as DlmmAddLiquidityParams, RebalanceLiquidityParams};
use ruint::aliases::U256;
use std::ops::Neg;

use crate::{
    error::ZapError,
    price_math::{get_price_base_factor, get_price_from_id, pow},
    safe_math::SafeMath,
    StrategyType, UnparsedAddLiquidityParams, ZapInRebalancingParams,
};

pub fn build_add_liquidity_params(
    amount_x: u64,
    amount_y: u64,
    active_id: i32,
    bin_step: u16,
    min_delta_id: i32,
    max_delta_id: i32,
    favor_x_in_active_id: bool,
    strategy: StrategyType,
) -> RebalanceLiquidityParams {
    let params = ZapInRebalancingParams {
        amount_x,
        amount_y,
        active_id,
        bin_step,
        min_delta_id,
        max_delta_id,
        favor_x_in_active_id,
        strategy,
    };

    let UnparsedAddLiquidityParams {
        x0,
        y0,
        delta_x,
        delta_y,
        bit_flag,
    } = params.get_rebalancing_params().unwrap();

    // bid side
    let params = RebalanceLiquidityParams {
        active_id,
        max_active_bin_slippage: 0,
        should_claim_fee: false,
        should_claim_reward: false,
        min_withdraw_x_amount: 0,
        max_deposit_x_amount: 0,
        min_withdraw_y_amount: 0,
        max_deposit_y_amount: 0,
        shrink_mode: 3, // we dont allow to shrink in both side
        padding: [0; 31],
        removes: vec![],
        adds: vec![DlmmAddLiquidityParams {
            min_delta_id,
            max_delta_id,
            x0,
            y0,
            delta_x,
            delta_y,
            favor_x_in_active_id,
            bit_flag,
            ..Default::default()
        }],
    };
    params
}
#[derive(Debug, Clone, Copy, Default)]
pub struct AmountInBin {
    pub bin_id: i32,
    pub amount_x: u64,
    pub amount_y: u64,
}

pub fn get_liquidity_distribution(amount_in_bins: &[AmountInBin], bin_step: u16) -> Vec<u64> {
    let mut liquidity_distributions = vec![];
    for bin in amount_in_bins.iter() {
        let price = U256::from(get_price_from_id(bin.bin_id, bin_step).unwrap());
        let (quote_amount_of_base_token, _) = U256::from(bin.amount_x)
            .safe_mul(price)
            .unwrap()
            .overflowing_shr(64);
        let quote_amount_of_base_token = u64::try_from(quote_amount_of_base_token)
            .map_err(|_| ZapError::TypeCastFailed)
            .unwrap();

        let total_quote = quote_amount_of_base_token.safe_add(bin.amount_y).unwrap();

        liquidity_distributions.push(total_quote);
    }
    return liquidity_distributions;
}

pub fn get_total_amount(amount_in_bins: &[AmountInBin]) -> (u64, u64) {
    let mut amount_x = 0;
    let mut amount_y = 0;
    for bin in amount_in_bins.iter() {
        amount_x = amount_x.safe_add(bin.amount_x).unwrap();
        amount_y = amount_y.safe_add(bin.amount_y).unwrap();
    }
    return (amount_x, amount_y);
}

pub fn assert_diff_amount(a: u64, b: u64, max_bps: u64) {
    let diff = if a > b { a - b } else { b - a };
    let diff_bps = diff.safe_mul(10_000).unwrap().safe_div(a).unwrap();
    assert!(diff_bps <= max_bps);
}

pub fn get_bin_add_liquidity(
    params: &RebalanceLiquidityParams,
    active_id: i32,
    bin_step: u16,
) -> Result<Vec<AmountInBin>> {
    let mut amounts_in_bin = vec![];
    for bin_range in params.adds.iter() {
        let add_liquidity_params = AddLiquidityParams {
            min_delta_id: bin_range.min_delta_id,
            max_delta_id: bin_range.max_delta_id,
            x0: bin_range.x0,
            y0: bin_range.y0,
            delta_x: bin_range.delta_x,
            delta_y: bin_range.delta_y,
            bit_flag: bin_range.bit_flag,
            favor_x_in_active_id: bin_range.favor_x_in_active_id,
            padding: bin_range.padding,
        };
        let mut amount_into_bin_range =
            add_liquidity_params.to_amount_in_bins(active_id, bin_step)?;
        amounts_in_bin.append(&mut amount_into_bin_range);
    }
    Ok(amounts_in_bin)
}

// total_x = x0 * (1+b)^-(active_id) + (x0 + delta_x) * (1+b)^-(active_id + 1) + .. (x0 + delta_x * max_delta_id) * -(1+b)^(active_id + max_delta_id)
#[derive(AnchorSerialize, AnchorDeserialize, Eq, PartialEq, Clone, Debug, Default)]
pub struct AddLiquidityParams {
    pub min_delta_id: i32, // min_bin_id = active_id + min_delta_id, min_delta_id <= max_delta_id
    pub max_delta_id: i32, // max_bin_id = active_id + max_delta_id
    pub x0: u64,
    pub y0: u64,
    pub delta_x: u64,
    pub delta_y: u64,
    pub bit_flag: u8,
    pub favor_x_in_active_id: bool, // only x or only y in active id, that flag is used when user deposit both tokens in active id
    pub padding: [u8; 16],          // padding for future use
}

pub const X0_NEG_FLAG: u8 = 0b1;
pub const Y0_NEG_FLAG: u8 = 0b10;
pub const DELTA_X_NEG_FLAG: u8 = 0b100;
pub const DELTA_Y_NEG_FLAG: u8 = 0b1000;

impl AddLiquidityParams {
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

    pub fn parse(&self) -> Result<ParsedAddLiquidityParams> {
        let &AddLiquidityParams {
            x0,
            y0,
            delta_x,
            delta_y,
            bit_flag,
            ..
        } = self;

        let mut parsed_x0 = i128::from(x0);
        let mut parsed_y0 = i128::from(y0);
        let mut parsed_delta_x = i128::from(delta_x);
        let mut parsed_delta_y = i128::from(delta_y);

        if bit_flag & X0_NEG_FLAG != 0 {
            parsed_x0 = parsed_x0.neg();
        }
        if bit_flag & Y0_NEG_FLAG != 0 {
            parsed_y0 = parsed_y0.neg();
        }
        if bit_flag & DELTA_X_NEG_FLAG != 0 {
            parsed_delta_x = parsed_delta_x.neg();
        }
        if bit_flag & DELTA_Y_NEG_FLAG != 0 {
            parsed_delta_y = parsed_delta_y.neg();
        }

        Ok(ParsedAddLiquidityParams {
            x0: parsed_x0,
            y0: parsed_y0,
            delta_x: parsed_delta_x,
            delta_y: parsed_delta_y,
        })
    }

    fn to_amount_in_bins(&self, active_id: i32, bin_step: u16) -> Result<Vec<AmountInBin>> {
        let parsed_params = self.parse()?;
        // only deposit y
        if self.is_only_deposit_y() {
            return get_amount_in_bins_bid_side(
                active_id,
                self.min_delta_id,
                self.max_delta_id,
                parsed_params.delta_y,
                parsed_params.y0,
            );
        }
        // only deposit x
        if self.is_only_deposit_x() {
            return get_amount_in_bins_ask_side(
                active_id,
                bin_step,
                self.min_delta_id,
                self.max_delta_id,
                parsed_params.delta_x,
                parsed_params.x0,
            );
        }

        // deposit both x and y, min_delta_id <= bid_side_end_delta_id && max_delta_id >= ask_side_start_delta_id
        let (bid_side_end_delta_id, ask_side_start_delta_id) = self.get_max_delta_id_both_side();

        let amounts_bid_side = get_amount_in_bins_bid_side(
            active_id,
            self.min_delta_id,
            bid_side_end_delta_id,
            parsed_params.delta_y,
            parsed_params.y0,
        )?;

        let amounts_ask_side = get_amount_in_bins_ask_side(
            active_id,
            bin_step,
            ask_side_start_delta_id,
            self.max_delta_id,
            parsed_params.delta_x,
            parsed_params.x0,
        )?;

        Ok([amounts_bid_side, amounts_ask_side].concat())
    }
}

#[derive(Default)]
pub struct ParsedAddLiquidityParams {
    pub x0: i128,
    pub y0: i128,
    pub delta_x: i128,
    pub delta_y: i128,
}

// impl ParsedAddLiquidityParams {
//     #[cfg(test)]
//     pub fn unparse(
//         &self,
//         min_delta_id: i32,
//         max_delta_id: i32,
//         favor_x_in_active_id: bool,
//     ) -> Result<AddLiquidityParams> {
//         let &ParsedAddLiquidityParams {
//             x0,
//             y0,
//             delta_x,
//             delta_y,
//         } = self;

//         let mut bit_flag = 0;

//         if x0 < 0 {
//             bit_flag |= X0_NEG_FLAG;
//         }
//         if y0 < 0 {
//             bit_flag |= Y0_NEG_FLAG;
//         }
//         if delta_x < 0 {
//             bit_flag |= DELTA_X_NEG_FLAG;
//         }
//         if delta_y < 0 {
//             bit_flag |= DELTA_Y_NEG_FLAG;
//         }

//         Ok(AddLiquidityParams {
//             min_delta_id,
//             max_delta_id,
//             x0: u64::try_from(x0.abs()).map_err(|_| ZapError::TypeCastFailed)?,
//             y0: u64::try_from(y0.abs()).map_err(|_| ZapError::TypeCastFailed)?,
//             delta_x: u64::try_from(delta_x.abs()).map_err(|_| ZapError::TypeCastFailed)?,
//             delta_y: u64::try_from(delta_y.abs()).map_err(|_| ZapError::TypeCastFailed)?,
//             favor_x_in_active_id,
//             bit_flag,
//             padding: [0u8; 16],
//         })
//     }
// }

pub fn get_amount_in_bins_bid_side(
    active_id: i32,
    min_delta_id: i32,
    max_delta_id: i32,
    delta_y: i128,
    y0: i128,
) -> Result<Vec<AmountInBin>> {
    // This won't be negative because already validated min_bin_id <= max_bin_id at validate_and_get_bin_range
    let bin_count = max_delta_id.safe_sub(min_delta_id)?.safe_add(1)?;
    let mut amounts_in_bin = vec![AmountInBin::default(); bin_count as usize];

    let min_bin_id = active_id.safe_add(min_delta_id)?;
    let max_bin_id = active_id.safe_add(max_delta_id)?;

    for (idx, bin_id) in (min_bin_id..=max_bin_id).enumerate() {
        let delta_bin = active_id.safe_sub(bin_id)?;

        let total_delta_y = delta_y.safe_mul(delta_bin.into())?;

        let amount_y = y0
            .safe_add(total_delta_y)?
            .try_into()
            .map_err(|_| ZapError::TypeCastFailed)?;

        amounts_in_bin[idx] = AmountInBin {
            bin_id,
            amount_x: 0,
            amount_y,
        };
    }

    Ok(amounts_in_bin)
}

pub fn get_amount_in_bins_ask_side(
    active_id: i32,
    bin_step: u16,
    min_delta_id: i32,
    max_delta_id: i32,
    delta_x: i128,
    x0: i128,
) -> Result<Vec<AmountInBin>> {
    // This won't be negative because already validated min_bin_id <= max_bin_id at validate_and_get_bin_range
    let bin_count = max_delta_id.safe_sub(min_delta_id)?.safe_add(1)?;

    let base_u128 = get_price_base_factor(bin_step)?;
    let base = U256::from(base_u128);

    let max_bin_id = active_id.safe_add(max_delta_id)?;
    let min_bin_id = active_id.safe_add(min_delta_id)?;

    // we use inverse base price to avoid safe_div (safe_mul can save more CU)
    let mut inverse_base_price =
        U256::from(pow(base_u128, max_bin_id.neg().into()).ok_or_else(|| ZapError::MathOverflow)?);

    let mut amounts_in_bin = vec![AmountInBin::default(); bin_count as usize];
    let mut current_bin_id = max_bin_id;

    loop {
        if current_bin_id < min_bin_id {
            break;
        }

        let delta_bin = current_bin_id.safe_sub(active_id)?;

        let total_delta_x = delta_x.safe_mul(delta_bin.into())?;

        let amount_x =
            U256::try_from(x0.safe_add(total_delta_x)?).map_err(|_| ZapError::TypeCastFailed)?;

        let (amount_x, _) = amount_x.safe_mul(inverse_base_price)?.overflowing_shr(64);

        let amount_x = u64::try_from(amount_x).map_err(|_| ZapError::TypeCastFailed)?;

        let idx: usize = current_bin_id
            .safe_sub(min_bin_id)?
            .try_into()
            .map_err(|_| ZapError::TypeCastFailed)?;

        amounts_in_bin[idx] = AmountInBin {
            bin_id: current_bin_id,
            amount_x,
            amount_y: 0,
        };

        (inverse_base_price, _) = inverse_base_price.safe_mul(base)?.overflowing_shr(64);

        current_bin_id = current_bin_id.safe_sub(1)?;
    }

    Ok(amounts_in_bin)
}
