use pinocchio::sysvars::instructions::IntrospectedInstruction;

use crate::{
    constants::{
        DLMM_SWAP2_AMOUNT_IN_OFFSET, DLMM_SWAP2_DESTINATION_ACCOUNT_INDEX,
        DLMM_SWAP2_REFERRAL_FEE_ACCOUNT_INDEX, DLMM_SWAP2_SOURCE_ACCOUNT_INDEX,
    },
    error::ProtocolZapError,
    utils::get_account_index_in_instruction,
    RawZapOutAmmInfo, ZapInfoProcessor, ZapOutParameters,
};

pub struct ZapDlmmInfoProcessor;

impl ZapInfoProcessor for ZapDlmmInfoProcessor {
    fn validate_payload(&self) -> Result<(), ProtocolZapError> {
        Ok(())
    }

    fn extract_raw_zap_out_amm_info(
        &self,
        _zap_params: &ZapOutParameters,
    ) -> Result<RawZapOutAmmInfo, ProtocolZapError> {
        Ok(RawZapOutAmmInfo {
            source_index: DLMM_SWAP2_SOURCE_ACCOUNT_INDEX,
            destination_index: DLMM_SWAP2_DESTINATION_ACCOUNT_INDEX,
            amount_in_offset: DLMM_SWAP2_AMOUNT_IN_OFFSET,
        })
    }

    fn validate_route_plan(
        &self,
        zap_out_instruction: &IntrospectedInstruction<'_>,
    ) -> Result<(), ProtocolZapError> {
        let referral_token_account_index =
            get_account_index_in_instruction(DLMM_SWAP2_REFERRAL_FEE_ACCOUNT_INDEX)?;

        let referral_token_account = zap_out_instruction
            .get_account_meta_at(referral_token_account_index)
            .map_err(|_| ProtocolZapError::InvalidZapAccounts)?;

        if referral_token_account.key != zap_sdk::constants::DLMM.to_bytes() {
            return Err(ProtocolZapError::ReferralFeeNotAllowed);
        }

        Ok(())
    }
}
