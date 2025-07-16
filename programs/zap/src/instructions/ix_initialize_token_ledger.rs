use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{const_pda, constants::TOKEN_LEDGER_PREFIX};

#[event_cpi]
#[derive(Accounts)]
pub struct InitializeTokenLedgerCtx<'info> {
    /// CHECK: zap authority
    #[account(
        address = const_pda::zap_authority::ID,
    )]
    pub zap_authority: AccountInfo<'info>,

    #[account(
        init,
        payer = payer,
        seeds = [
            TOKEN_LEDGER_PREFIX.as_ref(),
            token_mint.key().as_ref(),
        ],
        bump,
        token::mint = token_mint,
        token::authority = zap_authority,
        token::token_program = token_program
    )]
    pub token_ledger_account: InterfaceAccount<'info, TokenAccount>,

    pub token_mint: InterfaceAccount<'info, Mint>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

pub fn handle_initialize_token_ledger(_ctx: Context<InitializeTokenLedgerCtx>) -> Result<()> {
    Ok(())
}
