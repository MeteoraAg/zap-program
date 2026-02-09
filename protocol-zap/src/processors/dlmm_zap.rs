use crate::{
    constants::{
        DLMM_SWAP2_AMOUNT_IN_OFFSET, DLMM_SWAP2_DESTINATION_ACCOUNT_INDEX,
        DLMM_SWAP2_SOURCE_ACCOUNT_INDEX,
    },
    error::ProtozolZapError,
    RawZapOutAmmInfo, ZapInfoProcessor, ZapOutParameters,
};

pub struct ZapDlmmInfoProcessor;

impl ZapInfoProcessor for ZapDlmmInfoProcessor {
    fn validate_payload(&self, _payload: &[u8]) -> Result<(), ProtozolZapError> {
        Ok(())
    }

    fn extract_raw_zap_out_amm_info(
        &self,
        _zap_params: &ZapOutParameters,
    ) -> Result<RawZapOutAmmInfo, ProtozolZapError> {
        Ok(RawZapOutAmmInfo {
            source_index: DLMM_SWAP2_SOURCE_ACCOUNT_INDEX,
            destination_index: DLMM_SWAP2_DESTINATION_ACCOUNT_INDEX,
            amount_in_offset: DLMM_SWAP2_AMOUNT_IN_OFFSET,
        })
    }
}
