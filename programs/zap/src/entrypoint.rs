use anchor_lang::{prelude::*, solana_program};
use pinocchio::MAX_TX_ACCOUNTS;

use crate::{entry, p_handle_zap_out, ZAP_OUT_IX_ACCOUNTS};

#[inline(always)]
unsafe fn p_entrypoint(input: *mut u8) -> Option<u64> {
    const UNINIT: core::mem::MaybeUninit<pinocchio::account_info::AccountInfo> =
        core::mem::MaybeUninit::<pinocchio::account_info::AccountInfo>::uninit();
    // Create an array of uninitialized account infos.
    let mut accounts = [UNINIT; MAX_TX_ACCOUNTS];

    let (program_id, count, instruction_data) =
        pinocchio::entrypoint::deserialize(input, &mut accounts);

    let result = if instruction_data.starts_with(crate::instruction::ZapOut::DISCRIMINATOR) {
        let (left, right) = accounts.split_at_unchecked(ZAP_OUT_IX_ACCOUNTS);
        let accounts = core::slice::from_raw_parts(left.as_ptr() as _, ZAP_OUT_IX_ACCOUNTS);
        let remaining_accounts = core::slice::from_raw_parts(
            right.as_ptr() as _,
            count.checked_sub(ZAP_OUT_IX_ACCOUNTS)?,
        );

        Some(p_handle_zap_out(
            &program_id,
            accounts,
            remaining_accounts,
            &instruction_data,
        ))
    } else {
        None
    };

    result.map(|value| match value {
        Ok(()) => solana_program::entrypoint::SUCCESS,
        Err(error) => {
            error.log();
            anchor_lang::solana_program::program_error::ProgramError::from(error).into()
        }
    })
}

/// Hot path pinocchio entrypoint with anchor fallback otherwise
#[no_mangle]
pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
    match p_entrypoint(input) {
        Some(result) => result,
        None => {
            let (program_id, accounts, instruction_data) =
                unsafe { solana_program::entrypoint::deserialize(input) };

            match entry(program_id, &accounts, instruction_data) {
                Ok(()) => solana_program::entrypoint::SUCCESS,
                Err(error) => error.into(),
            }
        }
    }
}
solana_program::custom_heap_default!();
solana_program::custom_panic_default!();
