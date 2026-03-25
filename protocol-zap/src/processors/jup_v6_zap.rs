use std::collections::{HashMap, HashSet};

use crate::{
    constants::{
        JUP_V6_ROUTE_AMOUNT_IN_REVERSE_OFFSET, JUP_V6_ROUTE_DESTINATION_ACCOUNT_INDEX,
        JUP_V6_ROUTE_FIRST_SWAP_ACCOUNTS_OFFSET, JUP_V6_ROUTE_SOURCE_ACCOUNT_INDEX,
        JUP_V6_SHARED_ACCOUNT_ROUTE_AMOUNT_IN_REVERSE_OFFSET,
        JUP_V6_SHARED_ACCOUNT_ROUTE_DESTINATION_ACCOUNT_INDEX,
        JUP_V6_SHARED_ACCOUNT_ROUTE_SOURCE_ACCOUNT_INDEX,
    },
    error::ProtozolZapError,
    safe_math::{SafeCast, SafeMath},
    RawZapOutAmmInfo, ZapInfoProcessor, ZapOutParameters,
};
use borsh::BorshDeserialize;
use jupiter::types::{RoutePlanStep, Swap};

pub struct ZapJupV6RouteInfoProcessor;

fn ensure_whitelisted_swap_leg(route_plan_steps: &[RoutePlanStep]) -> Result<(), ProtozolZapError> {
    for step in route_plan_steps {
        // delegate to get_swap_source_index so the whitelist and source index mapping are in sync
        // but we discard the result
        get_swap_source_index(&step.swap)?;
    }

    Ok(())
}

fn get_swap_source_index(swap: &Swap) -> Result<usize, ProtozolZapError> {
    match swap {
        // Meteora ref: https://github.com/jup-ag/jupiter-aggregator-program/blob/e583ab6619f4646b4d7a0e2514aec62ae9fb62ec/dex_interfaces/src/lib.rs#L1632
        Swap::Meteora => Ok(1),
        // MeteoraDammV2 ref: https://github.com/jup-ag/jupiter-aggregator-program/blob/e583ab6619f4646b4d7a0e2514aec62ae9fb62ec/dex_interfaces/src/lib.rs#L6150
        Swap::MeteoraDammV2 | Swap::MeteoraDammV2WithRemainingAccounts => Ok(2),
        // MeteoraDlmm ref: https://github.com/jup-ag/jupiter-aggregator-program/blob/e583ab6619f4646b4d7a0e2514aec62ae9fb62ec/dex_interfaces/src/lib.rs#L2879
        // MeteoraDlmmSwapV2 ref: https://github.com/jup-ag/jupiter-aggregator-program/blob/e583ab6619f4646b4d7a0e2514aec62ae9fb62ec/dex_interfaces/src/lib.rs#L5956
        Swap::MeteoraDlmm | Swap::MeteoraDlmmSwapV2 { .. } => Ok(4),
        // Mercurial ref: https://github.com/jup-ag/jupiter-aggregator-program/blob/e583ab6619f4646b4d7a0e2514aec62ae9fb62ec/dex_interfaces/src/lib.rs#L40
        Swap::Mercurial => Ok(4),
        // Whirlpool ref: https://github.com/jup-ag/jupiter-aggregator-program/blob/e583ab6619f4646b4d7a0e2514aec62ae9fb62ec/dex_interfaces/src/lib.rs#L1396-L1398
        Swap::Whirlpool { a_to_b } => {
            if *a_to_b {
                Ok(3)
            } else {
                Ok(5)
            }
        }
        // WhirlpoolSwapV2 ref: https://github.com/jup-ag/jupiter-aggregator-program/blob/e583ab6619f4646b4d7a0e2514aec62ae9fb62ec/dex_interfaces/src/lib.rs#L1478-L1480
        Swap::WhirlpoolSwapV2 { a_to_b, .. } => {
            if *a_to_b {
                Ok(7)
            } else {
                Ok(9)
            }
        }
        // Raydium ref: https://github.com/jup-ag/jupiter-aggregator-program/blob/e583ab6619f4646b4d7a0e2514aec62ae9fb62ec/dex_interfaces/src/lib.rs#L769
        Swap::Raydium => Ok(14),
        // RaydiumV2 ref: https://github.com/jup-ag/jupiter-aggregator-program/blob/e583ab6619f4646b4d7a0e2514aec62ae9fb62ec/dex_interfaces/src/lib.rs#L849
        Swap::RaydiumV2 => Ok(5),
        // RaydiumCP ref: https://github.com/jup-ag/jupiter-aggregator-program/blob/e583ab6619f4646b4d7a0e2514aec62ae9fb62ec/dex_interfaces/src/lib.rs#L3297
        Swap::RaydiumCP => Ok(4),
        // RaydiumClmm ref: https://github.com/jup-ag/jupiter-aggregator-program/blob/e583ab6619f4646b4d7a0e2514aec62ae9fb62ec/dex_interfaces/src/lib.rs#L2247
        // RaydiumClmmV2 ref: https://github.com/jup-ag/jupiter-aggregator-program/blob/e583ab6619f4646b4d7a0e2514aec62ae9fb62ec/dex_interfaces/src/lib.rs#L2320
        Swap::RaydiumClmm | Swap::RaydiumClmmV2 => Ok(3),
        _ => Err(ProtozolZapError::InvalidZapOutParameters),
    }
}

pub(crate) fn get_jup_route_first_swap_source_account_index(
    payload: &[u8],
) -> Result<usize, ProtozolZapError> {
    let route_params = jupiter::client::args::Route::try_from_slice(payload)
        .map_err(|_| ProtozolZapError::InvalidZapOutParameters)?;

    let first_step = route_params
        .route_plan
        .first()
        .ok_or_else(|| ProtozolZapError::InvalidZapOutParameters)?;

    if first_step.input_index != 0 {
        return Err(ProtozolZapError::InvalidZapOutParameters);
    }

    let source_index = get_swap_source_index(&first_step.swap)?;

    JUP_V6_ROUTE_FIRST_SWAP_ACCOUNTS_OFFSET.safe_add(source_index)
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
