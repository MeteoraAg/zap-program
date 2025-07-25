use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program::invoke_signed},
};
use anchor_spl::token_interface::TokenAccount;
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{
    const_pda,
    constants::{
        amm_program_id::{DAMM_V2, DLMM, JUP_V6},
        AMOUNT_IN_JUP_V6_REVERSE_OFFSET,
    },
    error::ZapError,
    safe_math::SafeMath,
};

#[repr(u8)]
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    IntoPrimitive,
    TryFromPrimitive,
    AnchorDeserialize,
    AnchorSerialize,
)]
pub enum ActionType {
    SwapDammV2,
    SwapDlmm,
    SwapJupiterV6,
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

    /// CHECK:
    pub amm_program: UncheckedAccount<'info>,
}

fn modify_payload_data(
    payload_data: &mut Vec<u8>,
    discriminator: &[u8],
    amount_in: u64,
    index: usize,
) {
    payload_data.splice(0..0, discriminator.iter().cloned());
    payload_data.splice(index..index, amount_in.to_le_bytes().iter().cloned());
}

impl<'info> ZapOutCtx<'info> {
    fn validate_and_modify_instruction_data(
        &self,
        action_type: ActionType,
        payload_data: &mut Vec<u8>,
    ) -> Result<()> {
        let amount_in = self.token_ledger_account.amount;
        match action_type {
            ActionType::SwapDammV2 => {
                // validate amm program id
                require_keys_eq!(
                    self.amm_program.key(),
                    DAMM_V2,
                    ZapError::InvalidAmmProgramId
                );

                let discriminator = damm_v2::client::args::Swap::DISCRIMINATOR;
                modify_payload_data(payload_data, discriminator, amount_in, discriminator.len());
            }
            ActionType::SwapDlmm => {
                // validate amm program id
                require_keys_eq!(self.amm_program.key(), DLMM, ZapError::InvalidAmmProgramId);

                let discriminator = dlmm::client::args::Swap2::DISCRIMINATOR;
                modify_payload_data(payload_data, discriminator, amount_in, discriminator.len());
            }
            ActionType::SwapJupiterV6 => {
                // validate amm program id
                require_keys_eq!(
                    self.amm_program.key(),
                    JUP_V6,
                    ZapError::InvalidAmmProgramId
                );

                let discriminator = jup_v6::client::args::Route::DISCRIMINATOR;
                // Update amount data in payload_data to amount_in value
                let index = payload_data
                    .len()
                    .safe_sub(AMOUNT_IN_JUP_V6_REVERSE_OFFSET)?
                    .safe_add(discriminator.len())?;
                modify_payload_data(payload_data, discriminator, amount_in, index);
            }
        };
        Ok(())
    }
}

pub fn handle_zap_out<'c: 'info, 'info>(
    ctx: Context<'_, '_, 'c, 'info, ZapOutCtx<'info>>,
    action_type: u8,
    payload_data: &[u8],
) -> Result<()> {
    let accounts: Vec<AccountMeta> = ctx
        .remaining_accounts
        .iter()
        .map(|acc| {
            let is_signer = acc.key == &ctx.accounts.zap_authority.key();
            AccountMeta {
                pubkey: *acc.key,
                is_signer: is_signer,
                is_writable: acc.is_writable,
            }
        })
        .collect();

    let account_infos: Vec<AccountInfo> = ctx
        .remaining_accounts
        .iter()
        .map(|acc| AccountInfo { ..acc.clone() })
        .collect();

    let action_type = ActionType::try_from(action_type).map_err(|_| ZapError::InvalidActionType)?;
    let mut payload_data = payload_data.to_vec();
    ctx.accounts
        .validate_and_modify_instruction_data(action_type, &mut payload_data)?;
    let signers_seeds = zap_authority_seeds!();

    invoke_signed(
        &Instruction {
            program_id: ctx.accounts.amm_program.key(),
            accounts,
            data: payload_data,
        },
        &account_infos,
        &[&signers_seeds[..]],
    )?;

    Ok(())
}
