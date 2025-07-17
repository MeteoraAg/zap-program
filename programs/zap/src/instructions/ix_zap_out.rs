use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program::invoke_signed},
    InstructionData,
};
use anchor_spl::token_interface::TokenAccount;
use damm_v2::types::SwapParameters;

use crate::{const_pda, constants::amm_program_id, error::ZapError};

#[derive(Debug, Clone, Copy)]
pub enum AmmProgram {
    DammV2,
    Dlmm,
}

impl AmmProgram {
    pub fn from_pubkey(pubkey: &Pubkey) -> Result<AmmProgram> {
        match *pubkey {
            amm_program_id::DAMM_V2 => Ok(AmmProgram::DammV2),
            amm_program_id::DLMM => Ok(AmmProgram::Dlmm),
            _ => Err(error!(ZapError::UnsupportedAmmProgram)),
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ZapOutParameters {
    pub minimum_amount_out: u64,
    pub padding_0: [u64; 16],
    pub remaining_accounts_info: Option<dlmm::types::RemainingAccountsInfo>,
}

#[event_cpi]
#[derive(Accounts)]
pub struct ZapOutCtx<'info> {
    /// CHECK: zap authority
    #[account(
        address = const_pda::zap_authority::ID,
    )]
    pub zap_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub token_ledger_account: InterfaceAccount<'info, TokenAccount>,

    /// CHECK:
    pub amm_program: UncheckedAccount<'info>,
}

impl<'info> ZapOutCtx<'info> {
    fn get_damm_v2_instruction_data(&self, minimum_amount_out: u64) -> Result<Vec<u8>> {
        let data = damm_v2::client::args::Swap {
            params: SwapParameters {
                amount_in: self.token_ledger_account.amount,
                minimum_amount_out,
            },
        }
        .data();

        Ok(data)
    }

    fn get_dlmm_instruction_data(
        &self,
        minimum_amount_out: u64,
        remaining_accounts_info: dlmm::types::RemainingAccountsInfo,
    ) -> Result<Vec<u8>> {
        let data = dlmm::client::args::Swap2 {
            amount_in: self.token_ledger_account.amount,
            min_amount_out: minimum_amount_out,
            remaining_accounts_info,
        }
        .data();

        Ok(data)
    }
}

pub fn handle_zap_out<'c: 'info, 'info>(
    ctx: Context<'_, '_, 'c, 'info, ZapOutCtx<'info>>,
    params: ZapOutParameters,
) -> Result<()> {
    let ZapOutParameters {
        minimum_amount_out,
        remaining_accounts_info,
        ..
    } = params;
    let amm_program: AmmProgram = AmmProgram::from_pubkey(&ctx.accounts.amm_program.key())?;

    let accounts: Vec<AccountMeta> = ctx
        .remaining_accounts
        .iter()
        .map(|acc| {
            let is_signer = acc.key == &ctx.accounts.zap_authority.key();
            AccountMeta {
                pubkey: *acc.key,
                is_signer: is_signer,
                is_writable: acc.is_writable,
            }
        })
        .collect();

    let account_infos: Vec<AccountInfo> = ctx
        .remaining_accounts
        .iter()
        .map(|acc| AccountInfo { ..acc.clone() })
        .collect();

    let data = match amm_program {
        AmmProgram::DammV2 => ctx
            .accounts
            .get_damm_v2_instruction_data(minimum_amount_out)?,
        AmmProgram::Dlmm => {
            if let Some(remaining_accounts_info) = remaining_accounts_info {
                ctx.accounts
                    .get_dlmm_instruction_data(minimum_amount_out, remaining_accounts_info)?
            } else {
                return Err(ZapError::MissingDlmmRemainingAccountInfo.into());
            }
        }
    };

    let signers_seeds = zap_authority_seeds!();

    invoke_signed(
        &Instruction {
            program_id: ctx.accounts.amm_program.key(),
            accounts,
            data,
        },
        &account_infos,
        &[&signers_seeds[..]],
    )?;

    Ok(())
}
