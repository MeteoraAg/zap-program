use anchor_lang::prelude::*;
use anchor_spl::token_interface::TokenAccount;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ZapOut2Parameters {
    pub percentage: u8,
    pub offset_amount_in: u16,
    pub pre_user_token_balance: u64,
    pub max_swap_amount: u64, // avoid the issue someone send token to user token account when user zap out
    pub payload_data: Vec<u8>,
}

#[derive(Accounts)]
pub struct ZapOut2Ctx<'info> {
    #[account(mut)]
    pub user_token_in_account: InterfaceAccount<'info, TokenAccount>,

    /// CHECK:
    pub amm_program: UncheckedAccount<'info>,
}

pub mod p_zap_out_2 {
    use super::{AnchorDeserialize, ZapOut2Parameters};
    use crate::{constants::WHITELISTED_AMM_PROGRAMS, error::PinoError, math::safe_math::SafeMath};
    use pinocchio::{
        account_info::AccountInfo,
        cpi::slice_invoke,
        instruction::{AccountMeta, Instruction},
        program_error::ProgramError,
        pubkey::Pubkey,
        ProgramResult,
    };
    use pinocchio_log::log;
    use pinocchio_token::state::TokenAccount;

    impl ZapOut2Parameters {
        pub fn validate(&self) -> ProgramResult {
            if self.percentage > 100 || self.percentage <= 0 {
                Err(PinoError::InvalidZapOutParameters.into())
            } else {
                Ok(())
            }
        }

        pub fn get_swap_amount(&self, balance_change_amount: u64) -> Result<u64, ProgramError> {
            let swap_amount = if self.percentage == 100 {
                balance_change_amount
            } else {
                let amount = u128::from(balance_change_amount)
                    .safe_mul(self.percentage.into())
                    .map_err(|_| ProgramError::ArithmeticOverflow)?
                    .safe_div(100)
                    .map_err(|_| ProgramError::ArithmeticOverflow)?;
                u64::try_from(amount).map_err(|_| PinoError::TypeCastFailed)?
            };

            Ok(if swap_amount < self.max_swap_amount {
                swap_amount
            } else {
                self.max_swap_amount
            })
        }
    }

    fn is_support_amm_program(amm_program: &Pubkey, discriminator: &[u8]) -> bool {
        WHITELISTED_AMM_PROGRAMS
            .iter()
            .map(|(program, disc)| (program.as_array(), disc))
            .any(|(program, disc)| program.eq(amm_program) && disc.eq(discriminator))
    }

    fn modify_instruction_data(
        payload_data: &mut Vec<u8>,
        amount_in: u64,
        offset_amount_in: usize,
    ) -> ProgramResult {
        let amount_in_bytes = amount_in.to_le_bytes();
        let end_offset_index = offset_amount_in
            .safe_add(amount_in_bytes.len())
            .map_err(|_| ProgramError::ArithmeticOverflow)?;

        if end_offset_index > payload_data.len() {
            return Err(PinoError::InvalidOffset.into());
        }
        payload_data.splice(
            offset_amount_in..end_offset_index,
            amount_in_bytes.iter().cloned(),
        );

        Ok(())
    }

    pub fn handle(
        _program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        // Parse account infos
        let accounts_iter = &mut accounts.iter();
        let user_token_in_account_info = accounts_iter.next().ok_or(PinoError::InvalidAccount)?;
        let amm_program_info = accounts_iter.next().ok_or(PinoError::InvalidAccount)?;

        // Parse params
        let params = ZapOut2Parameters::try_from_slice(instruction_data)
            .map_err(|_| PinoError::InvalidZapOutParameters)?;

        params.validate()?;
        let disciminator = &params.payload_data[..8]; // first 8 bytes is discriminator
        if !is_support_amm_program(amm_program_info.key(), disciminator) {
            return Err(PinoError::AmmIsNotSupported.into());
        }

        let post_user_token_balance = {
            let user_token_in_account = TokenAccount::from_account_info(user_token_in_account_info)
                .map_err(|_| PinoError::InvalidAccount)?;
            user_token_in_account.amount()
        };

        // skip if pre_user_token_balance is greater than post_user_token_balance
        if params.pre_user_token_balance >= post_user_token_balance {
            return Ok(());
        }

        let balance_change_amount = post_user_token_balance
            .safe_sub(params.pre_user_token_balance)
            .map_err(|_| ProgramError::ArithmeticOverflow)?;
        let swap_amount = params.get_swap_amount(balance_change_amount)?;

        if swap_amount <= 0 {
            return Ok(());
        }

        let mut payload_data = params.payload_data.to_vec();
        modify_instruction_data(
            &mut payload_data,
            swap_amount,
            params.offset_amount_in.into(),
        )?;

        let account_infos: Vec<&AccountInfo> = accounts_iter.collect();
        let account_metas: Vec<AccountMeta> = account_infos
            .iter()
            .map(|acc| AccountMeta {
                pubkey: acc.key(),
                is_signer: acc.is_signer(),
                is_writable: acc.is_writable(),
            })
            .collect();

        // invoke instruction to amm
        slice_invoke(
            &Instruction {
                program_id: amm_program_info.key(),
                accounts: account_metas.as_slice(),
                data: &payload_data,
            },
            account_infos.as_slice(),
        )
    }
}
