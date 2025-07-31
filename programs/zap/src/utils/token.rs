use anchor_lang::{
    prelude::*,
    solana_program::{
        program::{invoke, invoke_signed},
        system_instruction,
    },
};

use anchor_spl::{
    memo::{self, BuildMemo, Memo},
    token::Token,
    token_2022::{
        get_account_data_size,
        spl_token_2022::{
            self,
            extension::{
                self, transfer_hook::get_program_id, BaseStateWithExtensions, ExtensionType,
                StateWithExtensions,
            },
        },
        GetAccountDataSize,
    },
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::error::ZapError;

pub fn initialize_token_account<'info>(
    authority: &AccountInfo<'info>,
    token_account: &AccountInfo<'info>,
    token_mint: &AccountInfo<'info>,
    funder: &AccountInfo<'info>,
    token_program: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
    signer_seeds: &[&[u8]],
) -> Result<()> {
    let is_token_2022 = token_program.key() == spl_token_2022::ID;

    // The size required for extensions that are mandatory on the TokenAccount side — based on the TokenExtensions enabled on the Mint —
    // is automatically accounted for. For non-mandatory extensions, however, they must be explicitly added,
    // so we specify ImmutableOwner explicitly.
    let space = get_account_data_size(
        CpiContext::new(
            token_program.clone(),
            GetAccountDataSize {
                mint: token_mint.clone(),
            },
        ),
        // Needless to say, the program will never attempt to change the owner of the vault.
        // However, since the ImmutableOwner extension only increases the account size by 4 bytes, the overhead of always including it is negligible.
        // On the other hand, it makes it easier to comply with cases where ImmutableOwner is required, and it adds a layer of safety from a security standpoint.
        // Therefore, we'll include it by default going forward. (Vaults initialized after this change will have the ImmutableOwner extension.)
        if is_token_2022 {
            &[ExtensionType::ImmutableOwner]
        } else {
            &[]
        },
    )?;

    create_pda_account(
        funder,
        &Rent::get()?,
        space as usize,
        &token_program.key,
        &system_program.clone(),
        &token_account.clone(),
        signer_seeds,
    )?;

    if is_token_2022 {
        // initialize ImmutableOwner extension
        invoke(
            &spl_token_2022::instruction::initialize_immutable_owner(
                token_program.key,
                token_account.key,
            )?,
            &[token_program.clone(), token_account.clone()],
        )?;
    }

    // initialize token account
    invoke(
        &spl_token_2022::instruction::initialize_account3(
            token_program.key,
            token_account.key,
            &token_mint.key(),
            &authority.key(),
        )?,
        &[
            token_program.clone(),
            token_account.clone(),
            token_mint.clone(),
            authority.clone(),
        ],
    )?;

    Ok(())
}

/// refer the code from https://github.com/solana-program/associated-token-account/blob/28cbfb701bb791ab74b912e5e489731e7c79e164/program/src/tools/account.rs#L19
pub fn create_pda_account<'a>(
    payer: &AccountInfo<'a>,
    rent: &Rent,
    space: usize,
    owner: &Pubkey,
    system_program: &AccountInfo<'a>,
    new_pda_account: &AccountInfo<'a>,
    new_pda_signer_seeds: &[&[u8]],
) -> Result<()> {
    if new_pda_account.lamports() > 0 {
        let required_lamports = rent
            .minimum_balance(space)
            .max(1)
            .saturating_sub(new_pda_account.lamports());

        if required_lamports > 0 {
            invoke(
                &system_instruction::transfer(payer.key, new_pda_account.key, required_lamports),
                &[
                    payer.clone(),
                    new_pda_account.clone(),
                    system_program.clone(),
                ],
            )?;
        }

        invoke_signed(
            &system_instruction::allocate(new_pda_account.key, space as u64),
            &[new_pda_account.clone(), system_program.clone()],
            &[new_pda_signer_seeds],
        )?;

        invoke_signed(
            &system_instruction::assign(new_pda_account.key, owner),
            &[new_pda_account.clone(), system_program.clone()],
            &[new_pda_signer_seeds],
        )?;
    } else {
        invoke_signed(
            &system_instruction::create_account(
                payer.key,
                new_pda_account.key,
                rent.minimum_balance(space).max(1),
                space as u64,
                owner,
            ),
            &[
                payer.clone(),
                new_pda_account.clone(),
                system_program.clone(),
            ],
            &[new_pda_signer_seeds],
        )?;
    }

    Ok(())
}

fn is_transfer_memo_required(user_token_ai: &AccountInfo<'_>) -> Result<bool> {
    if user_token_ai.owner.eq(&anchor_spl::token::ID) {
        return Ok(false);
    }

    let account_data = user_token_ai.try_borrow_data()?;
    let token_account_unpacked =
        StateWithExtensions::<spl_token_2022::state::Account>::unpack(&account_data)?;

    let memo_transfer_ext =
        token_account_unpacked.get_extension::<extension::memo_transfer::MemoTransfer>();

    if let Ok(memo_transfer) = memo_transfer_ext {
        Ok(memo_transfer.require_incoming_transfer_memos.into())
    } else {
        Ok(false)
    }
}

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
    Ok(get_program_id(&token_mint_unpacked))
}

#[derive(Clone, Copy)]
pub struct MemoTransferContext<'a, 'info> {
    pub memo_program: &'a Program<'info, Memo>,
    pub memo: &'static [u8],
}

pub fn transfer_token<'c: 'info, 'info>(
    zap_authority: AccountInfo<'info>,
    token_mint: &InterfaceAccount<'info, Mint>,
    token_ledger_account: &InterfaceAccount<'info, TokenAccount>,
    receiver_token_account: &InterfaceAccount<'info, TokenAccount>,
    token_program: &Interface<'info, TokenInterface>,
    amount: u64,
    signers_seeds: &[&[&[u8]]],
    transfer_hook_accounts: &'c [AccountInfo<'info>],
    memo_transfer_context: Option<MemoTransferContext<'_, 'info>>,
) -> Result<()> {
    let destination_account = receiver_token_account.to_account_info();

    if let Some(memo_ctx) = memo_transfer_context {
        if is_transfer_memo_required(&destination_account)? {
            let ctx: CpiContext<'_, '_, '_, '_, BuildMemo> =
                CpiContext::new(memo_ctx.memo_program.to_account_info(), BuildMemo {});
            memo::build_memo(ctx, memo_ctx.memo)?;
        }
    }

    let mut instruction = spl_token_2022::instruction::transfer_checked(
        token_program.key,
        &token_ledger_account.key(),
        &token_mint.key(),
        &receiver_token_account.key(),
        &zap_authority.key(),
        &[],
        amount,
        token_mint.decimals,
    )?;

    let mut account_infos = vec![
        token_ledger_account.to_account_info(),
        token_mint.to_account_info(),
        receiver_token_account.to_account_info(),
        zap_authority.to_account_info(),
    ];

    if let Some(hook_program_id) = get_transfer_hook_program_id(token_mint)? {
        require!(
            transfer_hook_accounts.len() > 0,
            ZapError::MissingRemainingAccountForTransferHook
        );

        spl_transfer_hook_interface::onchain::add_extra_accounts_for_execute_cpi(
            &mut instruction,
            &mut account_infos,
            &hook_program_id,
            token_ledger_account.to_account_info(),
            token_mint.to_account_info(),
            receiver_token_account.to_account_info(),
            zap_authority.to_account_info(),
            amount,
            transfer_hook_accounts,
        )?;
    }

    invoke_signed(&instruction, &account_infos, signers_seeds)?;

    Ok(())
}
