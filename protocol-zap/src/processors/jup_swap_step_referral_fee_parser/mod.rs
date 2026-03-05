use crate::{
    error::ProtocolZapError,
    jup_swap_step_referral_fee_parser::{
        mercurial::Mercurial,
        meteora_damm_v1::Meteora,
        meteora_damm_v2::{MeteoraDammV2, MeteoraDammV2WithRemainingAccounts},
        meteora_dlmm::{MeteoraDLMM, MeteoraDLMMSwapV2},
        raydium::{Raydium, RaydiumV2},
        raydium_clmm::{RaydiumClmm, RaydiumClmmV2},
        raydium_cp::RaydiumCp,
        whirlpool::{Whirlpool, WhirlpoolSwapV2},
    },
    safe_math::SafeMath,
    WhitelistedSwapStep,
};
use jupiter::types::RoutePlanStep;
use pinocchio::{
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::instructions::{IntrospectedAccountMeta, IntrospectedInstruction},
};
use pinocchio_pubkey::from_str;

pub mod mercurial;
pub mod meteora_damm_v1;
pub mod meteora_damm_v2;
pub mod meteora_dlmm;
pub mod raydium;
pub mod raydium_clmm;
pub mod raydium_cp;
pub mod whirlpool;

// The first account of each swap step always will be the program account, so the minimum length is 1
const PROGRAM_ACCOUNT_LENGTH: usize = 1;

pub trait SwapStepReferralFeeParser {
    /// Default implementation has no referral fee.
    fn ensure_no_referral_fee_account<'a>(
        &self,
        _processed_index: usize,
        _zap_out_instruction: &'a IntrospectedInstruction<'a>,
    ) -> Result<(), ProtocolZapError> {
        Ok(())
    }

    /// Return the end account index (inclusive) of the swap step in the instruction accounts
    fn get_end_account_index<'a>(
        &self,
        processed_index: usize,
        zap_out_instruction: &'a IntrospectedInstruction<'a>,
    ) -> Result<usize, ProtocolZapError>;

    /// Return the number of necessary accounts passed into AMM during swap. In anchor based project, it's the accounts in the `Context`
    fn get_base_account_length(&self) -> usize;

    /// Default: end = start + base_len (no remaining accounts)
    fn get_end_account_index_default(
        &self,
        processed_index: usize,
    ) -> Result<usize, ProtocolZapError> {
        let start = adjust_processed_index_to_next_swap_step_base_start_index(processed_index)?;
        start.safe_add(self.get_base_account_length())
    }

    /// end = start + base_len, then scan forward for the next placeholder account
    fn get_end_account_index_via_placeholder<'a>(
        &self,
        processed_index: usize,
        zap_out_instruction: &'a IntrospectedInstruction<'a>,
    ) -> Result<usize, ProtocolZapError> {
        let start = adjust_processed_index_to_next_swap_step_base_start_index(processed_index)?;
        let end_base = start.safe_add(self.get_base_account_length())?;
        find_next_placeholder_account_index(zap_out_instruction, end_base)
    }

    fn load_next_swap_step(
        &mut self,
        _next_swap_step: Option<&RoutePlanStep>,
    ) -> Result<(), ProtocolZapError> {
        Ok(())
    }
}

// Similar as IntrospectedInstruction::get_account_meta_at but return None if account index is out of bounds instead of returning error
fn get_account_meta<'a>(
    zap_out_instruction: &'a IntrospectedInstruction<'a>,
    account_index: usize,
) -> Result<Option<&'a IntrospectedAccountMeta>, ProtocolZapError> {
    match zap_out_instruction.get_account_meta_at(account_index) {
        Ok(account) => Ok(Some(account)),
        Err(err) => {
            if err == ProgramError::InvalidArgument {
                // Account index is out of bounds
                Ok(None)
            } else {
                // Unexpected error, propagate it
                Err(ProtocolZapError::UndeterminedError)
            }
        }
    }
}

#[inline(always)]
fn must_retrieve_account_meta<'a>(
    zap_out_instruction: &'a IntrospectedInstruction<'a>,
    account_index: usize,
) -> Result<&'a IntrospectedAccountMeta, ProtocolZapError> {
    zap_out_instruction
        .get_account_meta_at(account_index)
        .map_err(|_| ProtocolZapError::InvalidZapAccounts)
}

#[inline(always)]
fn is_placeholder_account(key: &[u8; 32]) -> bool {
    key.eq(jupiter::ID_CONST.as_array())
}

#[inline(always)]
fn adjust_processed_index_to_next_swap_step_base_start_index(
    processed_index: usize,
) -> Result<usize, ProtocolZapError> {
    // Processed index is end account index (inclusive) of the previous swap step instruction account.
    processed_index
        .safe_add(1)?
        .safe_add(PROGRAM_ACCOUNT_LENGTH)
}

fn find_next_placeholder_account_index<'a>(
    zap_out_instruction: &'a IntrospectedInstruction<'a>,
    processed_index: usize,
) -> Result<usize, ProtocolZapError> {
    let mut current_index = processed_index.safe_add(1)?;
    loop {
        let account_meta = get_account_meta(zap_out_instruction, current_index)?;

        match account_meta {
            Some(meta) => {
                if is_placeholder_account(&meta.key) {
                    return Ok(current_index);
                } else {
                    current_index = current_index.safe_add(1)?;
                }
            }
            None => return Err(ProtocolZapError::InvalidZapAccounts),
        }
    }
}

