use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use damm_v2::types::SwapParameters;

use crate::{const_pda, constants::amm_program_id, error::ZapError, DammV2SwapAccounts};

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
    fn handle_zap_out_damm_v2(
        &self,
        remaining_accounts: &'info [AccountInfo<'info>],
        data: Vec<u8>,
    ) -> Result<()> {
        let signer_seeds = zap_authority_seeds!(const_pda::zap_authority::BUMP);

        let parsed_remaining_accounts: DammV2SwapAccounts =
            DammV2SwapAccounts::parse_remaining_accounts(remaining_accounts)?;

        // validate damm v2 accounts
        parsed_remaining_accounts.validate()?;

        damm_v2::cpi::swap(
            CpiContext::new_with_signer(
                self.amm_program.to_account_info(),
                damm_v2::cpi::accounts::Swap {
                    pool_authority: parsed_remaining_accounts.pool_authority.to_account_info(),
                    pool: parsed_remaining_accounts.pool.to_account_info(),
                    input_token_account: self.token_ledger_account.to_account_info(),
                    output_token_account: parsed_remaining_accounts
                        .output_token_account
                        .to_account_info(),
                    token_a_vault: parsed_remaining_accounts.token_a_vault.to_account_info(),
                    token_b_vault: parsed_remaining_accounts.token_b_vault.to_account_info(),
                    token_a_mint: parsed_remaining_accounts.token_a_mint.to_account_info(),
                    token_b_mint: parsed_remaining_accounts.token_b_mint.to_account_info(),
                    payer: self.zap_authority.to_account_info(),
                    token_a_program: parsed_remaining_accounts.token_a_program.to_account_info(),
                    token_b_program: parsed_remaining_accounts.token_b_program.to_account_info(),
                    referral_token_account: None,
                    event_authority: parsed_remaining_accounts.event_authority.to_account_info(),
                    program: self.amm_program.to_account_info(),
                },
                &[&signer_seeds[..]],
            ),
            SwapParameters {
                amount_in: self.token_ledger_account.amount,
                minimum_amount_out: 0, // TODO: parse minimum_amount_out from data
            },
        )?;

        Ok(())
    }

    fn handle_zap_out_dlmm(&self) -> Result<()> {
        let signer_seeds = zap_authority_seeds!(const_pda::zap_authority::BUMP);
        // dlmm::cpi::swap2(
        //     CpiContext::new_with_signer(
        //         self.amm_program.to_account_info(),
        //         dlmm::cpi::accounts::Swap2 {
        //             lb_pair,
        //             bin_array_bitmap_extension,
        //             reserve_x,
        //             reserve_y,
        //             user_token_in,
        //             user_token_out,
        //             token_x_mint,
        //             token_y_mint,
        //             oracle,
        //             host_fee_in,
        //             user: self.zap_authority.to_account_info(),
        //             token_x_program,
        //             token_y_program,
        //             memo_program,
        //             event_authority,
        //             program: self.amm_program.to_account_info(),
        //         },
        //         &[&signer_seeds[..]],
        //     ),
        //     amount_in,
        //     min_amount_out,
        //     remaining_accounts_info,
        // )?;
        Ok(())
    }
}

pub fn handle_zap_out<'c: 'info, 'info>(
    ctx: Context<'_, '_, 'c, 'info, ZapOutCtx<'info>>,
    data: Vec<u8>,
) -> Result<()> {
    let amm_program: AmmProgram = AmmProgram::from_pubkey(&ctx.accounts.amm_program.key())?;

    match amm_program {
        AmmProgram::DammV2 => ctx
            .accounts
            .handle_zap_out_damm_v2(ctx.remaining_accounts, data)?,
        AmmProgram::Dlmm => ctx.accounts.handle_zap_out_dlmm()?,
    }

    Ok(())
}
