use crate::{
    error::ProtocolZapError, jup_swap_step_referral_fee_parser::SwapStepReferralFeeParser,
};
use pinocchio::{pubkey::Pubkey, sysvars::instructions::IntrospectedInstruction};
use pinocchio_pubkey::from_str;

pub const ID: Pubkey = from_str("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8");

pub struct Raydium;

impl Raydium {
    pub const BASE_ACCOUNT_LENGTH: usize = 17;
}

impl SwapStepReferralFeeParser for Raydium {
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

pub struct RaydiumV2;

impl RaydiumV2 {
    pub const BASE_ACCOUNT_LENGTH: usize = 8;
}

impl SwapStepReferralFeeParser for RaydiumV2 {
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
