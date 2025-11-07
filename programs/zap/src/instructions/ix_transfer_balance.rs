use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{parse_remaining_accounts, transfer_from_user, AccountsType, RemainingAccountsInfo};

#[derive(Accounts)]
pub struct TransferBalanceCtx<'info> {
    #[account(mut)]
    pub source_token: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut)]
    pub destination_token: Box<InterfaceAccount<'info, TokenAccount>>,

    pub token_mint: Box<InterfaceAccount<'info, Mint>>,

    pub token_program: Interface<'info, TokenInterface>,

    pub user: Signer<'info>,
}

pub fn handle_transfer_delta_balance<'c: 'info, 'info>(
    ctx: Context<'_, '_, 'c, 'info, TransferBalanceCtx<'info>>,
    pre_source_token_balance: u64,
    max_transfer_amount: u64,
    remaining_accounts_info: RemainingAccountsInfo,
) -> Result<()> {
    let delta_balance = ctx
        .accounts
        .source_token
        .amount
        .saturating_sub(pre_source_token_balance);
    let amount = delta_balance.min(max_transfer_amount);
    if amount > 0 {
        let mut remaining_accounts = &ctx.remaining_accounts[..];
        let parsed_transfer_hook_accounts = parse_remaining_accounts(
            &mut remaining_accounts,
            &remaining_accounts_info.slices,
            &[AccountsType::TransferHook],
        )?;
        transfer_from_user(
            &ctx.accounts.user,
            &ctx.accounts.token_mint,
            &ctx.accounts.source_token,
            &ctx.accounts.destination_token,
            &ctx.accounts.token_program,
            amount,
            parsed_transfer_hook_accounts.transfer_hook,
        )?;
    }
    Ok(())
}

pub fn handle_transfer_full_balance<'c: 'info, 'info>(
    ctx: Context<'_, '_, 'c, 'info, TransferBalanceCtx<'info>>,
    remaining_accounts_info: RemainingAccountsInfo,
) -> Result<()> {
    let amount = ctx.accounts.source_token.amount;
    if amount > 0 {
        let mut remaining_accounts = &ctx.remaining_accounts[..];
        let parsed_transfer_hook_accounts = parse_remaining_accounts(
            &mut remaining_accounts,
            &remaining_accounts_info.slices,
            &[AccountsType::TransferHook],
        )?;
        transfer_from_user(
            &ctx.accounts.user,
            &ctx.accounts.token_mint,
            &ctx.accounts.source_token,
            &ctx.accounts.destination_token,
            &ctx.accounts.token_program,
            amount,
            parsed_transfer_hook_accounts.transfer_hook,
        )?;
    }
    Ok(())
}

pub fn handle_transfer_max_balance<'c: 'info, 'info>(
    ctx: Context<'_, '_, 'c, 'info, TransferBalanceCtx<'info>>,
    max_amount: u64,
    remaining_accounts_info: RemainingAccountsInfo,
) -> Result<()> {
    let amount = ctx.accounts.source_token.amount.min(max_amount);
    if amount > 0 {
        let mut remaining_accounts = &ctx.remaining_accounts[..];
        let parsed_transfer_hook_accounts = parse_remaining_accounts(
            &mut remaining_accounts,
            &remaining_accounts_info.slices,
            &[AccountsType::TransferHook],
        )?;
        transfer_from_user(
            &ctx.accounts.user,
            &ctx.accounts.token_mint,
            &ctx.accounts.source_token,
            &ctx.accounts.destination_token,
            &ctx.accounts.token_program,
            amount,
            parsed_transfer_hook_accounts.transfer_hook,
        )?;
    }
    Ok(())
}
