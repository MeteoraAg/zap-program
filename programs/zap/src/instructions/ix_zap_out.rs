use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program::invoke_signed},
};
use anchor_spl::{
    memo::Memo,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{
    const_pda, constants::ZAPOUT_TRANSFER_MEMO, error::ZapError, safe_math::SafeMath,
    transfer_token, MemoTransferContext,
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ZapOutParameters {
    pub percentage: u8,
    pub offset_amount_in: u8,
    pub transfer_hook_length: u8,
    pub min_amount_out: u64,
    pub padding: [u64; 8],
    pub payload_data: Vec<u8>,
}

impl ZapOutParameters {
    fn validate(&self) -> Result<()> {
        require!(
            self.percentage <= 100 && self.percentage > 0,
            ZapError::InvalidZapOutParameters
        );

        // TODO: check whether need more validate

        Ok(())
    }
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

    #[account(mut)]
    pub user_token_out_account: InterfaceAccount<'info, TokenAccount>,

    /// Token in mint
    #[account(
        mint::token_program = input_token_program,
    )]
    pub token_in_mint: InterfaceAccount<'info, Mint>,

    pub input_token_program: Interface<'info, TokenInterface>,

    /// CHECK:
    pub amm_program: UncheckedAccount<'info>,

    pub memo_program: Option<Program<'info, Memo>>,
}

impl<'info> ZapOutCtx<'info> {
    fn get_swap_amount(&self, percentage: u8) -> Result<u64> {
        let total_amount = self.token_ledger_account.amount;

        let swap_amount = if percentage == 100 {
            total_amount
        } else {
            total_amount.safe_mul(percentage.into())?.safe_div(100)?
        };

        Ok(swap_amount)
    }

    fn modify_instruction_data(
        &self,
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
}

pub fn handle_zap_out<'c: 'info, 'info>(
    ctx: Context<'_, '_, 'c, 'info, ZapOutCtx<'info>>,
    params: &ZapOutParameters,
) -> Result<()> {
    // validate params
    params.validate()?;

    let remaining_accounts = ctx.remaining_accounts;
    let transfer_hook_length = params.transfer_hook_length as usize;
    let accounts: Vec<AccountMeta> = remaining_accounts[transfer_hook_length..]
        .iter()
        .map(|acc| AccountMeta {
            pubkey: *acc.key,
            is_signer: acc.is_signer,
            is_writable: acc.is_writable,
        })
        .collect();

    let account_infos: Vec<AccountInfo> = remaining_accounts[transfer_hook_length..]
        .iter()
        .map(|acc| AccountInfo { ..acc.clone() })
        .collect();

    let signers_seeds = zap_authority_seeds!();
    // transfer token to user_token_in_account

    let memo_transfer_context = if transfer_hook_length > 0 {
        require!(
            ctx.accounts.memo_program.is_some(),
            ZapError::MissingMemoProgram
        );
        Some(MemoTransferContext {
            memo_program: ctx.accounts.memo_program.as_ref().unwrap(),
            memo: ZAPOUT_TRANSFER_MEMO,
        })
    } else {
        None
    };

    let transfer_hook_account = &remaining_accounts[..transfer_hook_length];

    require!(
        transfer_hook_account.len() == transfer_hook_length,
        ZapError::MissingRemainingAccountForTransferHook
    );

    // transfer from token_ledger_account to user_token_in_account
    transfer_token(
        ctx.accounts.zap_authority.to_account_info(),
        &ctx.accounts.token_in_mint,
        &ctx.accounts.token_ledger_account,
        &ctx.accounts.user_token_in_account,
        &ctx.accounts.input_token_program,
        ctx.accounts.token_ledger_account.amount,
        &[&signers_seeds[..]],
        memo_transfer_context,
        Some(transfer_hook_account),
    )?;

    ctx.accounts.user_token_in_account.reload()?;

    let swap_amount = ctx.accounts.get_swap_amount(params.percentage)?;

    let mut payload_data = params.payload_data.to_vec();
    ctx.accounts.modify_instruction_data(
        &mut payload_data,
        swap_amount,
        params.offset_amount_in.into(),
    )?;

    let user_token_out_pre_balance = ctx.accounts.user_token_out_account.amount;
    // invoke instruction to amm
    invoke_signed(
        &Instruction {
            program_id: ctx.accounts.amm_program.key(),
            accounts,
            data: payload_data,
        },
        &account_infos,
        &[&signers_seeds[..]],
    )?;

    ctx.accounts.user_token_out_account.reload()?;

    let user_token_out_post_balance = ctx.accounts.user_token_out_account.amount;
    let total_amount_out_after_swap =
        user_token_out_post_balance.safe_sub(user_token_out_pre_balance)?;

    // prevent slippage from swap instruction
    require!(
        total_amount_out_after_swap >= params.min_amount_out,
        ZapError::ExceededSlippage
    );

    Ok(())
}
