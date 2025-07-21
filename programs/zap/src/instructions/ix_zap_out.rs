use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program::invoke_signed},
};
use anchor_spl::token_interface::TokenAccount;
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{
    const_pda,
    constants::amm_program_id::{DAMM_V2, DLMM},
    error::ZapError,
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

impl<'info> ZapOutCtx<'info> {
    fn get_instruction_data(
        &self,
        action_type: ActionType,
        payload_data: &[u8],
    ) -> Result<Vec<u8>> {
        let instruction_discriminator = match action_type {
            ActionType::SwapDammV2 => {
                // validate amm program id
                require_keys_eq!(
                    self.amm_program.key(),
                    DAMM_V2,
                    ZapError::InvalidAmmProgramId
                );

                damm_v2::client::args::Swap::DISCRIMINATOR
            }
            ActionType::SwapDlmm => {
                // validate amm program id
                require_keys_eq!(self.amm_program.key(), DLMM, ZapError::InvalidAmmProgramId);

                dlmm::client::args::Swap2::DISCRIMINATOR
            }
        };

        let mut data = Vec::with_capacity(256);
        data.extend_from_slice(instruction_discriminator);
        data.extend_from_slice(&self.token_ledger_account.amount.to_le_bytes());
        data.extend_from_slice(payload_data);

        Ok(data)
    }
}

pub fn handle_zap_out<'c: 'info, 'info>(
    ctx: Context<'_, '_, 'c, 'info, ZapOutCtx<'info>>,
    action_type: u8,
    payload_data: Vec<u8>,
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

    let action_type = ActionType::try_from(action_type).map_err(|_| ZapError::TypeCastFailed)?;
    let data = ctx
        .accounts
        .get_instruction_data(action_type, &payload_data)?;
    let signers_seeds = zap_authority_seeds!();

    invoke_signed(
        &Instruction {
            program_id: ctx.accounts.amm_program.key(),
            accounts,
            data,
        },
        &account_infos,
        &[&signers_seeds[..]],
    )?;

    Ok(())
}
