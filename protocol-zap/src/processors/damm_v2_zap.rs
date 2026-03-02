use pinocchio::sysvars::instructions::IntrospectedInstruction;

use crate::{
    constants::{
        DAMM_V2_SWAP_AMOUNT_IN_OFFSET, DAMM_V2_SWAP_DESTINATION_ACCOUNT_INDEX,
        DAMM_V2_SWAP_REFERRAL_FEE_ACCOUNT_INDEX, DAMM_V2_SWAP_SOURCE_ACCOUNT_INDEX,
    },
    error::ProtocolZapError,
    utils::get_account_index_in_instruction,
    RawZapOutAmmInfo, ZapInfoProcessor, ZapOutParameters,
};

pub struct ZapDammV2InfoProcessor;

impl ZapInfoProcessor for ZapDammV2InfoProcessor {
    fn validate_payload(&self) -> Result<(), ProtocolZapError> {
        Ok(())
    }

    fn extract_raw_zap_out_amm_info(
        &self,
        _zap_params: &ZapOutParameters,
    ) -> Result<RawZapOutAmmInfo, ProtocolZapError> {
        Ok(RawZapOutAmmInfo {
            source_index: DAMM_V2_SWAP_SOURCE_ACCOUNT_INDEX,
            destination_index: DAMM_V2_SWAP_DESTINATION_ACCOUNT_INDEX,
            amount_in_offset: DAMM_V2_SWAP_AMOUNT_IN_OFFSET,
        })
    }

    fn ensure_no_referral_fee(
        &self,
        zap_out_instruction: &IntrospectedInstruction<'_>,
    ) -> Result<(), ProtocolZapError> {
        let referral_token_account_index =
            get_account_index_in_instruction(DAMM_V2_SWAP_REFERRAL_FEE_ACCOUNT_INDEX)?;

        let referral_token_account = zap_out_instruction
            .get_account_meta_at(referral_token_account_index)
            .map_err(|_| ProtocolZapError::InvalidZapAccounts)?;

        if referral_token_account.key != zap_sdk::constants::DAMM_V2.to_bytes() {
            return Err(ProtocolZapError::ReferralFeeNotAllowed);
        }

        Ok(())
    }
}
