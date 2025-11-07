use anchor_lang::prelude::*;
use anchor_spl::token::accessor;

use crate::{constants::seeds::USER_LEDGER_PREFIX, UserLedger};

#[derive(Accounts)]
pub struct InitializeLedgerAccountCtx<'info> {
    #[account(
        init,
        seeds = [USER_LEDGER_PREFIX.as_ref(), owner.key().as_ref()],
        payer = payer,
        space = 8 + UserLedger::INIT_SPACE,
        bump
    )]
    pub legder: AccountLoader<'info, UserLedger>,

    pub owner: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handle_initialize_ledger_account(ctx: Context<InitializeLedgerAccountCtx>) -> Result<()> {
    let mut ledger = ctx.accounts.legder.load_init()?;
    ledger.owner = ctx.accounts.owner.key();
    Ok(())
}

#[derive(Accounts)]
pub struct CloseLedgerAccountCtx<'info> {
    #[account(
        mut,
        has_one = owner,
        close = rent_receiver,
    )]
    pub legder: AccountLoader<'info, UserLedger>,

    pub owner: Signer<'info>,

    #[account(mut)]
    pub rent_receiver: Signer<'info>,
}

pub fn handle_close_ledger_account(_ctx: Context<CloseLedgerAccountCtx>) -> Result<()> {
    // anchor do everything
    Ok(())
}

#[derive(Accounts)]
pub struct SetLedgerBalanceCtx<'info> {
    #[account(
       mut, has_one = owner
    )]
    pub legder: AccountLoader<'info, UserLedger>,

    pub owner: Signer<'info>,
}

pub fn handle_set_ledger_balance(
    ctx: Context<SetLedgerBalanceCtx>,
    amount: u64,
    is_token_a: bool,
) -> Result<()> {
    let mut ledger = ctx.accounts.legder.load_mut()?;
    if is_token_a {
        ledger.amount_a = amount
    } else {
        ledger.amount_b = amount
    }
    Ok(())
}

#[derive(Accounts)]
pub struct UpdateLedgerBalanceAfterSwapCtx<'info> {
    #[account(
       mut, has_one = owner
    )]
    pub legder: AccountLoader<'info, UserLedger>,

    /// CHECK: user must send correct user account
    pub token_account: UncheckedAccount<'info>,

    pub owner: Signer<'info>,
}

pub fn handle_update_ledger_balance_after_swap(
    ctx: Context<UpdateLedgerBalanceAfterSwapCtx>,
    pre_source_token_balance: u64,
    max_transfer_amount: u64,
    is_token_a: bool,
) -> Result<()> {
    let current_token_balance = accessor::amount(&ctx.accounts.token_account.to_account_info())?;
    let delta_balance: u64 = current_token_balance.saturating_sub(pre_source_token_balance);
    let amount = delta_balance.min(max_transfer_amount);
    let mut ledger = ctx.accounts.legder.load_mut()?;
    if is_token_a {
        ledger.amount_a = amount
    } else {
        ledger.amount_b = amount
    }
    Ok(())
}
