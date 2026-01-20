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
use anchor_lang::prelude::*;
use jupiter::types::RoutePlanStep;
use jupiter::types::Swap;

pub struct ZapJupV6RouteInfoProcessor;

fn ensure_whitelisted_swap_leg(route_plan_steps: &[RoutePlanStep]) -> Result<()> {
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
            _ => return Err(ProtocolZapError::InvalidZapOutParameters.into()),
        }
    }

    Ok(())
}

/// Validates that the route plan fully converges
/// - Every input index (original and intermediate) must be 100% consumed
/// - All swap paths must converge to exactly one terminal output
fn ensure_route_plan_fully_converges(route_plan_steps: &[RoutePlanStep]) -> Result<()> {
    // Verify each unique input_index sums to exactly 100%
    for (i, step) in route_plan_steps.iter().enumerate() {
        // Only process first occurrence of each input_index
        let seen = route_plan_steps[..i]
            .iter()
            .any(|s| s.input_index == step.input_index);
        if seen {
            continue;
        }

        let percent_sum = route_plan_steps
            .iter()
            .filter(|s| s.input_index == step.input_index)
            .try_fold(0u8, |acc, s| acc.checked_add(s.percent))
            .ok_or(ProtocolZapError::MathOverflow)?;

        require!(
            percent_sum == 100,
            ProtocolZapError::InvalidZapOutParameters
        );
    }

    // Count terminal outputs: unique outputs never used as inputs
    let terminal_count = route_plan_steps
        .iter()
        .enumerate()
        .filter(|(i, step)| {
            let is_first = !route_plan_steps[..*i]
                .iter()
                .any(|s| s.output_index == step.output_index);
            let is_terminal = !route_plan_steps
                .iter()
                .any(|s| s.input_index == step.output_index);
            is_first && is_terminal
        })
        .count();

    require!(
        terminal_count == 1,
        ProtocolZapError::InvalidZapOutParameters
    );

    Ok(())
}

impl ZapInfoProcessor for ZapJupV6RouteInfoProcessor {
    fn validate_payload(&self, payload: &[u8]) -> Result<()> {
        let route_params = jupiter::client::args::Route::try_from_slice(payload)?;
        ensure_whitelisted_swap_leg(&route_params.route_plan)?;
        ensure_route_plan_fully_converges(&route_params.route_plan)?;

        // Ensure no platform_fee_bps is 0, so operator can't steal funds by providing their account as platform_fee_account
        require!(
            route_params.platform_fee_bps == 0,
            ProtocolZapError::InvalidZapOutParameters
        );

        Ok(())
    }

    fn extract_raw_zap_out_amm_info(
        &self,
        zap_params: &ZapOutParameters,
    ) -> Result<RawZapOutAmmInfo> {
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
    fn validate_payload(&self, payload: &[u8]) -> Result<()> {
        let route_params = jupiter::client::args::SharedAccountsRoute::try_from_slice(payload)?;
        ensure_whitelisted_swap_leg(&route_params.route_plan)?;
        ensure_route_plan_fully_converges(&route_params.route_plan)?;

        // Ensure no platform_fee_bps is 0, so operator can't steal funds by providing their account as platform_fee_account
        require!(
            route_params.platform_fee_bps == 0,
            ProtocolZapError::InvalidZapOutParameters
        );

        Ok(())
    }

    fn extract_raw_zap_out_amm_info(
        &self,
        zap_params: &ZapOutParameters,
    ) -> Result<RawZapOutAmmInfo> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_plan_converges_with_100_percent() {
        let route_plan_1_market = vec![RoutePlanStep {
            swap: Swap::Meteora,
            percent: 100,
            input_index: 0,
            output_index: 1,
        }];
        assert!(ensure_route_plan_fully_converges(&route_plan_1_market).is_ok());
        let route_plan_multi_market: Vec<RoutePlanStep> = vec![
            RoutePlanStep {
                swap: Swap::Meteora,
                percent: 50,
                input_index: 0,
                output_index: 1,
            },
            RoutePlanStep {
                swap: Swap::Raydium,
                percent: 50,
                input_index: 0,
                output_index: 1,
            },
            RoutePlanStep {
                swap: Swap::MeteoraDlmm,
                percent: 100,
                input_index: 1,
                output_index: 2,
            },
        ];
        assert!(ensure_route_plan_fully_converges(&route_plan_multi_market).is_ok());
    }

    #[test]
    fn test_route_plan_fails_with_partial_percent() {
        let route_plan_1_market = vec![RoutePlanStep {
            swap: Swap::Meteora,
            percent: 50,
            input_index: 0,
            output_index: 1,
        }];
        assert!(ensure_route_plan_fully_converges(&route_plan_1_market).is_err());
        let route_plan_1_market = vec![
            RoutePlanStep {
                swap: Swap::Meteora,
                percent: 100,
                input_index: 0,
                output_index: 1,
            },
            RoutePlanStep {
                swap: Swap::Raydium,
                percent: 50,
                input_index: 1,
                output_index: 2,
            },
            RoutePlanStep {
                swap: Swap::MeteoraDlmm,
                percent: 50,
                input_index: 1,
                output_index: 3,
            },
        ];
        assert!(ensure_route_plan_fully_converges(&route_plan_1_market).is_err());
    }
}
