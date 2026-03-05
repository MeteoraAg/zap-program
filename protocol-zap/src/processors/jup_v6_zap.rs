use std::collections::{HashMap, HashSet};

use crate::{
    constants::{
        JUP_V6_ROUTE_AMOUNT_IN_REVERSE_OFFSET, JUP_V6_ROUTE_DESTINATION_ACCOUNT_INDEX,
        JUP_V6_ROUTE_SOURCE_ACCOUNT_INDEX, JUP_V6_SHARED_ACCOUNT_ROUTE_AMOUNT_IN_REVERSE_OFFSET,
        JUP_V6_SHARED_ACCOUNT_ROUTE_DESTINATION_ACCOUNT_INDEX,
        JUP_V6_SHARED_ACCOUNT_ROUTE_SOURCE_ACCOUNT_INDEX,
    },
    error::ProtocolZapError,
    jup_swap_step_referral_fee_parser::get_referral_fee_parser,
    safe_math::{SafeCast, SafeMath},
    RawZapOutAmmInfo, ZapInfoProcessor, ZapOutParameters,
};
use borsh::BorshDeserialize;
use jupiter::types::Swap;
use jupiter::types::{RemainingAccountsInfo, RoutePlanStep};
use pinocchio::sysvars::instructions::IntrospectedInstruction;

pub struct ZapJupV6RouteInfoProcessor {
    route_params: jupiter::client::args::Route,
}

impl ZapJupV6RouteInfoProcessor {
    // Jupiter V6 Route instruction has 9 accounts before the remaining accounts which are used for swap legs.
    const ROUTE_BASE_ACCOUNT_LENGTH: usize = 9;

    pub fn new(payload: &[u8]) -> Result<Self, ProtocolZapError> {
        let route_params = jupiter::client::args::Route::try_from_slice(payload)
            .map_err(|_| ProtocolZapError::InvalidZapOutParameters)?;

        Ok(Self { route_params })
    }
}

#[derive(Debug, Clone)]
pub enum WhitelistedSwapStep {
    Meteora,
    MeteoraDammV2,
    MeteoraDammV2WithRemainingAccounts,
    MeteoraDlmm,
    MeteoraDlmmSwapV2,
    Mercurial,
    Whirlpool,
    WhirlpoolSwapV2 {
        remaining_accounts_info: Option<RemainingAccountsInfo>,
    },
    Raydium,
    RaydiumV2,
    RaydiumCP,
    RaydiumClmm,
    RaydiumClmmV2,
}

impl TryFrom<&Swap> for WhitelistedSwapStep {
    type Error = ProtocolZapError;

    fn try_from(value: &Swap) -> Result<Self, Self::Error> {
        match value {
            Swap::Meteora => Ok(Self::Meteora),
            Swap::MeteoraDammV2 => Ok(Self::MeteoraDammV2),
            Swap::MeteoraDammV2WithRemainingAccounts => {
                Ok(Self::MeteoraDammV2WithRemainingAccounts)
            }
            Swap::MeteoraDlmm => Ok(Self::MeteoraDlmm),
            Swap::MeteoraDlmmSwapV2 { .. } => Ok(Self::MeteoraDlmmSwapV2),
            Swap::Mercurial => Ok(Self::Mercurial),
            Swap::Whirlpool { .. } => Ok(Self::Whirlpool),
            Swap::WhirlpoolSwapV2 {
                remaining_accounts_info,
                ..
            } => Ok(Self::WhirlpoolSwapV2 {
                remaining_accounts_info: remaining_accounts_info.clone(),
            }),
            Swap::Raydium => Ok(Self::Raydium),
            Swap::RaydiumV2 => Ok(Self::RaydiumV2),
            Swap::RaydiumCP => Ok(Self::RaydiumCP),
            Swap::RaydiumClmm => Ok(Self::RaydiumClmm),
            Swap::RaydiumClmmV2 => Ok(Self::RaydiumClmmV2),
            _ => Err(ProtocolZapError::NonWhitelistedSwapStep),
        }
    }
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

    fn validate_route_plan(
        &self,
        zap_out_instruction: &IntrospectedInstruction<'_>,
    ) -> Result<(), ProtocolZapError> {
        // In Jupiter, once platform_fee_bps is set to 0, the platform_fee_account is not read at all
        // Ensure platform_fee_bps is 0, so operator can't steal funds by providing their account as platform_fee_account
        // Jupiter platform fee is a fee charged on global input amount / output amount depending on platform_fee_account.mint
        // Global input amount = input amount of jupiter swap before split into multiple legs
        // Global output amount = output amount of jupiter swap after all legs are completed
        if self.route_params.platform_fee_bps != 0 {
            return Err(ProtocolZapError::InvalidZapOutParameters);
        }

        internal_validate_route_plan(
            &self.route_params.route_plan,
            Self::ROUTE_BASE_ACCOUNT_LENGTH,
            zap_out_instruction,
        )?;

        Ok(())
    }
}

pub struct ZapJupV6SharedRouteInfoProcessor {
    route_params: jupiter::client::args::SharedAccountsRoute,
}

impl ZapJupV6SharedRouteInfoProcessor {
    // Jupiter V6 SharedAccountsRoute instruction has 13 accounts before the remaining accounts which are used for swap legs.
    const ROUTE_BASE_ACCOUNT_LENGTH: usize = 13;

    pub fn new(payload: &[u8]) -> Result<Self, ProtocolZapError> {
        let route_params = jupiter::client::args::SharedAccountsRoute::try_from_slice(payload)
            .map_err(|_| ProtocolZapError::InvalidZapOutParameters)?;

        Ok(Self { route_params })
    }
}

impl ZapInfoProcessor for ZapJupV6SharedRouteInfoProcessor {
    fn validate_payload(&self) -> Result<(), ProtocolZapError> {
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

    fn validate_route_plan(
        &self,
        zap_out_instruction: &IntrospectedInstruction<'_>,
    ) -> Result<(), ProtocolZapError> {
        // In Jupiter, once platform_fee_bps is set to 0, the platform_fee_account is not read at all
        // Ensure platform_fee_bps is 0, so operator can't steal funds by providing their account as platform_fee_account
        // Jupiter platform fee is a fee charged on global input amount / output amount depending on platform_fee_account.mint
        // Global input amount = input amount of jupiter swap before split into multiple legs
        // Global output amount = output amount of jupiter swap after all legs are completed
        if self.route_params.platform_fee_bps != 0 {
            return Err(ProtocolZapError::InvalidZapOutParameters);
        }

        internal_validate_route_plan(
            &self.route_params.route_plan,
            Self::ROUTE_BASE_ACCOUNT_LENGTH,
            zap_out_instruction,
        )?;

        Ok(())
    }
}

fn internal_validate_route_plan(
    route_plan_steps: &[RoutePlanStep],
    base_account_offset: usize,
    zap_out_instruction: &IntrospectedInstruction<'_>,
) -> Result<(), ProtocolZapError> {
    let mut index = base_account_offset;

    let mut iter = route_plan_steps.iter().peekable();

    while let Some(step) = iter.next() {
        let swap_step = WhitelistedSwapStep::try_from(&step.swap)?;

        let mut referral_fee_parser = get_referral_fee_parser(&swap_step);

        let next_swap_step = iter.peek().copied();
        referral_fee_parser.load_next_swap_step(next_swap_step)?;
        referral_fee_parser.ensure_no_referral_fee_account(index, zap_out_instruction)?;

        index = referral_fee_parser.get_end_account_index(index, zap_out_instruction)?;
    }

    Ok(())
}
