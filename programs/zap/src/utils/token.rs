use crate::error::ZapError;
use anchor_lang::prelude::*;
use anchor_spl::token_2022::spl_token_2022::extension::{self};
use anchor_spl::token_interface::{TokenAccount, TokenInterface};
use anchor_spl::{
    token::Token,
    token_2022::spl_token_2022::{self, extension::StateWithExtensions},
    token_interface::Mint,
};

fn get_transfer_hook_program_id<'info>(
    token_mint: &InterfaceAccount<'info, Mint>,
) -> Result<Option<Pubkey>> {
    let token_mint_info = token_mint.to_account_info();
    if *token_mint_info.owner == Token::id() {
        return Ok(None);
    }

    let token_mint_data = token_mint_info.try_borrow_data()?;
    let token_mint_unpacked =
        StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&token_mint_data)?;
    Ok(extension::transfer_hook::get_program_id(
        &token_mint_unpacked,
    ))
}

pub fn transfer_from_user<'a, 'c: 'info, 'info>(
    authority: &'a Signer<'info>,
    token_mint: &'a InterfaceAccount<'info, Mint>,
    source_token_account: &'a InterfaceAccount<'info, TokenAccount>,
    destination_token_account: &'a InterfaceAccount<'info, TokenAccount>,
    token_program: &'a Interface<'info, TokenInterface>,
    amount: u64,
    transfer_hook_accounts: Option<&'c [AccountInfo<'info>]>,
) -> Result<()> {
    let destination_account = destination_token_account.to_account_info();

    let mut instruction = spl_token_2022::instruction::transfer_checked(
        token_program.key,
        &source_token_account.key(),
        &token_mint.key(),
        destination_account.key,
        authority.key,
        &[],
        amount,
        token_mint.decimals,
    )?;

    let mut account_infos = vec![
        source_token_account.to_account_info(),
        token_mint.to_account_info(),
        destination_account.to_account_info(),
        authority.to_account_info(),
    ];

    if let Some(hook_program_id) = get_transfer_hook_program_id(token_mint)? {
        let Some(transfer_hook_accounts) = transfer_hook_accounts else {
            return Err(ZapError::MissingRemainingAccountForTransferHook.into());
        };

        spl_transfer_hook_interface::onchain::add_extra_accounts_for_execute_cpi(
            &mut instruction,
            &mut account_infos,
            &hook_program_id,
            source_token_account.to_account_info(),
            token_mint.to_account_info(),
            destination_account.to_account_info(),
            authority.to_account_info(),
            amount,
            transfer_hook_accounts,
        )?;
    } else {
        require!(
            transfer_hook_accounts.is_none(),
            ZapError::NoTransferHookProgram
        );
    }

    anchor_lang::solana_program::program::invoke(&instruction, &account_infos)?;

    Ok(())
}