fn get_swap_step_program_addresses(swap_step: &WhitelistedSwapStep) -> &'static [Pubkey] {
    const MERCURIAL: [Pubkey; 1] = [from_str("MERLuDFBMmsHnsBPZw2sDQZHvXFMwp8EdjudcU2HKky")];
    const METEORA: [Pubkey; 1] = [from_str("Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EQVn5UaB")];
    const METEORA_DAMM_V2: [Pubkey; 1] = [from_str("cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG")];
    const METEORA_DLMM: [Pubkey; 1] = [from_str("LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo")];
    const WHIRLPOOL: [Pubkey; 2] = [
        // whirlpool
        from_str("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc"),
        // cropper
        from_str("H8W3ctz92svYg6mkn1UtGfu2aQr2fnUFHM1RhScEtQDt"),
    ];
    const WHIRLPOOL_V2: [Pubkey; 1] = [from_str("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc")];
    const RAYDIUM: [Pubkey; 1] = [from_str("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8")];
    const RAYDIUM_CP: [Pubkey; 1] = [from_str("CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C")];
    const RAYDIUM_CLMM: [Pubkey; 4] = [
        // Raydium CLMM
        from_str("CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK"),
        // Pancake
        from_str("HpNfyc2Saw7RKkQd8nEL4khUcuPhQ7WwY1B2qjx8jxFq"),
        // ByReal
        from_str("REALQqNEomY6cQGZJUGwywTBD2UmDT32rZcNnfxQ5N2"),
        // Stabble
        from_str("6dMXqGZ3ga2dikrYS9ovDXgHGh5RUsb2RTUj6hrQXhk6"),
    ];

    match swap_step {
        WhitelistedSwapStep::Mercurial => &MERCURIAL,
        WhitelistedSwapStep::Meteora => &METEORA,
        WhitelistedSwapStep::MeteoraDammV2
        | WhitelistedSwapStep::MeteoraDammV2WithRemainingAccounts => &METEORA_DAMM_V2,
        WhitelistedSwapStep::MeteoraDlmm | WhitelistedSwapStep::MeteoraDlmmSwapV2 => &METEORA_DLMM,
        WhitelistedSwapStep::Whirlpool => &WHIRLPOOL,
        WhitelistedSwapStep::WhirlpoolSwapV2 { .. } => &WHIRLPOOL_V2,
        WhitelistedSwapStep::Raydium | WhitelistedSwapStep::RaydiumV2 => &RAYDIUM,
        WhitelistedSwapStep::RaydiumCP => &RAYDIUM_CP,
        WhitelistedSwapStep::RaydiumClmm | WhitelistedSwapStep::RaydiumClmmV2 => &RAYDIUM_CLMM,
    }
}

fn find_next_swap_step_program_account_index<'a>(
    zap_out_instruction: &'a IntrospectedInstruction<'a>,
    processed_index: usize,
    swap_step: &WhitelistedSwapStep,
) -> Result<usize, ProtocolZapError> {
    let mut current_index = processed_index.safe_add(1)?;
    let program_addresses = get_swap_step_program_addresses(swap_step);

    loop {
        let account_meta = get_account_meta(zap_out_instruction, current_index)?;

        match account_meta {
            Some(meta) => {
                if program_addresses.iter().any(|address| meta.key.eq(address)) {
                    return Ok(current_index);
                } else {
                    current_index = current_index.safe_add(1)?;
                }
            }
            None => return Err(ProtocolZapError::InvalidZapAccounts),
        }
    }
}

pub fn get_referral_fee_parser(
    swap_step: &WhitelistedSwapStep,
) -> Box<dyn SwapStepReferralFeeParser> {
    match swap_step {
        WhitelistedSwapStep::Meteora => Box::new(Meteora),
        WhitelistedSwapStep::MeteoraDammV2 => Box::new(MeteoraDammV2),
        WhitelistedSwapStep::MeteoraDammV2WithRemainingAccounts => {
            Box::new(MeteoraDammV2WithRemainingAccounts)
        }
        WhitelistedSwapStep::MeteoraDlmm => Box::new(MeteoraDLMM),
        WhitelistedSwapStep::MeteoraDlmmSwapV2 => Box::new(MeteoraDLMMSwapV2),
        WhitelistedSwapStep::Whirlpool => Box::new(Whirlpool),
        WhitelistedSwapStep::WhirlpoolSwapV2 {
            remaining_accounts_info,
        } => Box::new(WhirlpoolSwapV2 {
            remaining_accounts_info: remaining_accounts_info.clone(),
        }),
        WhitelistedSwapStep::Mercurial => Box::new(Mercurial::default()),
        WhitelistedSwapStep::Raydium => Box::new(Raydium),
        WhitelistedSwapStep::RaydiumV2 => Box::new(RaydiumV2),
        WhitelistedSwapStep::RaydiumCP => Box::new(RaydiumCp),
        WhitelistedSwapStep::RaydiumClmm => Box::new(RaydiumClmm),
        WhitelistedSwapStep::RaydiumClmmV2 => Box::new(RaydiumClmmV2),
    }
}
