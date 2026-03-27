use crate::constants::{SOL_ADDRESS, USDC_ADDRESS};
use crate::error::ProtozolZapError;
use crate::safe_math::SafeMath;
use crate::{constants, get_zap_amm_processor, RawZapOutAmmInfo, ZapOutParameters};
use borsh::BorshDeserialize;
use pinocchio::pubkey::Pubkey;
use pinocchio::sysvars::instructions::{Instructions, IntrospectedInstruction};
mod token;
use token::{get_associated_token_address, get_token_amount};

fn validate_zap_parameters(
    zap_params: &ZapOutParameters,
    max_claim_amount: u64,
    amount_in_offset: u16,
    claimer_token_account_data: &[u8],
) -> Result<(), ProtozolZapError> {
    if zap_params.percentage != 100 {
        return Err(ProtozolZapError::InvalidZapOutParameters);
    }

    if zap_params.offset_amount_in != amount_in_offset {
        return Err(ProtozolZapError::InvalidZapOutParameters);
    }

    // Ensure no stealing from operator by setting a higher pre_token_balance than actual balance to steal fund
    // Eg: Operator set 100 pre balance, but actual balance is 0
    // Actual claimed amount is 300
    // Zap will attempt to swap post - pre = 300 - 100 = 200
    // Leftover 100 will be stolen by operator
    if zap_params.pre_user_token_balance != get_token_amount(claimer_token_account_data)? {
        return Err(ProtozolZapError::InvalidZapOutParameters);
    }

    // Ensure max swap amount is greater than or equal to max claim amount
    // meaning operator must swap at least the amount of the claim amount
    if zap_params.max_swap_amount < max_claim_amount {
        return Err(ProtozolZapError::InvalidZapOutParameters);
    }

    Ok(())
}

// Search for zap out instruction in the next instruction after the current one
fn search_and_validate_zap_out_instruction(
    max_claim_amount: u64,
    sysvar_instructions: &Instructions<&[u8]>,
    claimer_token_account_key: &Pubkey,
    claimer_token_account_data: &[u8],
    treasury_address: &Pubkey,
    treasury_paired_destination_token_address: &Pubkey,
) -> Result<(), ProtozolZapError> {
    // Zap out instruction must be next to current instruction
    let ix = sysvar_instructions
        .get_instruction_relative(1)
        .map_err(|_| ProtozolZapError::MissingZapOutInstruction)?;

    if *ix.get_program_id() != constants::ZAP {
        return Err(ProtozolZapError::MissingZapOutInstruction);
    }

    let disc = ix
        .get_instruction_data()
        .get(..8)
        .ok_or_else(|| ProtozolZapError::InvalidZapOutParameters)?;

    if disc != constants::ZAP_OUT_DISC {
        return Err(ProtozolZapError::MissingZapOutInstruction);
    }

    let zap_params = ZapOutParameters::try_from_slice(&ix.get_instruction_data()[8..])
        .map_err(|_| ProtozolZapError::InvalidZapOutParameters)?;

    let ZapOutAmmInfo {
        zap_user_token_in_address,
        amm_source_token_address: source_token_address,
        amm_destination_token_address: destination_token_address,
        amount_in_offset,
    } = extract_amm_accounts_and_info(&zap_params, ix)?;

    // Zap out from operator fee receiving account
    validate_zap_parameters(
        &zap_params,
        max_claim_amount,
        amount_in_offset,
        claimer_token_account_data,
    )?;

    // There's no validation to make sure that `user_token_in_account` is the same as `amm_source_token_address`
    // Operator could steal the fund by providing a fake token account with 0 to bypass the zap swap invoke
    // https://github.com/MeteoraAg/zap-program/blob/117e7d5586aa27cf97e6fde6266e25ee4e496f18/programs/zap/src/instructions/ix_zap_out.rs#L91
    if zap_user_token_in_address != *claimer_token_account_key {
        return Err(ProtozolZapError::InvalidZapAccounts);
    }

    // Zap out from operator fee receiving account
    if source_token_address != *claimer_token_account_key {
        return Err(ProtozolZapError::InvalidZapAccounts);
    }

    let treasury_usdc_address = get_associated_token_address(&treasury_address, &USDC_ADDRESS);
    let treasury_sol_address = get_associated_token_address(&treasury_address, &SOL_ADDRESS);

    // Zap to paired mint in the pool, or SOL, or USDC treasury
    if destination_token_address != *treasury_paired_destination_token_address
        && destination_token_address != treasury_usdc_address
        && destination_token_address != treasury_sol_address
    {
        return Err(ProtozolZapError::InvalidZapAccounts);
    }

    Ok(())
}

