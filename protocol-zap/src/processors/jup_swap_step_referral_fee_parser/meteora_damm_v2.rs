use crate::{
    error::ProtocolZapError,
    jup_swap_step_referral_fee_parser::{
        adjust_processed_index_to_next_swap_step_base_start_index, must_retrieve_account_meta,
        SwapStepReferralFeeParser,
    },
    safe_math::SafeMath,
};
use pinocchio::sysvars::instructions::IntrospectedInstruction;

// Index of referral fee account in base account
const REFERRAL_ACCOUNT_INDEX: usize = 12;
const BASE_ACCOUNT_LENGTH: usize = 14;

fn internal_ensure_no_referral_fee_account<'a>(
    processed_index: usize,
    zap_out_instruction: &'a IntrospectedInstruction<'a>,
) -> Result<(), ProtocolZapError> {
    let start_account_index =
        adjust_processed_index_to_next_swap_step_base_start_index(processed_index)?;
    let referral_fee_index = start_account_index.safe_add(REFERRAL_ACCOUNT_INDEX)?;

    let referral_fee_account_meta =
        must_retrieve_account_meta(zap_out_instruction, referral_fee_index)?;

    // DAMM v2 use it's own account as placeholder of Option::None
    if referral_fee_account_meta
        .key
        .ne(zap_sdk::constants::DAMM_V2.as_array())
    {
        return Err(ProtocolZapError::ReferralFeeNotAllowed);
    }

    Ok(())
}

// Meteora DAMM v2
pub struct MeteoraDammV2;

impl SwapStepReferralFeeParser for MeteoraDammV2 {
    fn get_base_account_length(&self) -> usize {
        BASE_ACCOUNT_LENGTH
    }

    fn ensure_no_referral_fee_account<'a>(
        &self,
        processed_index: usize,
        zap_out_instruction: &'a IntrospectedInstruction<'a>,
    ) -> Result<(), ProtocolZapError> {
        internal_ensure_no_referral_fee_account(processed_index, zap_out_instruction)
    }

    fn get_end_account_index<'a>(
        &self,
        processed_index: usize,
        _zap_out_instruction: &'a IntrospectedInstruction<'a>,
    ) -> Result<usize, ProtocolZapError> {
        self.get_end_account_index_default(processed_index)
    }
}

// Meteora DAMM v2 with fee rate limiter.
pub struct MeteoraDammV2WithRemainingAccounts;

impl SwapStepReferralFeeParser for MeteoraDammV2WithRemainingAccounts {
    fn get_base_account_length(&self) -> usize {
        BASE_ACCOUNT_LENGTH
    }

    fn ensure_no_referral_fee_account<'a>(
        &self,
        processed_index: usize,
        zap_out_instruction: &'a IntrospectedInstruction<'a>,
    ) -> Result<(), ProtocolZapError> {
        internal_ensure_no_referral_fee_account(processed_index, zap_out_instruction)
    }

    fn get_end_account_index<'a>(
        &self,
        processed_index: usize,
        zap_out_instruction: &'a IntrospectedInstruction<'a>,
    ) -> Result<usize, ProtocolZapError> {
        self.get_end_account_index_via_placeholder(processed_index, zap_out_instruction)
    }
}
