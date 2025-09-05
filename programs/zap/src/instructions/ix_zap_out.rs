use std::cmp::min;

use anchor_lang::prelude::*;
use anchor_spl::token_interface::TokenAccount;
use pinocchio::{
    cpi::slice_invoke,
    instruction::{AccountMeta, Instruction},
};
// use pinocchio::cpi::invoke;

use crate::{constants::WHITELISTED_AMM_PROGRAMS, error::ZapError, safe_math::SafeMath};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ZapOutParameters {
    pub percentage: u8,
    pub offset_amount_in: u16,
    pub pre_user_token_balance: u64,
    pub max_swap_amount: u64, // avoid the issue someone send token to user token account when user zap out
    pub payload_data: Vec<u8>,
}

impl ZapOutParameters {
    fn validate(&self) -> Result<()> {
        require!(
            self.percentage <= 100 && self.percentage > 0,
            ZapError::InvalidZapOutParameters
        );

        Ok(())
    }

    fn get_swap_amount(&self, balance_change_amount: u64) -> Result<u64> {
        let swap_amount = if self.percentage == 100 {
            balance_change_amount
        } else {
            let amount = u128::from(balance_change_amount)
                .safe_mul(self.percentage.into())?
                .safe_div(100)?;
            u64::try_from(amount).map_err(|_| ZapError::TypeCastFailed)?
        };

        Ok(min(swap_amount, self.max_swap_amount))
    }
}

pub fn is_support_amm_program(amm_program: &Pubkey, discriminator: &[u8]) -> bool {
    WHITELISTED_AMM_PROGRAMS
        .iter()
        .any(|(program, disc)| program.eq(amm_program) && disc.eq(discriminator))
}

// pub const ZAP_OUT_IX_ACCOUNTS: usize = 2;
pub const ZAP_OUT_IX_ACCOUNTS: usize = 2;
pub const ZAP_OUT_MAX_IX_ACCOUNTS: usize = 100;
#[derive(Accounts)]
pub struct ZapOutCtx<'info> {
    #[account(mut)]
    pub user_token_in_account: InterfaceAccount<'info, TokenAccount>,

    /// CHECK:
    pub amm_program: UncheckedAccount<'info>,
}

pub fn modify_instruction_data(
    payload_data: &mut Vec<u8>,
    amount_in: u64,
    offset_amount_in: usize,
) -> Result<()> {
    let amount_in_bytes = amount_in.to_le_bytes();
    let end_offset_index = offset_amount_in.safe_add(amount_in_bytes.len())?;

    require!(
        end_offset_index <= payload_data.len(),
        ZapError::InvalidOffset
    );
    payload_data.splice(
        offset_amount_in..end_offset_index,
        amount_in_bytes.iter().cloned(),
    );

    Ok(())
}

// pub fn handle_zap_out<'c: 'info, 'info>(
//     ctx: Context<'_, '_, 'c, 'info, ZapOutCtx<'info>>,
//     params: &ZapOutParameters,
// ) -> Result<()> {
//     // validate params
//     params.validate()?;
//     let disciminator = &params.payload_data[..8]; // first 8 bytes is discriminator
//     require!(
//         is_support_amm_program(ctx.accounts.amm_program.key, disciminator),
//         ZapError::AmmIsNotSupported
//     );
//     let post_user_token_balance = ctx.accounts.user_token_in_account.amount;
//     if params.pre_user_token_balance >= post_user_token_balance {
//         // skip if pre_user_token_balance is greater than post_user_token_balance
//         return Ok(());
//     }
//     let balance_change_amount = post_user_token_balance.safe_sub(params.pre_user_token_balance)?;
//     let swap_amount = params.get_swap_amount(balance_change_amount)?;

//     if swap_amount > 0 {
//         let mut payload_data = params.payload_data.to_vec();
//         modify_instruction_data(
//             &mut payload_data,
//             swap_amount,
//             params.offset_amount_in.into(),
//         )?;

//         let accounts: Vec<AccountMeta> = ctx
//             .remaining_accounts
//             .iter()
//             .map(|acc| AccountMeta {
//                 pubkey: *acc.key,
//                 is_signer: acc.is_signer,
//                 is_writable: acc.is_writable,
//             })
//             .collect();

//         let account_infos: Vec<AccountInfo> = ctx
//             .remaining_accounts
//             .iter()
//             .map(|acc| AccountInfo { ..acc.clone() })
//             .collect();
//         // invoke instruction to amm
//         invoke(
//             &Instruction {
//                 program_id: ctx.accounts.amm_program.key(),
//                 accounts,
//                 data: payload_data,
//             },
//             &account_infos,
//         )?;
//     }

//     Ok(())
// }

pub fn p_handle_zap_out(
    _program_id: &pinocchio::pubkey::Pubkey,
    zap_out_accounts: &[pinocchio::account_info::AccountInfo],
    remaining_accounts: &[pinocchio::account_info::AccountInfo],
    data: &[u8],
) -> Result<()> {
    // TODO fix unwrap
    let params = ZapOutParameters::deserialize(&mut &data[8..]).unwrap();
    let [user_token_in_account, amm_program] = zap_out_accounts else {
        return Err(ProgramError::NotEnoughAccountKeys.into());
    };

    // validate params
    params.validate()?;
    let disciminator = &params.payload_data[..8]; // first 8 bytes is discriminator
    require!(
        is_support_amm_program(&Pubkey::new_from_array(*amm_program.key()), disciminator),
        ZapError::AmmIsNotSupported
    );
    let user_token_in_account =
        pinocchio_token::state::TokenAccount::from_account_info(user_token_in_account).unwrap();
    let post_user_token_balance = user_token_in_account.amount();
    if params.pre_user_token_balance >= post_user_token_balance {
        // skip if pre_user_token_balance is greater than post_user_token_balance
        return Ok(());
    }
    let balance_change_amount = post_user_token_balance.safe_sub(params.pre_user_token_balance)?;
    let swap_amount = params.get_swap_amount(balance_change_amount)?;
    if swap_amount > 0 {
        let mut payload_data = params.payload_data.to_vec();
        modify_instruction_data(
            &mut payload_data,
            swap_amount,
            params.offset_amount_in.into(),
        )?;

        let accounts: Vec<AccountMeta> = remaining_accounts
            .iter()
            .map(|acc| AccountMeta {
                pubkey: acc.key(),
                is_signer: acc.is_signer(),
                is_writable: acc.is_writable(),
            })
            .collect();

        let account_infos: Vec<&pinocchio::account_info::AccountInfo> =
            remaining_accounts.iter().map(|acc| acc).collect();

        drop(user_token_in_account);
        // invoke instruction to amm
        slice_invoke(
            &Instruction {
                program_id: amm_program.key(),
                accounts: &accounts,
                data: &payload_data,
            },
            &account_infos[..],
        )
        .unwrap();
    }

    Ok(())
}
