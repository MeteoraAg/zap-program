use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program::invoke_signed},
    InstructionData,
};
use anchor_spl::token_interface::TokenAccount;
use damm_v2::types::SwapParameters;
use num_enum::{IntoPrimitive, TryFromPrimitive};

use crate::{
    const_pda,
    constants::{
        amm_program_id::{DAMM_V2, DLMM},
        ACTION_TYPE_INDEX, PAYLOAD_DATA_START_INDEX,
    },
    error::ZapError,
    parameters::ZapOutParametersDecoder,
    SwapDammV2Params, SwapDlmmParams,
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

#[event_cpi]
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
    fn get_instruction_data(&self, data: Vec<u8>) -> Result<Vec<u8>> {
        let action_type = ActionType::try_from(data[ACTION_TYPE_INDEX])
            .map_err(|_| ZapError::InvalidActionType)?;
        let payload = data[PAYLOAD_DATA_START_INDEX..].to_vec();

        let parsed_data = match action_type {
            ActionType::SwapDammV2 => {
                // validate amm program id
                require_keys_eq!(
                    self.amm_program.key(),
                    DAMM_V2,
                    ZapError::InvalidAmmProgramId
                );

                // decode payload data for swap damm v2 params
                let SwapDammV2Params { minimum_amount_out } = SwapDammV2Params::decode(payload)?;

                damm_v2::client::args::Swap {
                    params: SwapParameters {
                        amount_in: self.token_ledger_account.amount,
                        minimum_amount_out: minimum_amount_out,
                    },
                }
                .data()
            }
            ActionType::SwapDlmm => {
                // validate amm program id
                require_keys_eq!(self.amm_program.key(), DLMM, ZapError::InvalidAmmProgramId);
                // decode payload data for swap dlmm params
                let SwapDlmmParams {
                    minimum_amount_out,
                    remaining_accounts_info,
                } = SwapDlmmParams::decode(payload)?;

                dlmm::client::args::Swap2 {
                    amount_in: self.token_ledger_account.amount,
                    min_amount_out: minimum_amount_out,
                    remaining_accounts_info,
                }
                .data()
            }
        };
        Ok(parsed_data)
    }
}

pub fn handle_zap_out<'c: 'info, 'info>(
    ctx: Context<'_, '_, 'c, 'info, ZapOutCtx<'info>>,
    data: Vec<u8>,
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

    let data = ctx.accounts.get_instruction_data(data)?;
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

    // TODO emit event

    Ok(())
}
