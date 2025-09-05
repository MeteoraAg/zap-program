use std::{cell::RefCell, collections::BTreeSet, rc::Rc, u64};

use anchor_lang::{
    context::Context,
    solana_program::{account_info::AccountInfo, msg, pubkey::Pubkey},
    Accounts, AnchorDeserialize, Discriminator,
};
use pinocchio::{
    account_info::AccountInfo as pAccountInfo, entrypoint, program_error::ProgramError,
    pubkey::Pubkey as pPubkey, ProgramResult,
};
use pinocchio_log::log;

use crate::{instruction, zap, ZapOutCtx, ZapOutCtxBumps, ZapOutParameters};

pub struct BridgedAccountInfo<'a> {
    key: Pubkey,
    lamports: &'a mut u64,
    data: &'a mut [u8],
    owner: Pubkey,
    is_signer: bool,
    is_writable: bool,
    executable: bool,
}

impl<'a> From<&'a mut BridgedAccountInfo<'a>> for AccountInfo<'a> {
    fn from(info: &'a mut BridgedAccountInfo<'a>) -> Self {
        AccountInfo {
            key: &info.key,
            lamports: Rc::new(RefCell::new(info.lamports)),
            data: Rc::new(RefCell::new(info.data)),
            owner: &info.owner,
            rent_epoch: u64::MAX,
            is_signer: info.is_signer,
            is_writable: info.is_writable,
            executable: info.executable,
        }
    }
}

entrypoint!(process_instruction);

pub unsafe fn process_instruction(
    program_id: &pPubkey,
    accounts: &[pAccountInfo],
    data: &[u8],
) -> ProgramResult {
    let (discriminator, mut instruction_data) = data.split_at(8);

    match discriminator {
        data if data == instruction::ZapOut::DISCRIMINATOR => {
            log!("Zap Out");

            let program_id = Pubkey::new_from_array(*program_id);
            let mut bumps = ZapOutCtxBumps::default();

            let accounts_iter = &mut accounts.iter();

            let mut bridged_account_info: Vec<BridgedAccountInfo> = accounts_iter
                .map(|info| BridgedAccountInfo {
                    key: Pubkey::new_from_array(*info.key()),
                    lamports: info.borrow_mut_lamports_unchecked(),
                    data: info.borrow_mut_data_unchecked(),
                    owner: Pubkey::new_from_array(*info.owner()),
                    is_signer: info.is_signer(),
                    is_writable: info.is_writable(),
                    executable: info.executable(),
                })
                .collect();
            let account_info: Vec<AccountInfo> =
                bridged_account_info.iter_mut().map(|e| e.into()).collect();

            let mut zap_out_ctx = ZapOutCtx::try_accounts(
                &program_id,
                &mut account_info.as_slice(),
                instruction_data,
                &mut bumps,
                &mut BTreeSet::new(),
            )
            .map_err(|_| ProgramError::Custom(101))?;

            let ctx = Context::new(
                &program_id,
                &mut zap_out_ctx,
                account_info.as_slice(),
                bumps,
            );

            let params = ZapOutParameters::deserialize(&mut instruction_data)
                .map_err(|_| ProgramError::InvalidInstructionData)?;

            zap::zap_out(ctx, params).map_err(|e| {
                msg!("er {}", e);
                ProgramError::Custom(100)
            })?
        }
        _ => {
            log!("default {}", discriminator);
        }
    }

    Ok(())
}
