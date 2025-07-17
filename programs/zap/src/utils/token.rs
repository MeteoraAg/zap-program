use anchor_lang::{
    prelude::*,
    solana_program::{
        program::{invoke, invoke_signed},
        system_instruction,
    },
};

use anchor_spl::token_2022::{
    get_account_data_size, spl_token_2022::extension::ExtensionType, GetAccountDataSize,
};

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
