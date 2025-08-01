use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program::invoke},
};
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{
    const_pda, constants::WHITELISTED_AMM_PROGRAMS, error::ZapError, safe_math::SafeMath,
    transfer_token,
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ZapOutParameters {
    pub percentage: u8,
    pub offset_amount_in: u16,
    pub transfer_hook_length: u8,
    pub payload_data: Vec<u8>,
}

impl ZapOutParameters {
    fn validate(&self) -> Result<()> {
        require!(
            self.percentage <= 100 && self.percentage > 0,
            ZapError::InvalidZapOutParameters
        );

        Ok(())
    }

    fn get_swap_amount(&self, total_amount: u64) -> Result<u64> {
        let swap_amount = if self.percentage == 100 {
            total_amount
        } else {
            let amount = u128::from(total_amount)
                .safe_mul(self.percentage.into())?
                .safe_div(100)?;
            u64::try_from(amount).map_err(|_| ZapError::TypeCastFailed)?
        };

        Ok(swap_amount)
    }
}

pub fn is_support_amm_program(amm_program: &Pubkey, discriminator: &[u8]) -> bool {
    WHITELISTED_AMM_PROGRAMS
        .iter()
        .any(|(program, disc)| program.eq(amm_program) && disc.eq(discriminator))
}

#[derive(Accounts)]
pub struct ZapOutCtx<'info> {
    /// CHECK: zap authority
    #[account(
        address = const_pda::zap_authority::ID,
    )]
    pub zap_authority: UncheckedAccount<'info>,

    #[account(mut)]
    pub token_ledger_account: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub user_token_in_account: InterfaceAccount<'info, TokenAccount>,

    /// Token in mint
    #[account(
        mint::token_program = input_token_program,
    )]
    pub token_in_mint: InterfaceAccount<'info, Mint>,

    pub input_token_program: Interface<'info, TokenInterface>,

    /// CHECK:
    pub amm_program: UncheckedAccount<'info>,
}

// Acknowledged: We are aware of memo transfer requirements for certain token accounts
// but v1 does not support it as very few token accounts currently use the memo transfer extension.

pub fn modify_instruction_data(
    payload_data: &mut Vec<u8>,
    amount_in: u64,
    offset_amount_in: usize,
) -> Result<()> {
    let amount_in_bytes = amount_in.to_le_bytes();
    let end_offset_index = offset_amount_in.safe_add(amount_in_bytes.len())?;

    require!(
        end_offset_index <= payload_data.len(),
        ZapError::InvalidOffset
    );
    payload_data.splice(
        offset_amount_in..end_offset_index,
        amount_in_bytes.iter().cloned(),
    );

    Ok(())
}

pub fn handle_zap_out<'c: 'info, 'info>(
    ctx: Context<'_, '_, 'c, 'info, ZapOutCtx<'info>>,
    params: &ZapOutParameters,
) -> Result<()> {
    // validate params
    params.validate()?;
    let disciminator = &params.payload_data[..8]; // first 8 bytes is discriminator
    require!(
        is_support_amm_program(ctx.accounts.amm_program.key, disciminator),
        ZapError::AmmIsNotSupported
    );
    let token_ledger_balance = ctx.accounts.token_ledger_account.amount;
    if token_ledger_balance == 0 {
        // skip if token ledger balance is zero
        return Ok(());
    }
    let transfer_hook_length = params.transfer_hook_length as usize;
    let transfer_hook_accounts = &ctx.remaining_accounts[..transfer_hook_length];
    let pre_balance_user_token_in = ctx.accounts.user_token_in_account.amount;
    // transfer from token_ledger_account to user_token_in_account
    // Acknowledged: With this design, users will be charged transfer fees twice if the token has the transfer fee extension enabled.
    // However, we can ignore this issue in the first version.
    transfer_token(
        ctx.accounts.zap_authority.to_account_info(),
        &ctx.accounts.token_in_mint,
        &ctx.accounts.token_ledger_account,
        &ctx.accounts.user_token_in_account,
        &ctx.accounts.input_token_program,
        token_ledger_balance,
        transfer_hook_accounts,
    )?;

    ctx.accounts.user_token_in_account.reload()?;
    let post_balance_user_token_in = ctx.accounts.user_token_in_account.amount;
    let total_amount = post_balance_user_token_in.safe_sub(pre_balance_user_token_in)?;

    let swap_amount = params.get_swap_amount(total_amount)?;

    if swap_amount > 0 {
        let mut payload_data = params.payload_data.to_vec();
        modify_instruction_data(
            &mut payload_data,
            swap_amount,
            params.offset_amount_in.into(),
        )?;

        let accounts: Vec<AccountMeta> = ctx.remaining_accounts[transfer_hook_length..]
            .iter()
            .map(|acc| AccountMeta {
                pubkey: *acc.key,
                is_signer: acc.is_signer,
                is_writable: acc.is_writable,
            })
            .collect();

        let account_infos: Vec<AccountInfo> = ctx.remaining_accounts[transfer_hook_length..]
            .iter()
            .map(|acc| AccountInfo { ..acc.clone() })
            .collect();
        // invoke instruction to amm
        invoke(
            &Instruction {
                program_id: ctx.accounts.amm_program.key(),
                accounts,
                data: payload_data,
            },
            &account_infos,
        )?;
    }

    Ok(())
}
