use crate::{
    error::ProtocolZapError, jup_swap_step_referral_fee_parser::SwapStepReferralFeeParser,
};
use pinocchio::{pubkey::Pubkey, sysvars::instructions::IntrospectedInstruction};
use pinocchio_pubkey::from_str;

pub const ID: Pubkey = from_str("CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK");
pub const PANCAKE_SWAP_ID: Pubkey = from_str("HpNfyc2Saw7RKkQd8nEL4khUcuPhQ7WwY1B2qjx8jxFq");
pub const BYREAL_ID: Pubkey = from_str("REALQqNEomY6cQGZJUGwywTBD2UmDT32rZcNnfxQ5N2");
pub const STABBLE_ID: Pubkey = from_str("6dMXqGZ3ga2dikrYS9ovDXgHGh5RUsb2RTUj6hrQXhk6");

pub struct RaydiumClmm;

impl RaydiumClmm {
    pub const BASE_ACCOUNT_LENGTH: usize = 10;
}

impl SwapStepReferralFeeParser for RaydiumClmm {
    fn get_base_account_length(&self) -> usize {
        Self::BASE_ACCOUNT_LENGTH
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

impl RaydiumClmmV2 {
    pub const BASE_ACCOUNT_LENGTH: usize = 13;
}

impl SwapStepReferralFeeParser for RaydiumClmmV2 {
    fn get_base_account_length(&self) -> usize {
        Self::BASE_ACCOUNT_LENGTH
    }

    fn get_end_account_index<'a>(
        &self,
        processed_index: usize,
        zap_out_instruction: &'a IntrospectedInstruction<'a>,
    ) -> Result<usize, ProtocolZapError> {
        self.get_end_account_index_via_placeholder(processed_index, zap_out_instruction)
    }
}
