use anchor_lang::prelude::*;

use crate::{
    constants::{DLMM_PAYLOAD_DATA_OFFSET, VEC_DATA_START_OFFSET, VEC_LENGTH_PREFIX_SIZE},
    error::ZapError,
    safe_math::SafeMath,
    ZapOutParametersDecoder,
};

use dlmm::types::{AccountsType, RemainingAccountsInfo, RemainingAccountsSlice};

#[derive(Clone, Debug)]
pub struct SwapDlmmParams {
    pub minimum_amount_out: u64,
    pub remaining_accounts_info: dlmm::types::RemainingAccountsInfo,
}

impl ZapOutParametersDecoder for RemainingAccountsInfo {
    fn decode(payload_data: Vec<u8>) -> Result<Self> {
        // Anchor serializes Vec<T> with a 4 bytes length prefix (u32)
        // Format: [length: u32][item1][item2][item3]...
        // Example: Vec with 2 items => [2, 0, 0, 0][item1_data][item2_data]
        let slice_count = u32::from_le_bytes(
            payload_data[..VEC_LENGTH_PREFIX_SIZE]
                .try_into()
                .map_err(|_| ZapError::TypeCastFailed)?,
        );

        if slice_count == 0 {
            return Ok(RemainingAccountsInfo { slices: vec![] });
        }

        let slice_count_usize: usize = slice_count
            .try_into()
            .map_err(|_| ZapError::TypeCastFailed)?;

        // Parsing each slice: [accounts_type: u8][length: u8]
        let mut slices = Vec::with_capacity(slice_count_usize);
        let slices_data = &payload_data[VEC_DATA_START_OFFSET..].to_vec();
        let mut index = 0;
        for _ in 0..slice_count_usize {
            let accounts_type = decode_account_type(slices_data[index])?;
            let length_index = index.safe_add(1)?;
            let length = slices_data[length_index];

            slices.push(RemainingAccountsSlice {
                accounts_type,
                length,
            });
            index = index.safe_add(1)?;
        }

        Ok(RemainingAccountsInfo { slices })
    }
}

impl ZapOutParametersDecoder for SwapDlmmParams {
    fn decode(payload_data: Vec<u8>) -> Result<Self> {
        // the first 8 bytes of payload data for minimum_amount_out
        let minimum_amount_out = u64::from_le_bytes(
            payload_data[..DLMM_PAYLOAD_DATA_OFFSET]
                .try_into()
                .map_err(|_| ZapError::TypeCastFailed)?,
        );

        let remaining_accounts_info = RemainingAccountsInfo::decode(
            payload_data[DLMM_PAYLOAD_DATA_OFFSET..]
                .try_into()
                .map_err(|_| ZapError::TypeCastFailed)?,
        )?;

        Ok(SwapDlmmParams {
            minimum_amount_out,
            remaining_accounts_info,
        })
    }
}

fn decode_account_type(value: u8) -> Result<AccountsType> {
    match value {
        0 => Ok(AccountsType::TransferHookX),
        1 => Ok(AccountsType::TransferHookY),
        2 => Ok(AccountsType::TransferHookReward),
        _ => Err(ZapError::TypeCastFailed.into()),
    }
}
