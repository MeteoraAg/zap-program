use crate::{
    error::ProtocolZapError, jup_swap_step_referral_fee_parser::SwapStepReferralFeeParser,
    safe_math::SafeMath,
};
use jupiter::types::RemainingAccountsInfo;
use pinocchio::{pubkey::Pubkey, sysvars::instructions::IntrospectedInstruction};
use pinocchio_pubkey::from_str;

pub const ID: Pubkey = from_str("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc");
pub const CROPPER_ID: Pubkey = from_str("H8W3ctz92svYg6mkn1UtGfu2aQr2fnUFHM1RhScEtQDt");

// Orca Whirlpool
pub struct Whirlpool;

impl Whirlpool {
    pub const BASE_ACCOUNT_LENGTH: usize = 11;
}

impl SwapStepReferralFeeParser for Whirlpool {
    fn get_base_account_length(&self) -> usize {
        Self::BASE_ACCOUNT_LENGTH
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

impl WhirlpoolSwapV2 {
    pub const BASE_ACCOUNT_LENGTH: usize = 15;
}

impl SwapStepReferralFeeParser for WhirlpoolSwapV2 {
    fn get_base_account_length(&self) -> usize {
        Self::BASE_ACCOUNT_LENGTH
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
