use pinocchio::pubkey::Pubkey;
use pinocchio_pubkey::pubkey;

pub const ZAP_OUT_ACCOUNTS_LEN: usize = 2;

pub const JUP_V6_SHARED_ACCOUNT_ROUTE_AMOUNT_IN_REVERSE_OFFSET: usize = 1 + 2 + 8 + 8; // Due to jupiter parameters have dynamic length type (vec), we have to do parameters_data.length - JUP_V6_SHARED_ACCOUNT_ROUTE_AMOUNT_IN_REVERSE_OFFSET
pub const JUP_V6_SHARED_ACCOUNT_ROUTE_SOURCE_ACCOUNT_INDEX: usize = 3;
pub const JUP_V6_SHARED_ACCOUNT_ROUTE_DESTINATION_ACCOUNT_INDEX: usize = 6;

pub const JUP_V6_ROUTE_AMOUNT_IN_REVERSE_OFFSET: usize = 1 + 2 + 8 + 8;
pub const JUP_V6_ROUTE_SOURCE_ACCOUNT_INDEX: usize = 2;
pub const JUP_V6_ROUTE_DESTINATION_ACCOUNT_INDEX: usize = 4;
// the offset of the first swap accounts in the route plan when called through ix_zap_out.
// 2 (ix_zap_out) + 9 (jup route named accounts) + 1 (jup swap_program that is consumed in extract_swap_program)
// ref for 9 named accounts: https://github.com/jup-ag/jupiter-aggregator-program/blob/e583ab6619f4646b4d7a0e2514aec62ae9fb62ec/programs/jupiter/src/lib.rs#L410
// ref for swap_program: https://github.com/jup-ag/jupiter-aggregator-program/blob/e583ab6619f4646b4d7a0e2514aec62ae9fb62ec/programs/jupiter/src/process_swap.rs#L3455
pub const JUP_V6_ROUTE_FIRST_SWAP_ACCOUNTS_OFFSET: usize = ZAP_OUT_ACCOUNTS_LEN + 9 + 1;

pub const DLMM_SWAP2_AMOUNT_IN_OFFSET: u16 = 8;
pub const DLMM_SWAP2_SOURCE_ACCOUNT_INDEX: usize = 4;
pub const DLMM_SWAP2_DESTINATION_ACCOUNT_INDEX: usize = 5;

pub const DAMM_V2_SWAP_AMOUNT_IN_OFFSET: u16 = 8;
pub const DAMM_V2_SWAP_SOURCE_ACCOUNT_INDEX: usize = 2;
pub const DAMM_V2_SWAP_DESTINATION_ACCOUNT_INDEX: usize = 3;

pub const ZAP: Pubkey = pubkey!("zapvX9M3uf5pvy4wRPAbQgdQsM1xmuiFnkfHKPvwMiz");
pub const ZAP_OUT_DISC: [u8; 8] = [155, 108, 185, 112, 104, 210, 161, 64];

pub const USDC_ADDRESS: Pubkey = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");

pub const SOL_ADDRESS: Pubkey = pubkey!("So11111111111111111111111111111111111111112");

pub const MINTS_DISALLOWED_TO_ZAP_OUT: [Pubkey; 2] = [USDC_ADDRESS, SOL_ADDRESS];

pub const SPL_TOKEN_PROGRAM_ID: Pubkey = pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

pub const SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID: Pubkey =
    pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
