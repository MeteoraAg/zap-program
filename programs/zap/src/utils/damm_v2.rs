use anchor_lang::prelude::*;

use crate::{constants::NUM_DAMM_V2_REMAINING_ACCOUNTS, error::ZapError};

pub struct DammV2SwapAccounts<'info> {
    pub pool_authority: &'info AccountInfo<'info>,
    pub pool: &'info AccountInfo<'info>,
    pub output_token_account: &'info AccountInfo<'info>,
    pub token_a_vault: &'info AccountInfo<'info>,
    pub token_b_vault: &'info AccountInfo<'info>,
    pub token_a_mint: &'info AccountInfo<'info>,
    pub token_b_mint: &'info AccountInfo<'info>,
    pub token_a_program: &'info AccountInfo<'info>,
    pub token_b_program: &'info AccountInfo<'info>,
    pub event_authority: &'info AccountInfo<'info>,
}

impl<'info> DammV2SwapAccounts<'info> {
    pub fn parse_remaining_accounts(
        remaining_accounts: &'info [AccountInfo<'info>],
    ) -> Result<DammV2SwapAccounts> {
        // validate total remaining accounts required for damm v2 swap instruction
        require!(
            remaining_accounts.len() == NUM_DAMM_V2_REMAINING_ACCOUNTS,
            ZapError::MissingDammV2RemainingAccount
        );

        let accounts = DammV2SwapAccounts {
            pool_authority: &remaining_accounts[0],
            pool: &remaining_accounts[1],
            output_token_account: &remaining_accounts[2],
            token_a_vault: &remaining_accounts[3],
            token_b_vault: &remaining_accounts[4],
            token_a_mint: &remaining_accounts[5],
            token_b_mint: &remaining_accounts[6],
            token_a_program: &remaining_accounts[7],
            token_b_program: &remaining_accounts[8],
            event_authority: &remaining_accounts[9],
        };

        Ok(accounts)
    }

    pub fn validate(&self) -> Result<()> {
        let pool: AccountLoader<'_, damm_v2::accounts::Pool> = AccountLoader::try_from(self.pool)?;
        let pool_state = pool.load()?;

        require_keys_eq!(
            pool_state.token_a_vault,
            self.token_a_vault.key(),
            ZapError::InvalidDammV2Accounts
        );

        require_keys_eq!(
            pool_state.token_b_vault,
            self.token_b_vault.key(),
            ZapError::InvalidDammV2Accounts
        );

        require_keys_eq!(
            pool_state.token_a_mint,
            self.token_a_mint.key(),
            ZapError::InvalidDammV2Accounts
        );

        require_keys_eq!(
            pool_state.token_b_mint,
            self.token_b_mint.key(),
            ZapError::InvalidDammV2Accounts
        );

        Ok(())
    }
}
