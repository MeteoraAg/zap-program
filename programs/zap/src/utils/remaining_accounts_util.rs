use crate::error::ZapError;
use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum AccountsType {
    TransferHook,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RemainingAccountsSlice {
    pub accounts_type: AccountsType,
    pub length: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct RemainingAccountsInfo {
    pub slices: Vec<RemainingAccountsSlice>,
}

impl RemainingAccountsInfo {
    pub fn total_length(&self) -> u8 {
        self.slices.iter().map(|s| s.length).sum()
    }
}

#[derive(Debug, Default)]
pub struct ParsedRemainingAccounts<'a, 'info> {
    pub transfer_hook: Option<&'a [AccountInfo<'info>]>,
}

/// Parse remaining accounts by consume all the transfer hooks related accounts.
pub fn parse_remaining_accounts<'a, 'info>(
    remaining_accounts: &mut &'a [AccountInfo<'info>],
    remaining_accounts_slice: &[RemainingAccountsSlice],
    valid_accounts_type_list: &[AccountsType],
) -> Result<ParsedRemainingAccounts<'a, 'info>> {
    let mut parsed_remaining_accounts = ParsedRemainingAccounts::default();

    if remaining_accounts_slice.is_empty() {
        return Ok(ParsedRemainingAccounts::default());
    }

    for slice in remaining_accounts_slice.iter() {
        if !valid_accounts_type_list.contains(&slice.accounts_type) {
            return Err(ZapError::InvalidRemainingAccountSlice.into());
        }

        if slice.length == 0 {
            continue;
        }

        if remaining_accounts.len() < slice.length as usize {
            return Err(ZapError::InsufficientRemainingAccounts.into());
        }

        let end_idx = slice.length as usize;
        let accounts = &remaining_accounts[0..end_idx];
        *remaining_accounts = &remaining_accounts[end_idx..];

        match slice.accounts_type {
            AccountsType::TransferHook => {
                if parsed_remaining_accounts.transfer_hook.is_some() {
                    return Err(ZapError::DuplicatedRemainingAccountTypes.into());
                }
                parsed_remaining_accounts.transfer_hook = Some(accounts);
            }
        }
    }

    Ok(parsed_remaining_accounts)
}
