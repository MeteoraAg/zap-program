use crate::{
    error::ProtocolZapError, jup_swap_step_referral_fee_parser::SwapStepReferralFeeParser,
};
use pinocchio::sysvars::instructions::IntrospectedInstruction;

pub struct RaydiumClmm;

impl SwapStepReferralFeeParser for RaydiumClmm {
    fn get_base_account_length(&self) -> usize {
        10
    }

    fn get_end_account_index<'a>(
        &self,
        processed_index: usize,
        zap_out_instruction: &'a IntrospectedInstruction<'a>,
    ) -> Result<usize, ProtocolZapError> {
        self.get_end_account_index_via_placeholder(processed_index, zap_out_instruction)
    }
}

pub struct RaydiumClmmV2;

impl SwapStepReferralFeeParser for RaydiumClmmV2 {
    fn get_base_account_length(&self) -> usize {
        13
    }

    fn get_end_account_index<'a>(
        &self,
        processed_index: usize,
        zap_out_instruction: &'a IntrospectedInstruction<'a>,
    ) -> Result<usize, ProtocolZapError> {
        self.get_end_account_index_via_placeholder(processed_index, zap_out_instruction)
    }
}
