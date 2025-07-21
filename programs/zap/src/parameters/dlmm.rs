use std::mem::size_of;

use anchor_lang::prelude::*;

use crate::error::ZapError;

use dlmm::types::{AccountsType, RemainingAccountsInfo, RemainingAccountsSlice};

const DLMM_PAYLOAD_DATA_OFFSET: usize = 8;
const VEC_DATA_START_OFFSET: usize = 4; // Data starts after length prefix
const VEC_LENGTH_PREFIX_SIZE: usize = 4; // 4 bytes length prefix
const SLICE_SIZE: usize = 2; // accounts_type(1 byte) + length(1 byte)
const MAX_SLICE_ACCOUNTS: usize = 2; // max slice accounts included: TransferHookX, TransferHookY

#[derive(Clone, Debug)]
pub struct SwapDlmmParams {
    pub minimum_amount_out: u64,
    pub remaining_accounts_info: dlmm::types::RemainingAccountsInfo,
}

impl SwapDlmmParams {
    pub fn unpack(payload_data: &[u8]) -> Result<Self> {
        require!(
            payload_data.len() >= DLMM_PAYLOAD_DATA_OFFSET,
            ZapError::InvalidDataLength
        );
        // the first 8 bytes of payload data for minimum_amount_out
        let minimum_amount_out = u64::from_le_bytes(
            payload_data[..DLMM_PAYLOAD_DATA_OFFSET]
                .try_into()
                .map_err(|_| ZapError::TypeCastFailed)?,
        );

        let remaining_accouns_info_data = &payload_data[DLMM_PAYLOAD_DATA_OFFSET..];
        if remaining_accouns_info_data.is_empty() {
            return Ok(SwapDlmmParams {
                minimum_amount_out,
                remaining_accounts_info: RemainingAccountsInfo { slices: vec![] },
            });
        }

        require!(
            remaining_accouns_info_data.len() > VEC_LENGTH_PREFIX_SIZE,
            ZapError::InvalidDataLength
        );

        // Serializes Vec<T> with a 4 bytes length prefix (u32)
        // Format: [length: u32][item1][item2][item3]...
        // Example: Vec with 2 items => [2, 0, 0, 0][item1_data][item2_data]
        let slice_count = u32::from_le_bytes(
            remaining_accouns_info_data[..VEC_LENGTH_PREFIX_SIZE]
                .try_into()
                .map_err(|_| ZapError::TypeCastFailed)?,
        );

        let slice_count_usize: usize = slice_count
            .try_into()
            .map_err(|_| ZapError::TypeCastFailed)?;

        require!(
            slice_count_usize <= MAX_SLICE_ACCOUNTS,
            ZapError::InvalidDataLength
        );

        // Parsing each slice: [accounts_type: u8][length: u8]
        let mut slices = Vec::with_capacity(slice_count_usize);
        let slices_data = &remaining_accouns_info_data[VEC_DATA_START_OFFSET..];

        require!(
            slices_data.len() == slice_count_usize * SLICE_SIZE,
            ZapError::InvalidDataLength
        );

        let mut index = 0;
        for _ in 0..slice_count_usize {
            let accounts_type = decode_account_type(slices_data[index])?;
            let length_index = index + 1;
            let length = slices_data[length_index];

            slices.push(RemainingAccountsSlice {
                accounts_type,
                length,
            });
            index += 2;
        }

        Ok(SwapDlmmParams {
            minimum_amount_out,
            remaining_accounts_info: RemainingAccountsInfo { slices },
        })
    }

    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        buf.extend_from_slice(&self.minimum_amount_out.to_le_bytes());
        let slice_count = self.remaining_accounts_info.slices.len() as u32;
        if slice_count == 0 {
            return buf;
        }

        buf.extend_from_slice(&slice_count.to_le_bytes());

        for slice in &self.remaining_accounts_info.slices {
            let encoded_account_type = encode_account_type(&slice.accounts_type);
            buf.push(encoded_account_type);
            buf.push(slice.length)
        }

        buf
    }
}

pub fn decode_account_type(value: u8) -> Result<AccountsType> {
    match value {
        0 => Ok(AccountsType::TransferHookX),
        1 => Ok(AccountsType::TransferHookY),
        2 => Ok(AccountsType::TransferHookReward),
        _ => Err(ZapError::TypeCastFailed.into()),
    }
}

pub fn encode_account_type(account_type: &AccountsType) -> u8 {
    match account_type {
        AccountsType::TransferHookX => 0,
        AccountsType::TransferHookY => 1,
        AccountsType::TransferHookReward => 2,
    }
}
