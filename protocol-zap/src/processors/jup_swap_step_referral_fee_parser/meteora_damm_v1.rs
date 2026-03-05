use crate::{
    error::ProtocolZapError,
    jup_swap_step_referral_fee_parser::{
        adjust_processed_index_to_next_swap_step_base_start_index, is_placeholder_account,
        must_retrieve_account_meta, SwapStepReferralFeeParser,
    },
    safe_math::SafeMath,
};
use pinocchio::sysvars::instructions::IntrospectedInstruction;

// Meteora DAMM v1
pub struct Meteora;

impl Meteora {
    // 0. Stake pool account
    // 1. Referral fee account
    const REMAINING_ACCOUNTS_LENGTH: usize = 2;
}

impl SwapStepReferralFeeParser for Meteora {
    fn get_base_account_length(&self) -> usize {
        15
    }

    fn ensure_no_referral_fee_account<'a>(
        &self,
        processed_index: usize,
        zap_out_instruction: &'a IntrospectedInstruction<'a>,
    ) -> Result<(), ProtocolZapError> {
        let start_account_index =
            adjust_processed_index_to_next_swap_step_base_start_index(processed_index)?;
        let referral_fee_index = start_account_index
            .safe_add(self.get_base_account_length())?
            .safe_add(1)?;

        let referral_fee_account_meta =
            must_retrieve_account_meta(zap_out_instruction, referral_fee_index)?;

        if !is_placeholder_account(&referral_fee_account_meta.key) {
            return Err(ProtocolZapError::ReferralFeeNotAllowed);
        }

        Ok(())
    }

    fn get_end_account_index<'a>(
        &self,
        processed_index: usize,
        _zap_out_instruction: &'a IntrospectedInstruction<'a>,
    ) -> Result<usize, ProtocolZapError> {
        let start_account_index =
            adjust_processed_index_to_next_swap_step_base_start_index(processed_index)?;
        let end_account_index = start_account_index
            .safe_add(self.get_base_account_length())?
            .safe_add(Self::REMAINING_ACCOUNTS_LENGTH)?;

        Ok(end_account_index)
    }
}
