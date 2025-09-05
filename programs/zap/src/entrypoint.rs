use anchor_lang::{
    solana_program::{custom_heap_default, custom_panic_default, entrypoint as a_entrypoint},
    Discriminator,
};
use pinocchio::{
    account_info::AccountInfo as pAccountInfo, entrypoint as p_entrypoint, MAX_TX_ACCOUNTS, SUCCESS,
};
use pinocchio_log::log;

use crate::{entry, instruction::ZapOut2, p_zap_out_2};

#[no_mangle]
pub unsafe extern "C" fn entrypoint(input: *mut u8) -> u64 {
    const UNINIT: core::mem::MaybeUninit<pAccountInfo> =
        core::mem::MaybeUninit::<pAccountInfo>::uninit();
    let mut accounts = [UNINIT; MAX_TX_ACCOUNTS];
    let (program_id, count, instruction_data) =
        p_entrypoint::deserialize::<MAX_TX_ACCOUNTS>(input, &mut accounts);

    let (discriminator, data) = instruction_data.split_at(8);

    match discriminator {
        disc if disc == ZapOut2::DISCRIMINATOR => {
            log!("Zap Out 2");

            match p_zap_out_2::handle(
                &program_id,
                core::slice::from_raw_parts(accounts.as_ptr() as _, count),
                &data,
            ) {
                Ok(()) => SUCCESS,
                Err(error) => error.into(),
            }
        }
        _ => {
            let (__program_id, __accounts, __instruction_data) =
                unsafe { a_entrypoint::deserialize(input) };
            match entry(__program_id, &__accounts, __instruction_data) {
                Ok(()) => a_entrypoint::SUCCESS,
                Err(error) => error.into(),
            }
        }
    }
}

custom_heap_default!();
custom_panic_default!();
