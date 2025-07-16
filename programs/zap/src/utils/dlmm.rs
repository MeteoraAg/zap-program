use anchor_lang::prelude::*;

use crate::error::ZapError;

pub struct DlmmSwapAccounts<'info> {
    pub lb_pair: &'info AccountInfo<'info>,
    pub reserve_x: &'info AccountInfo<'info>,
    pub reserve_y: &'info AccountInfo<'info>,
    pub user_token_out: &'info AccountInfo<'info>,
    pub token_x_mint: &'info AccountInfo<'info>,
    pub token_y_mint: &'info AccountInfo<'info>,
    pub oracle: &'info AccountInfo<'info>,
    pub event_authority: &'info AccountInfo<'info>,
    pub token_x_program: &'info AccountInfo<'info>,
    pub token_y_program: &'info AccountInfo<'info>,
    pub memo_program: &'info AccountInfo<'info>,
    pub bin_array_bitmap_extension: Option<&'info AccountInfo<'info>>,
    pub host_fee_in: Option<&'info AccountInfo<'info>>,
}

impl<'info> DlmmSwapAccounts<'info> {
    // pub fn parse_remaining_accounts(
    //     remaining_accounts: &'info [AccountInfo<'info>],
    // ) -> Result<DlmmSwapAccounts> {

    // }

    pub fn validate(&self) -> Result<()> {
        Ok(())
    }
}
