use crate::constants::{SOL_ADDRESS, USDC_ADDRESS};
use crate::error::ZapSdkError;
use crate::safe_math::SafeMath;
use crate::{constants, get_zap_amm_processor, RawZapOutAmmInfo, ZapOutParameters};
use borsh::BorshDeserialize;
use solana_account_info::AccountInfo;
use solana_instruction::AccountMeta;
use solana_program::sysvar::instructions::{
    load_current_index_checked, load_instruction_at_checked,
};
use solana_program_error::{ProgramError, ProgramResult};
use solana_pubkey::Pubkey;
use spl_associated_token_account::get_associated_token_address;

fn validate_zap_parameters<'info>(
    zap_params: &ZapOutParameters,
    max_claim_amount: u64,
    amount_in_offset: u16,
    claimer_token_account: &AccountInfo<'info>,
) -> ProgramResult {
    if zap_params.percentage != 100 {
        return Err(ZapSdkError::InvalidZapOutParameters.into());
    }

    if zap_params.offset_amount_in != amount_in_offset {
        return Err(ZapSdkError::InvalidZapOutParameters.into());
    }

    // Ensure no stealing from operator by setting a higher pre_token_balance than actual balance to steal fund
    // Eg: Operator set 100 pre balance, but actual balance is 0
    // Actual claimed amount is 300
    // Zap will attempt to swap post - pre = 300 - 100 = 200
    // Leftover 100 will be stolen by operator
    if zap_params.pre_user_token_balance != accessor::amount(claimer_token_account)? {
        return Err(ZapSdkError::InvalidZapOutParameters.into());
    }

    if zap_params.max_swap_amount < max_claim_amount {
        return Err(ZapSdkError::InvalidZapOutParameters.into());
    }

    Ok(())
}

// Search for zap out instruction in the next instruction after the current one
fn search_and_validate_zap_out_instruction<'info>(
    current_index: u16,
    max_claim_amount: u64,
    sysvar_instructions_account: &AccountInfo<'info>,
    claimer_token_account: &AccountInfo<'info>,
    treasury_address: Pubkey,
    treasury_paired_destination_token_address: Pubkey,
) -> ProgramResult {
    // Zap out instruction must be next to current instruction
    let next_index = current_index.safe_add(1)?;
    let ix = load_instruction_at_checked(next_index.into(), sysvar_instructions_account)?;

    if ix.program_id != constants::ZAP {
        return Err(ZapSdkError::MissingZapOutInstruction.into());
    }

    let disc = ix
        .data
        .get(..8)
        .ok_or_else(|| ZapSdkError::InvalidZapOutParameters)?;

    if disc != constants::ZAP_OUT_DISC {
        return Err(ZapSdkError::MissingZapOutInstruction.into());
    }

    let zap_params = ZapOutParameters::try_from_slice(&ix.data[8..])?;

    let ZapOutAmmInfo {
        zap_user_token_in_address,
        amm_source_token_address: source_token_address,
        amm_destination_token_address: destination_token_address,
        amount_in_offset,
    } = extract_amm_accounts_and_info(&zap_params, &ix.accounts)?;

    // Zap out from operator fee receiving account
    validate_zap_parameters(
        &zap_params,
        max_claim_amount,
        amount_in_offset,
        claimer_token_account,
    )?;

    // There's no validation to make sure that `user_token_in_account` is the same as `amm_source_token_address`
    // Operator could steal the fund by providing a fake token account with 0 to bypass the zap swap invoke
    // https://github.com/MeteoraAg/zap-program/blob/117e7d5586aa27cf97e6fde6266e25ee4e496f18/programs/zap/src/instructions/ix_zap_out.rs#L91
    if zap_user_token_in_address != *claimer_token_account.key {
        return Err(ZapSdkError::InvalidZapAccounts.into());
    }

    // Zap out from operator fee receiving account
    if source_token_address != *claimer_token_account.key {
        return Err(ZapSdkError::InvalidZapAccounts.into());
    }

    let treasury_usdc_address = get_associated_token_address(&treasury_address, &USDC_ADDRESS);
    let treasury_sol_address = get_associated_token_address(&treasury_address, &SOL_ADDRESS);

    // Zap to paired mint in the pool, or SOL, or USDC treasury
    if destination_token_address != treasury_paired_destination_token_address
        && destination_token_address != treasury_usdc_address
        && destination_token_address != treasury_sol_address
    {
        return Err(ZapSdkError::InvalidZapAccounts.into());
    }

    Ok(())
}

