use std::cmp::min;

use anchor_lang::prelude::*;

pub mod constants;
use constants::*;
pub mod damm_v2_zap;
use damm_v2_zap::*;
pub mod dlmm_zap;
use dlmm_zap::ZapDlmmInfoProcessor;

use crate::{error::ProtocolZapError, jup_v6_zap::ZapJupV6RouteInfoProcessor, safe_math::SafeMath};
pub mod error;
pub mod jup_v6_zap;
pub mod safe_math;
pub mod tests;
pub mod utils;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ZapOutParameters {
    pub percentage: u8,
    pub offset_amount_in: u16,
    pub pre_user_token_balance: u64,
    pub max_swap_amount: u64, // avoid the issue someone send token to user token account when user zap out
    pub payload_data: Vec<u8>,
}

impl ZapOutParameters {
    pub fn validate(&self) -> Result<()> {
        require!(
            self.percentage <= 100 && self.percentage > 0,
            ProtocolZapError::InvalidZapOutParameters
        );

        Ok(())
    }

    pub fn get_swap_amount(&self, balance_change_amount: u64) -> Result<u64> {
        let swap_amount = if self.percentage == 100 {
            balance_change_amount
        } else {
            let amount = u128::from(balance_change_amount)
                .safe_mul(self.percentage.into())?
                .safe_div(100)?;
            u64::try_from(amount).map_err(|_| ProtocolZapError::TypeCastFailed)?
        };

        Ok(min(swap_amount, self.max_swap_amount))
    }
}

pub struct RawZapOutAmmInfo {
    source_index: usize,
    destination_index: usize,
    amount_in_offset: u16,
}

pub trait ZapInfoProcessor {
    fn validate_payload(&self, payload: &[u8]) -> Result<()>;
    fn extract_raw_zap_out_amm_info(
        &self,
        zap_params: &ZapOutParameters,
    ) -> Result<RawZapOutAmmInfo>;
}

const DAMM_V2_SWAP_DISC_REF: &[u8] = &DAMM_V2_SWAP_DISC;
const DLMM_SWAP2_DISC_REF: &[u8] = &DLMM_SWAP2_DISC;
const JUP_V6_ROUTE_DISC_REF: &[u8] = &JUP_V6_ROUTE_DISC;
const JUP_V6_SHARED_ACCOUNT_ROUTE_DISC_REF: &[u8] = &JUP_V6_SHARED_ACCOUNT_ROUTE_DISC;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZapAmmProgram {
    Dlmm,
    DammV2,
    JupV6Route,
    JupV6SharedAccountRoute,
}

impl ZapAmmProgram {
    pub fn try_from_raw(disc: &[u8], program: Pubkey) -> Option<Self> {
        match (disc, program) {
            (DLMM_SWAP2_DISC_REF, DLMM) => Some(Self::Dlmm),
            (DAMM_V2_SWAP_DISC_REF, DAMM_V2) => Some(Self::DammV2),
            (JUP_V6_ROUTE_DISC_REF, JUP_V6) => Some(Self::JupV6Route),
            (JUP_V6_SHARED_ACCOUNT_ROUTE_DISC_REF, JUP_V6) => Some(Self::JupV6SharedAccountRoute),
            _ => None,
        }
    }

    pub fn get_processor(&self) -> Box<dyn ZapInfoProcessor> {
        match self {
            Self::Dlmm => Box::new(ZapDlmmInfoProcessor),
            Self::DammV2 => Box::new(ZapDammV2InfoProcessor),
            Self::JupV6Route | Self::JupV6SharedAccountRoute => {
                Box::new(ZapJupV6RouteInfoProcessor)
            }
        }
    }
}
