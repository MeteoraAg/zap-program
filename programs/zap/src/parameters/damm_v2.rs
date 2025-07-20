use anchor_lang::prelude::*;

use crate::{constants::DAMM_V2_SWAP_DATA_PAYLOAD_LEN, error::ZapError, ZapOutParametersDecoder};

#[derive(Clone, Debug)]
pub struct SwapDammV2Params {
    pub minimum_amount_out: u64,
}

impl ZapOutParametersDecoder for SwapDammV2Params {
    fn decode(payload_data: Vec<u8>) -> Result<Self> {
        require!(
            payload_data.len() == DAMM_V2_SWAP_DATA_PAYLOAD_LEN,
            ZapError::InvalidDataLength
        );
        let minimum_amount_out = u64::from_le_bytes(
            payload_data
                .try_into()
                .map_err(|_| ZapError::TypeCastFailed)?,
        );
        Ok(SwapDammV2Params { minimum_amount_out })
    }
}
