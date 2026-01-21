use solana_program::entrypoint_deprecated::ProgramResult;
use solana_program_error::ProgramError;

use crate::{
    constants::{
        DLMM_SWAP2_AMOUNT_IN_OFFSET, DLMM_SWAP2_DESTINATION_ACCOUNT_INDEX,
        DLMM_SWAP2_SOURCE_ACCOUNT_INDEX,
    },
    RawZapOutAmmInfo, ZapInfoProcessor, ZapOutParameters,
};

pub struct ZapDlmmInfoProcessor;

impl ZapInfoProcessor for ZapDlmmInfoProcessor {
    fn validate_payload(&self, _payload: &[u8]) -> ProgramResult {
        Ok(())
    }

    fn extract_raw_zap_out_amm_info(
        &self,
        _zap_params: &ZapOutParameters,
    ) -> Result<RawZapOutAmmInfo, ProgramError> {
        Ok(RawZapOutAmmInfo {
            source_index: DLMM_SWAP2_SOURCE_ACCOUNT_INDEX,
            destination_index: DLMM_SWAP2_DESTINATION_ACCOUNT_INDEX,
            amount_in_offset: DLMM_SWAP2_AMOUNT_IN_OFFSET,
        })
    }
}
