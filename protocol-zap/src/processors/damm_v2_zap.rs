use crate::{
    constants::{
        DAMM_V2_SWAP_AMOUNT_IN_OFFSET, DAMM_V2_SWAP_DESTINATION_ACCOUNT_INDEX,
        DAMM_V2_SWAP_SOURCE_ACCOUNT_INDEX,
    },
    error::ProtozolZapError,
    RawZapOutAmmInfo, ZapInfoProcessor, ZapOutParameters,
};

pub struct ZapDammV2InfoProcessor;

impl ZapInfoProcessor for ZapDammV2InfoProcessor {
    fn validate_payload(&self, _payload: &[u8]) -> Result<(), ProtozolZapError> {
        Ok(())
    }

    fn extract_raw_zap_out_amm_info(
        &self,
        _zap_params: &ZapOutParameters,
    ) -> Result<RawZapOutAmmInfo, ProtozolZapError> {
        Ok(RawZapOutAmmInfo {
            source_index: DAMM_V2_SWAP_SOURCE_ACCOUNT_INDEX,
            destination_index: DAMM_V2_SWAP_DESTINATION_ACCOUNT_INDEX,
            amount_in_offset: DAMM_V2_SWAP_AMOUNT_IN_OFFSET,
        })
    }
}
