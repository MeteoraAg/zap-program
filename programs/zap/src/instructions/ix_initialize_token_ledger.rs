use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenInterface};

use crate::{const_pda, constants::TOKEN_LEDGER_PREFIX, initialize_token_account};

#[event_cpi]
#[derive(Accounts)]
pub struct InitializeTokenLedgerCtx<'info> {
    /// CHECK: zap authority
    #[account(
        address = const_pda::zap_authority::ID,
    )]
    pub zap_authority: AccountInfo<'info>,
    /// CHECK: token_ledger_account initialize in program
    #[account(
        mut,
        seeds = [
            TOKEN_LEDGER_PREFIX.as_ref(),
            token_mint.key().as_ref(),
        ],
        bump,
    )]
    pub token_ledger_account: UncheckedAccount<'info>,

    pub token_mint: InterfaceAccount<'info, Mint>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

pub fn handle_initialize_token_ledger(ctx: Context<InitializeTokenLedgerCtx>) -> Result<()> {
    initialize_token_account(
        &ctx.accounts.zap_authority.to_account_info(),
        &ctx.accounts.token_ledger_account.to_account_info(),
        &ctx.accounts.token_mint.to_account_info(),
        &ctx.accounts.payer.to_account_info(),
        &ctx.accounts.token_program.to_account_info(),
        &ctx.accounts.system_program.to_account_info(),
        &[
            TOKEN_LEDGER_PREFIX.as_ref(),
            ctx.accounts.token_mint.key().as_ref(),
            &[ctx.bumps.token_ledger_account][..],
        ],
    )?;
    Ok(())
}
