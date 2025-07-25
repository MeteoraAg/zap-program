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

impl<'info> ZapOutCtx<'info> {
    fn build_payload_data(
        &self,
        payload_data: &[u8],
        discriminator: &[u8],
        start_index: usize,
        end_index: usize,
    ) -> Vec<u8> {
        let mut data = vec![];
        let amount_in = self.token_ledger_account.amount.to_le_bytes();
        data.extend_from_slice(&discriminator);
        data.extend_from_slice(&payload_data[..start_index]);
        data.extend_from_slice(&amount_in);
        data.extend_from_slice(&payload_data[end_index..]);

        data
    }
    fn get_instruction_data(
        &self,
        action_type: ActionType,
        payload_data: &[u8],
    ) -> Result<Vec<u8>> {
        let data = match action_type {
            ActionType::SwapDammV2 => {
                // validate amm program id
                require_keys_eq!(
                    self.amm_program.key(),
                    DAMM_V2,
                    ZapError::InvalidAmmProgramId
                );

                let discriminator = damm_v2::client::args::Swap::DISCRIMINATOR;
                self.build_payload_data(payload_data, discriminator, 0, 0)
            }
            ActionType::SwapDlmm => {
                // validate amm program id
                require_keys_eq!(self.amm_program.key(), DLMM, ZapError::InvalidAmmProgramId);

                let discriminator = dlmm::client::args::Swap2::DISCRIMINATOR;
                self.build_payload_data(payload_data, discriminator, 0, 0)
            }
            ActionType::SwapJupiterV6 => {
                // validate amm program id
                require_keys_eq!(
                    self.amm_program.key(),
                    JUP_V6,
                    ZapError::InvalidAmmProgramId
                );

                require!(
                    payload_data.len() > AMOUNT_IN_JUP_V6_REVERSE_OFFSET,
                    ZapError::InvalidDataLen
                );

                let discriminator = jup_v6::client::args::Route::DISCRIMINATOR;
                // Update amount data in payload_data to amount_in value
                let start_index = payload_data
                    .len()
                    .safe_sub(AMOUNT_IN_JUP_V6_REVERSE_OFFSET)?;
                let end_index = start_index.safe_add(8)?; // 8 bytes for amount_in
                self.build_payload_data(payload_data, discriminator, start_index, end_index)
            }
        };

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

    let action_type = ActionType::try_from(action_type).map_err(|_| ZapError::InvalidActionType)?;
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
