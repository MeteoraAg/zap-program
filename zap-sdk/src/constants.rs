use pinocchio::pubkey::Pubkey;
use pinocchio_pubkey::pubkey;

pub const JUP_V6_SHARED_ACCOUNT_ROUTE_AMOUNT_IN_REVERSE_OFFSET: usize = 1 + 2 + 8 + 8; // Due to jupiter parameters have dynamic length type (vec), we have to do parameters_data.length - JUP_V6_SHARED_ACCOUNT_ROUTE_AMOUNT_IN_REVERSE_OFFSET
pub const JUP_V6_SHARED_ACCOUNT_ROUTE_SOURCE_ACCOUNT_INDEX: usize = 3;
pub const JUP_V6_SHARED_ACCOUNT_ROUTE_DESTINATION_ACCOUNT_INDEX: usize = 6;

pub const JUP_V6_ROUTE_AMOUNT_IN_REVERSE_OFFSET: usize = 1 + 2 + 8 + 8;
pub const JUP_V6_ROUTE_SOURCE_ACCOUNT_INDEX: usize = 2;
pub const JUP_V6_ROUTE_DESTINATION_ACCOUNT_INDEX: usize = 4;

pub const DLMM_SWAP2_AMOUNT_IN_OFFSET: u16 = 8;
pub const DLMM_SWAP2_SOURCE_ACCOUNT_INDEX: usize = 4;
pub const DLMM_SWAP2_DESTINATION_ACCOUNT_INDEX: usize = 5;

pub const DAMM_V2_SWAP_AMOUNT_IN_OFFSET: u16 = 8;
pub const DAMM_V2_SWAP_SOURCE_ACCOUNT_INDEX: usize = 2;
pub const DAMM_V2_SWAP_DESTINATION_ACCOUNT_INDEX: usize = 3;

pub const JUP_V6: Pubkey = pubkey!("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4");
// https://github.com/MeteoraAg/zap-program/blob/4b67bfc64e5a023a1b74386be8b82c3908934a0b/idls/jupiter.json#L161
pub const JUP_V6_ROUTE_DISC: [u8; 8] = [229, 23, 203, 151, 122, 227, 173, 42];
// https://github.com/MeteoraAg/zap-program/blob/4b67bfc64e5a023a1b74386be8b82c3908934a0b/idls/jupiter.json#L270
pub const JUP_V6_SHARED_ACCOUNT_ROUTE_DISC: [u8; 8] = [193, 32, 155, 51, 65, 214, 156, 129];

pub const DLMM: Pubkey = pubkey!("LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo");
// https://github.com/MeteoraAg/zap-program/blob/4b67bfc64e5a023a1b74386be8b82c3908934a0b/idls/dlmm.json#L5242-L5251
pub const DLMM_SWAP2_DISC: [u8; 8] = [65, 75, 63, 76, 235, 91, 91, 136];

pub const DAMM_V2: Pubkey = pubkey!("cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG");
// https://github.com/MeteoraAg/zap-program/blob/4b67bfc64e5a023a1b74386be8b82c3908934a0b/idls/damm_v2.json#L2154
pub const DAMM_V2_SWAP_DISC: [u8; 8] = [248, 198, 158, 145, 225, 117, 135, 200];

pub const ZAP: Pubkey = pubkey!("zapvX9M3uf5pvy4wRPAbQgdQsM1xmuiFnkfHKPvwMiz");
pub const ZAP_OUT_DISC: [u8; 8] = [155, 108, 185, 112, 104, 210, 161, 64];

pub const USDC_ADDRESS: Pubkey = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");

pub const SOL_ADDRESS: Pubkey = pubkey!("So11111111111111111111111111111111111111112");

pub const MINTS_DISALLOWED_TO_ZAP_OUT: [Pubkey; 2] = [USDC_ADDRESS, SOL_ADDRESS];

pub const SPL_TOKEN_PROGRAM_ID: Pubkey = pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

pub const SPL_ASSOCIATED_TOKEN_ACCOUNT_PROGRAM_ID: Pubkey =
    pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
