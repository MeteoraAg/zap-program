use crate::error::ZapError;
use anchor_lang::prelude::*;
// use anchor_spl::token_2022::spl_token_2022::extension::transfer_fee::TransferFee;
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

// #[derive(Debug)]
// pub struct TransferFeeExcludedAmount {
//     pub amount: u64,
//     pub transfer_fee: u64,
// }

// pub fn calculate_transfer_fee_excluded_amount<'info>(
//     token_mint: &InterfaceAccount<'info, Mint>,
//     transfer_fee_included_amount: u64,
// ) -> Result<TransferFeeExcludedAmount> {
//     if let Some(epoch_transfer_fee) = get_epoch_transfer_fee(token_mint)? {
//         let transfer_fee = epoch_transfer_fee
//             .calculate_fee(transfer_fee_included_amount)
//             .ok_or_else(|| ZapError::MathOverflow)?;
//         let transfer_fee_excluded_amount = transfer_fee_included_amount
//             .checked_sub(transfer_fee)
//             .ok_or_else(|| ZapError::MathOverflow)?;
//         return Ok(TransferFeeExcludedAmount {
//             amount: transfer_fee_excluded_amount,
//             transfer_fee,
//         });
//     }

//     Ok(TransferFeeExcludedAmount {
//         amount: transfer_fee_included_amount,
//         transfer_fee: 0,
//     })
// }

// pub fn get_epoch_transfer_fee<'info>(
//     token_mint: &InterfaceAccount<'info, Mint>,
// ) -> Result<Option<TransferFee>> {
//     let token_mint_info = token_mint.to_account_info();
//     if *token_mint_info.owner == Token::id() {
//         return Ok(None);
//     }

//     let token_mint_data = token_mint_info.try_borrow_data()?;
//     let token_mint_unpacked =
//         StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&token_mint_data)?;
//     if let Ok(transfer_fee_config) =
//         token_mint_unpacked.get_extension::<extension::transfer_fee::TransferFeeConfig>()
//     {
//         let epoch = Clock::get()?.epoch;
//         return Ok(Some(transfer_fee_config.get_epoch_fee(epoch).clone()));
//     }

//     Ok(None)
// }
