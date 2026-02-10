use crate::{
    constants::{SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID, SPL_TOKEN_PROGRAM_ID},
    error::ProtozolZapError,
};
use pinocchio::pubkey::{find_program_address, Pubkey};

// Adapted from anchor_spl::token::accessor::amount
// https://github.com/solana-foundation/anchor/blob/2cb7ababa7dba3ac269fd2e60cfa06793ad2b989/spl/src/token.rs#L519
pub(crate) fn get_token_amount(token_account_data: &[u8]) -> Result<u64, ProtozolZapError> {
    if token_account_data.len() < 72 {
        return Err(ProtozolZapError::InvalidZapAccounts);
    }
    let mut amount_bytes = [0u8; 8];
    amount_bytes.copy_from_slice(&token_account_data[64..72]);
    Ok(u64::from_le_bytes(amount_bytes))
}

// Adatpted from spl_associated_token_account::get_associated_token_address
// https://github.com/solana-labs/solana-program-library/blob/72bb80d76de5628749e9f06ed1a36d3986dab2a2/associated-token-account/client/src/address.rs#L56
pub(crate) fn get_associated_token_address(wallet: &Pubkey, mint: &Pubkey) -> Pubkey {
    let seeds: &[&[u8]] = &[wallet, &SPL_TOKEN_PROGRAM_ID, mint];
    let (address, _bump) = find_program_address(seeds, &SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID);
    address
}
