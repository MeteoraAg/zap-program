use anchor_lang::solana_program::entrypoint as a_entrypoint;
use pinocchio::{
    account_info::AccountInfo as pAccountInfo, entrypoint as p_entrypoint, MAX_TX_ACCOUNTS, SUCCESS,
};

use crate::entry;

#[no_mangle]
pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
    if false {
        const UNINIT: core::mem::MaybeUninit<pAccountInfo> =
            core::mem::MaybeUninit::<pAccountInfo>::uninit();
        // Create an array of uninitialized account infos.
        let mut accounts = [UNINIT; MAX_TX_ACCOUNTS];
        let (program_id, count, instruction_data) =
            p_entrypoint::deserialize::<MAX_TX_ACCOUNTS>(input, &mut accounts);

        // Call the program's entrypoint passing `count` account infos; we know that
        // they are initialized so we cast the pointer to a slice of `[AccountInfo]`.
        // match $process_instruction(
        //     &program_id,
        //     core::slice::from_raw_parts(accounts.as_ptr() as _, count),
        //     &instruction_data,
        // ) {
        //     Ok(()) => $crate::SUCCESS,
        //     Err(error) => error.into(),
        // }

        SUCCESS
    } else {
        let (program_id, accounts, instruction_data) = unsafe { a_entrypoint::deserialize(input) };
        match entry(program_id, &accounts, instruction_data) {
            Ok(()) => a_entrypoint::SUCCESS,
            Err(error) => error.into(),
        }
    }
}
