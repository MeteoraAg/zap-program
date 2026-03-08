use crate::{
    error::ProtocolZapError, jup_swap_step_referral_fee_parser::SwapStepReferralFeeParser,
};
use pinocchio::{pubkey::Pubkey, sysvars::instructions::IntrospectedInstruction};
use pinocchio_pubkey::from_str;

pub const ID: Pubkey = from_str("CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C");

pub struct RaydiumCp;

impl RaydiumCp {
    pub const BASE_ACCOUNT_LENGTH: usize = 13;
}

impl SwapStepReferralFeeParser for RaydiumCp {
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
