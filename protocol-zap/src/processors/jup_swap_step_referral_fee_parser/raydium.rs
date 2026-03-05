use crate::{
    error::ProtocolZapError, jup_swap_step_referral_fee_parser::SwapStepReferralFeeParser,
};
use pinocchio::sysvars::instructions::IntrospectedInstruction;

pub struct Raydium;

impl SwapStepReferralFeeParser for Raydium {
    fn get_base_account_length(&self) -> usize {
        17
    }

    fn get_end_account_index<'a>(
        &self,
        processed_index: usize,
        _zap_out_instruction: &'a IntrospectedInstruction<'a>,
    ) -> Result<usize, ProtocolZapError> {
        self.get_end_account_index_default(processed_index)
    }
}

pub struct RaydiumV2;

impl SwapStepReferralFeeParser for RaydiumV2 {
    fn get_base_account_length(&self) -> usize {
        8
    }

    fn get_end_account_index<'a>(
        &self,
        processed_index: usize,
        _zap_out_instruction: &'a IntrospectedInstruction<'a>,
    ) -> Result<usize, ProtocolZapError> {
        self.get_end_account_index_default(processed_index)
    }
}