pub fn validate_zap_out_to_treasury<'info>(
    claimed_amount: u64,
    calling_program_id: Pubkey,
    claimer_token_account: &AccountInfo<'info>,
    sysvar_instructions_account: &AccountInfo<'info>,
    treasury_address: Pubkey,
    treasury_paired_destination_token_address: Pubkey,
) -> ProgramResult {
    let current_index = load_current_index_checked(sysvar_instructions_account)?;

    let current_instruction =
        load_instruction_at_checked(current_index.into(), sysvar_instructions_account)?;

    // Ensure the instruction is direct instruction call
    if current_instruction.program_id != calling_program_id {
        return Err(ZapSdkError::CpiDisabled.into());
    }

    search_and_validate_zap_out_instruction(
        current_index,
        claimed_amount,
        sysvar_instructions_account,
        claimer_token_account,
        treasury_address,
        treasury_paired_destination_token_address,
    )?;

    Ok(())
}

pub struct ZapOutAmmInfo {
    // Account used to compare delta changes with pre_balance to decide swap amount
    pub zap_user_token_in_address: Pubkey,
    pub amm_source_token_address: Pubkey,
    pub amm_destination_token_address: Pubkey,
    pub amount_in_offset: u16,
}

fn extract_amm_accounts_and_info(
    zap_params: &ZapOutParameters,
    zap_account: &[AccountMeta],
) -> Result<ZapOutAmmInfo, ProgramError> {
    // Accounts in ZapOutCtx
    const ZAP_OUT_ACCOUNTS_LEN: usize = 2;

    let zap_user_token_in_address = zap_account
        .get(0)
        .map(|acc| acc.pubkey)
        .ok_or_else(|| ZapSdkError::InvalidZapAccounts)?;

    let zap_amm_program_address = zap_account
        .get(1)
        .map(|acc| acc.pubkey)
        .ok_or_else(|| ZapSdkError::InvalidZapAccounts)?;

    let amm_disc = zap_params
        .payload_data
        .get(..8)
        .ok_or_else(|| ZapSdkError::InvalidZapOutParameters)?;

    let zap_info_processor = get_zap_amm_processor(amm_disc, zap_amm_program_address)?;

    let amm_payload = zap_params
        .payload_data
        .get(8..)
        .ok_or_else(|| ZapSdkError::InvalidZapOutParameters)?;

    zap_info_processor.validate_payload(&amm_payload)?;

    let RawZapOutAmmInfo {
        source_index,
        destination_index,
        amount_in_offset,
    } = zap_info_processor.extract_raw_zap_out_amm_info(zap_params)?;

    // Start from remaining accounts of zap program
    let amm_accounts = zap_account
        .get(ZAP_OUT_ACCOUNTS_LEN..)
        .ok_or_else(|| ZapSdkError::InvalidZapAccounts)?;

    let source_token_address = amm_accounts
        .get(source_index)
        .map(|acc| acc.pubkey)
        .ok_or_else(|| ZapSdkError::InvalidZapAccounts)?;

    let destination_token_address = amm_accounts
        .get(destination_index)
        .map(|acc| acc.pubkey)
        .ok_or_else(|| ZapSdkError::InvalidZapAccounts)?;

    Ok(ZapOutAmmInfo {
        zap_user_token_in_address,
        amm_source_token_address: source_token_address,
        amm_destination_token_address: destination_token_address,
        amount_in_offset,
    })
}

mod accessor {
    use solana_account_info::AccountInfo;
    use solana_program_error::ProgramError;

    pub fn amount(account: &AccountInfo) -> Result<u64, ProgramError> {
        let bytes = account.try_borrow_data()?;
        let mut amount_bytes = [0u8; 8];
        amount_bytes.copy_from_slice(&bytes[64..72]);
        Ok(u64::from_le_bytes(amount_bytes))
    }
}
