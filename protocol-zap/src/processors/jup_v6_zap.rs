use std::collections::{HashMap, HashSet};

use crate::{
    constants::{
        JUP_V6_ROUTE_AMOUNT_IN_REVERSE_OFFSET, JUP_V6_ROUTE_DESTINATION_ACCOUNT_INDEX,
        JUP_V6_ROUTE_SOURCE_ACCOUNT_INDEX, JUP_V6_SHARED_ACCOUNT_ROUTE_AMOUNT_IN_REVERSE_OFFSET,
        JUP_V6_SHARED_ACCOUNT_ROUTE_DESTINATION_ACCOUNT_INDEX,
        JUP_V6_SHARED_ACCOUNT_ROUTE_SOURCE_ACCOUNT_INDEX,
    },
    error::ProtocolZapError,
    safe_math::{SafeCast, SafeMath},
    RawZapOutAmmInfo, ZapInfoProcessor, ZapOutParameters,
};
use borsh::BorshDeserialize;
use jupiter::types::RoutePlanStep;
use jupiter::types::Swap;
use pinocchio::sysvars::instructions::IntrospectedInstruction;

pub struct ZapJupV6RouteInfoProcessor {
    route_params: jupiter::client::args::Route,
}

impl ZapJupV6RouteInfoProcessor {
    pub fn new(payload: &[u8]) -> Result<Self, ProtocolZapError> {
        let route_params = jupiter::client::args::Route::try_from_slice(payload)
            .map_err(|_| ProtocolZapError::InvalidZapOutParameters)?;

        Ok(Self { route_params })
    }
}

fn ensure_whitelisted_swap_leg(route_plan_steps: &[RoutePlanStep]) -> Result<(), ProtocolZapError> {
    for step in route_plan_steps {
        match step.swap {
            Swap::Meteora
            | Swap::MeteoraDammV2
            | Swap::MeteoraDammV2WithRemainingAccounts
            | Swap::MeteoraDlmm
            | Swap::MeteoraDlmmSwapV2 { .. }
            | Swap::Mercurial
            | Swap::Whirlpool { .. }
            | Swap::WhirlpoolSwapV2 { .. }
            | Swap::Raydium
            | Swap::RaydiumV2
            | Swap::RaydiumCP
            | Swap::RaydiumClmm
            | Swap::RaydiumClmmV2 => {
                // whitelisted swap leg
            }
            _ => return Err(ProtocolZapError::InvalidZapOutParameters),
        }
    }

    Ok(())
}

/// Validates that the route plan fully converges
/// - Every input index (original and intermediate) must be 100% consumed
/// - All swap paths must converge to exactly one terminal output
pub(crate) fn ensure_route_plan_fully_converges(
    route_plan_steps: &[RoutePlanStep],
) -> Result<(), ProtocolZapError> {
    let mut input_percent: HashMap<u8, u8> = HashMap::new();
    let mut output_indices = HashSet::new();

    for step in route_plan_steps {
        let percent = input_percent.entry(step.input_index).or_insert(0);
        *percent = percent
            .checked_add(step.percent)
            .ok_or_else(|| ProtocolZapError::MathOverflow)?;
        output_indices.insert(step.output_index);
    }

    // Verify each unique input_index sums to exactly 100%
    if input_percent.values().any(|value| *value != 100) {
        return Err(ProtocolZapError::InvalidZapOutParameters);
    }

    // Count terminal outputs: unique outputs never used as inputs
    let terminal_count = output_indices
        .iter()
        .filter(|idx| !input_percent.contains_key(idx))
        .count();

    if terminal_count != 1 {
        return Err(ProtocolZapError::InvalidZapOutParameters);
    }

    Ok(())
}

impl ZapInfoProcessor for ZapJupV6RouteInfoProcessor {
    fn validate_payload(&self) -> Result<(), ProtocolZapError> {
        ensure_whitelisted_swap_leg(&self.route_params.route_plan)?;
        ensure_route_plan_fully_converges(&self.route_params.route_plan)?;

        Ok(())
    }

    fn extract_raw_zap_out_amm_info(
        &self,
        zap_params: &ZapOutParameters,
    ) -> Result<RawZapOutAmmInfo, ProtocolZapError> {
        let amount_in_offset = zap_params
            .payload_data
            .len()
            .safe_sub(JUP_V6_ROUTE_AMOUNT_IN_REVERSE_OFFSET)?
            .safe_cast()?;

        Ok(RawZapOutAmmInfo {
            source_index: JUP_V6_ROUTE_SOURCE_ACCOUNT_INDEX,
            destination_index: JUP_V6_ROUTE_DESTINATION_ACCOUNT_INDEX,
            amount_in_offset,
        })
    }

    fn ensure_no_referral_fee(
        &self,
        _zap_out_instruction: &IntrospectedInstruction<'_>,
    ) -> Result<(), ProtocolZapError> {
        // In Jupiter, once platform_fee_bps is set to 0, the platform_fee_account is not read at all
        // Ensure platform_fee_bps is 0, so operator can't steal funds by providing their account as platform_fee_account
        if self.route_params.platform_fee_bps != 0 {
            return Err(ProtocolZapError::InvalidZapOutParameters);
        }

        Ok(())
    }
}

pub struct ZapJupV6SharedRouteInfoProcessor {
    route_params: jupiter::client::args::SharedAccountsRoute,
}

impl ZapJupV6SharedRouteInfoProcessor {
    pub fn new(payload: &[u8]) -> Result<Self, ProtocolZapError> {
        let route_params = jupiter::client::args::SharedAccountsRoute::try_from_slice(payload)
            .map_err(|_| ProtocolZapError::InvalidZapOutParameters)?;

        Ok(Self { route_params })
    }
}

impl ZapInfoProcessor for ZapJupV6SharedRouteInfoProcessor {
    fn validate_payload(&self) -> Result<(), ProtocolZapError> {
        ensure_whitelisted_swap_leg(&self.route_params.route_plan)?;
        ensure_route_plan_fully_converges(&self.route_params.route_plan)?;

        Ok(())
    }

    fn extract_raw_zap_out_amm_info(
        &self,
        zap_params: &ZapOutParameters,
    ) -> Result<RawZapOutAmmInfo, ProtocolZapError> {
        let amount_in_offset = zap_params
            .payload_data
            .len()
            .safe_sub(JUP_V6_SHARED_ACCOUNT_ROUTE_AMOUNT_IN_REVERSE_OFFSET)?
            .safe_cast()?;

        Ok(RawZapOutAmmInfo {
            source_index: JUP_V6_SHARED_ACCOUNT_ROUTE_SOURCE_ACCOUNT_INDEX,
            destination_index: JUP_V6_SHARED_ACCOUNT_ROUTE_DESTINATION_ACCOUNT_INDEX,
            amount_in_offset,
        })
    }

    fn ensure_no_referral_fee(
        &self,
        _zap_out_instruction: &IntrospectedInstruction<'_>,
    ) -> Result<(), ProtocolZapError> {
        // In Jupiter, once platform_fee_bps is set to 0, the platform_fee_account is not read at all
        // Ensure platform_fee_bps is 0, so operator can't steal funds by providing their account as platform_fee_account
        if self.route_params.platform_fee_bps != 0 {
            return Err(ProtocolZapError::InvalidZapOutParameters);
        }

        Ok(())
    }
}
