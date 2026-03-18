use std::collections::{HashMap, HashSet};

use crate::{
    constants::{
        JUP_V6_ROUTE_AMOUNT_IN_REVERSE_OFFSET, JUP_V6_ROUTE_DESTINATION_ACCOUNT_INDEX,
        JUP_V6_ROUTE_SOURCE_ACCOUNT_INDEX, JUP_V6_SHARED_ACCOUNT_ROUTE_AMOUNT_IN_REVERSE_OFFSET,
        JUP_V6_SHARED_ACCOUNT_ROUTE_DESTINATION_ACCOUNT_INDEX,
        JUP_V6_SHARED_ACCOUNT_ROUTE_SOURCE_ACCOUNT_INDEX,
    },
    error::ProtozolZapError,
    safe_math::{SafeCast, SafeMath},
    RawZapOutAmmInfo, ZapInfoProcessor, ZapOutParameters,
};
use borsh::BorshDeserialize;
use jupiter::types::RoutePlanStep;
use jupiter::types::Swap;

pub struct ZapJupV6RouteInfoProcessor;

fn ensure_whitelisted_swap_leg(route_plan_steps: &[RoutePlanStep]) -> Result<(), ProtozolZapError> {
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
            _ => return Err(ProtozolZapError::InvalidZapOutParameters),
        }
    }

    Ok(())
}

/// Validates the route plan:
/// - every input index (original and intermediate) must be 100% consumed
/// - root input (index 0) must be consumed
/// - prevent phantom input, a non-root input must have been produced by a prior step
/// - cannot output to an index already used as input
/// - all swap paths must converge to exactly one terminal output
pub(crate) fn validate_route_plan(
    route_plan_steps: &[RoutePlanStep],
) -> Result<(), ProtozolZapError> {
    let mut input_indices_seen = HashSet::new();
    let mut input_percent: HashMap<u8, u8> = HashMap::new();
    let mut output_indices = HashSet::new();

    for step in route_plan_steps {
        // prevent phantom input, a non-root input must have been produced by a prior step
        // this also ensures root input (index 0) is consumed
        // output_indices starts empty, so the first step must use input_index 0 or it will error
        if step.input_index != 0 && !output_indices.contains(&step.input_index) {
            return Err(ProtozolZapError::InvalidZapOutParameters);
        }

        // cannot output to an index already used as input
        input_indices_seen.insert(step.input_index);
        if input_indices_seen.contains(&step.output_index) {
            return Err(ProtozolZapError::InvalidZapOutParameters);
        }

        let percent = input_percent.entry(step.input_index).or_insert(0);
        *percent = percent
            .checked_add(step.percent)
            .ok_or_else(|| ProtozolZapError::MathOverflow)?;

        output_indices.insert(step.output_index);
    }

    // Verify each unique input_index sums to exactly 100%
    if input_percent.values().any(|value| *value != 100) {
        return Err(ProtozolZapError::InvalidZapOutParameters);
    }

    // Count terminal outputs: unique outputs never used as inputs
    let terminal_count = output_indices
        .iter()
        .filter(|idx| !input_percent.contains_key(idx))
        .count();

    if terminal_count != 1 {
        return Err(ProtozolZapError::InvalidZapOutParameters);
    }

    Ok(())
}

impl ZapInfoProcessor for ZapJupV6RouteInfoProcessor {
    fn validate_payload(&self, payload: &[u8]) -> Result<(), ProtozolZapError> {
        let route_params = jupiter::client::args::Route::try_from_slice(payload)
            .map_err(|_| ProtozolZapError::InvalidZapOutParameters)?;
        ensure_whitelisted_swap_leg(&route_params.route_plan)?;
        validate_route_plan(&route_params.route_plan)?;

        // Ensure platform_fee_bps is 0, so operator can't steal funds by providing their account as platform_fee_account
        if route_params.platform_fee_bps != 0 {
            return Err(ProtozolZapError::InvalidZapOutParameters);
        }

        Ok(())
    }

    fn extract_raw_zap_out_amm_info(
        &self,
        zap_params: &ZapOutParameters,
    ) -> Result<RawZapOutAmmInfo, ProtozolZapError> {
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
}

pub struct ZapJupV6SharedRouteInfoProcessor;

impl ZapInfoProcessor for ZapJupV6SharedRouteInfoProcessor {
    fn validate_payload(&self, payload: &[u8]) -> Result<(), ProtozolZapError> {
        let route_params = jupiter::client::args::SharedAccountsRoute::try_from_slice(payload)
            .map_err(|_| ProtozolZapError::InvalidZapOutParameters)?;
        ensure_whitelisted_swap_leg(&route_params.route_plan)?;
        validate_route_plan(&route_params.route_plan)?;

        // Ensure platform_fee_bps is 0, so operator can't steal funds by providing their account as platform_fee_account
        if route_params.platform_fee_bps != 0 {
            return Err(ProtozolZapError::InvalidZapOutParameters);
        }

        Ok(())
    }

    fn extract_raw_zap_out_amm_info(
        &self,
        zap_params: &ZapOutParameters,
    ) -> Result<RawZapOutAmmInfo, ProtozolZapError> {
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
}
