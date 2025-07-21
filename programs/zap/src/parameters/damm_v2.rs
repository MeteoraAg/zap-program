use std::mem::size_of;

use anchor_lang::prelude::*;

use crate::error::ZapError;

const DAMM_V2_SWAP_DATA_PAYLOAD_LEN: usize = 8;

#[derive(Clone, Debug)]
pub struct SwapDammV2Params {
    pub minimum_amount_out: u64,
}

impl SwapDammV2Params {
    pub fn unpack(payload_data: &[u8]) -> Result<Self> {
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

    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        buf.extend_from_slice(&self.minimum_amount_out.to_le_bytes());

        buf
    }
}
