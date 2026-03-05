use crate::{
    error::ProtocolZapError, jup_swap_step_referral_fee_parser::SwapStepReferralFeeParser,
    safe_math::SafeMath,
};
use jupiter::types::RemainingAccountsInfo;
use pinocchio::sysvars::instructions::IntrospectedInstruction;

// Orca Whirlpool
pub struct Whirlpool;

impl SwapStepReferralFeeParser for Whirlpool {
    fn get_base_account_length(&self) -> usize {
        11
    }

    fn get_end_account_index<'a>(
        &self,
        processed_index: usize,
        _zap_out_instruction: &'a IntrospectedInstruction<'a>,
    ) -> Result<usize, ProtocolZapError> {
        self.get_end_account_index_default(processed_index)
    }
}

// Whirlpool SwapV2
pub struct WhirlpoolSwapV2 {
    pub remaining_accounts_info: Option<RemainingAccountsInfo>,
}

impl SwapStepReferralFeeParser for WhirlpoolSwapV2 {
    fn get_base_account_length(&self) -> usize {
        15
    }

    fn get_end_account_index<'a>(
        &self,
        processed_index: usize,
        _zap_out_instruction: &'a IntrospectedInstruction<'a>,
    ) -> Result<usize, ProtocolZapError> {
        let end_base = self.get_end_account_index_default(processed_index)?;

        if let Some(remaining_accounts_info) = &self.remaining_accounts_info {
            let remaining_accounts_length: u8 = remaining_accounts_info
                .slices
                .iter()
                .map(|slice| slice.length)
                .sum();

            Ok(end_base.safe_add(remaining_accounts_length.into())?)
        } else {
            Ok(end_base)
        }
    }
}