pub fn validate_zap_out_to_treasury(
    claimed_amount: u64,
    calling_program_id: &Pubkey,
    claimer_token_account_key: &Pubkey,
    claimer_token_account_data: &[u8],
    sysvar_instructions_data: &[u8],
    treasury_address: &Pubkey,
    treasury_paired_destination_token_address: &Pubkey,
) -> Result<(), ProtozolZapError> {
    let sysvar_instructions = unsafe { Instructions::new_unchecked(sysvar_instructions_data) };
    let current_index = sysvar_instructions.load_current_index();

    let current_instruction = sysvar_instructions
        .load_instruction_at(current_index.into())
        .map_err(|_| ProtozolZapError::InvalidZapAccounts)?;

    // Ensure the instruction is direct instruction call
    if *current_instruction.get_program_id() != *calling_program_id {
        return Err(ProtozolZapError::CpiDisabled);
    }

    search_and_validate_zap_out_instruction(
        claimed_amount,
        &sysvar_instructions,
        claimer_token_account_key,
        claimer_token_account_data,
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
    zap_in_instruction: IntrospectedInstruction<'_>,
) -> Result<ZapOutAmmInfo, ProtozolZapError> {
    // Accounts in ZapOutCtx
    const ZAP_OUT_ACCOUNTS_LEN: usize = 2;

    let zap_user_token_in_address = zap_in_instruction
        .get_account_meta_at(0)
        .map_err(|_| ProtozolZapError::InvalidZapAccounts)?
        .key;

    let zap_amm_program_address = zap_in_instruction
        .get_account_meta_at(1)
        .map_err(|_| ProtozolZapError::InvalidZapAccounts)?
        .key;

    let amm_disc = zap_params
        .payload_data
        .get(..8)
        .ok_or_else(|| ProtozolZapError::InvalidZapOutParameters)?;

    let zap_info_processor = get_zap_amm_processor(amm_disc, zap_amm_program_address)?;

    let amm_payload = zap_params
        .payload_data
        .get(8..)
        .ok_or_else(|| ProtozolZapError::InvalidZapOutParameters)?;

    zap_info_processor.validate_payload(amm_payload)?;

    let RawZapOutAmmInfo {
        source_index,
        destination_index,
        amount_in_offset,
    } = zap_info_processor.extract_raw_zap_out_amm_info(zap_params)?;

    let offset_source_index = ZAP_OUT_ACCOUNTS_LEN.safe_add(source_index)?;
    let source_token_address = zap_in_instruction
        .get_account_meta_at(offset_source_index)
        .map_err(|_| ProtozolZapError::InvalidZapAccounts)?
        .key;

    let offset_destination_index = ZAP_OUT_ACCOUNTS_LEN.safe_add(destination_index)?;
    let destination_token_address = zap_in_instruction
        .get_account_meta_at(offset_destination_index)
        .map_err(|_| ProtozolZapError::InvalidZapAccounts)?
        .key;

    Ok(ZapOutAmmInfo {
        zap_user_token_in_address,
        amm_source_token_address: source_token_address,
        amm_destination_token_address: destination_token_address,
        amount_in_offset,
    })
}
