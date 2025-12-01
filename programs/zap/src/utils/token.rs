use anchor_lang::prelude::*;
use anchor_spl::{
    token::Token,
    token_2022::spl_token_2022::{
        self,
        extension::{
            self, transfer_fee::TransferFee, BaseStateWithExtensions, StateWithExtensions,
        },
    },
    token_interface::Mint,
};
use damm_v2::token::TransferFeeExcludedAmount;

use crate::error::ZapError;

pub struct TransferFeeCalculator {
    pub epoch_transfer_fee: TransferFee,
    // cache this, so we could save compute unit for non transfer fee token mint
    pub no_transfer_fee_extension: bool,
}

impl TransferFeeCalculator {
    pub fn calculate_transfer_fee_excluded_amount(
        &self,
        amount: u64,
    ) -> Result<TransferFeeExcludedAmount> {
        if self.no_transfer_fee_extension {
            Ok(TransferFeeExcludedAmount {
                amount,
                transfer_fee: 0,
            })
        } else {
            let transfer_fee = self
                .epoch_transfer_fee
                .calculate_fee(amount)
                .ok_or_else(|| ZapError::MathOverflow)?;
            let transfer_fee_excluded_amount = amount
                .checked_sub(transfer_fee)
                .ok_or_else(|| ZapError::MathOverflow)?;
            return Ok(TransferFeeExcludedAmount {
                amount: transfer_fee_excluded_amount,
                transfer_fee,
            });
        }
    }
}

pub fn new_transfer_fee_calculator<'info>(
    token_mint: &InterfaceAccount<'info, Mint>,
) -> Result<TransferFeeCalculator> {
    let token_mint_info = token_mint.to_account_info();
    if *token_mint_info.owner == Token::id() {
        return Ok(TransferFeeCalculator {
            epoch_transfer_fee: TransferFee::default(),
            no_transfer_fee_extension: true,
        });
    }

    let token_mint_data = token_mint_info.try_borrow_data()?;
    let token_mint_unpacked =
        StateWithExtensions::<spl_token_2022::state::Mint>::unpack(&token_mint_data)?;
    if let Ok(transfer_fee_config) =
        token_mint_unpacked.get_extension::<extension::transfer_fee::TransferFeeConfig>()
    {
        let epoch = Clock::get()?.epoch;
        return Ok(TransferFeeCalculator {
            epoch_transfer_fee: *transfer_fee_config.get_epoch_fee(epoch),
            no_transfer_fee_extension: false,
        });
    } else {
        return Ok(TransferFeeCalculator {
            epoch_transfer_fee: TransferFee::default(),
            no_transfer_fee_extension: true,
        });
    }
}
