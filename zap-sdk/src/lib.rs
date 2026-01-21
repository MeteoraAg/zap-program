use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::entrypoint::ProgramResult;
pub mod constants;
use constants::*;
pub mod damm_v2_zap;
use damm_v2_zap::*;
pub mod dlmm_zap;
use dlmm_zap::ZapDlmmInfoProcessor;
use solana_program_error::ProgramError;
use solana_pubkey::Pubkey;

use crate::{
    error::ZapSdkError,
    jup_v6_zap::{ZapJupV6RouteInfoProcessor, ZapJupV6SharedRouteInfoProcessor},
};
pub mod error;
pub mod jup_v6_zap;
pub mod safe_math;
pub mod tests;
pub mod utils;

#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug)]
pub struct ZapOutParameters {
    pub percentage: u8,
    pub offset_amount_in: u16,
    pub pre_user_token_balance: u64,
    pub max_swap_amount: u64, // avoid the issue someone send token to user token account when user zap out
    pub payload_data: Vec<u8>,
}

pub struct RawZapOutAmmInfo {
    source_index: usize,
    destination_index: usize,
    amount_in_offset: u16,
}

pub trait ZapInfoProcessor {
    fn validate_payload(&self, payload: &[u8]) -> ProgramResult;
    fn extract_raw_zap_out_amm_info(
        &self,
        zap_params: &ZapOutParameters,
    ) -> Result<RawZapOutAmmInfo, ProgramError>;
}

const DAMM_V2_SWAP_DISC_REF: &[u8] = &DAMM_V2_SWAP_DISC;
const DLMM_SWAP2_DISC_REF: &[u8] = &DLMM_SWAP2_DISC;
const JUP_V6_ROUTE_DISC_REF: &[u8] = &JUP_V6_ROUTE_DISC;
const JUP_V6_SHARED_ACCOUNT_ROUTE_DISC_REF: &[u8] = &JUP_V6_SHARED_ACCOUNT_ROUTE_DISC;

pub fn get_zap_amm_processor(
    amm_disc: &[u8],
    amm_program_address: Pubkey,
) -> Result<Box<dyn ZapInfoProcessor>, ProgramError> {
    match (amm_disc, amm_program_address) {
        (DLMM_SWAP2_DISC_REF, DLMM) => Ok(Box::new(ZapDlmmInfoProcessor)),
        (DAMM_V2_SWAP_DISC_REF, DAMM_V2) => Ok(Box::new(ZapDammV2InfoProcessor)),
        (JUP_V6_ROUTE_DISC_REF, JUP_V6) => Ok(Box::new(ZapJupV6RouteInfoProcessor)),
        (JUP_V6_SHARED_ACCOUNT_ROUTE_DISC_REF, JUP_V6) => {
            Ok(Box::new(ZapJupV6SharedRouteInfoProcessor))
        }
        _ => Err(ZapSdkError::InvalidZapOutParameters.into()),
    }
}
