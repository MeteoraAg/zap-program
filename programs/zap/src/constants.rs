use anchor_lang::constant;
use anchor_lang::{prelude::Pubkey, pubkey};

#[constant]
pub const DAMM_V1: Pubkey = pubkey!("Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EQVn5UaB");
// https://github.com/MeteoraAg/zap-program/blob/main/idls/damm_v2.json#L3512-L3521
#[constant]
pub const DAMM_V1_SWAP_DISC: [u8; 8] = [248, 198, 158, 145, 225, 117, 135, 200];

#[constant]
pub const DAMM_V2: Pubkey = pubkey!("cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG");
// https://github.com/MeteoraAg/zap-program/blob/main/idls/damm_v2.json#L3512-L3521
#[constant]
pub const DAMM_V2_SWAP_DISC: [u8; 8] = [248, 198, 158, 145, 225, 117, 135, 200];

#[constant]
pub const JUP_V6: Pubkey = pubkey!("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4");
// https://github.com/MeteoraAg/zap-program/blob/main/idls/jup_v6.json#L14-L23
#[constant]
pub const JUP_V6_ROUTE_DISC: [u8; 8] = [229, 23, 203, 151, 122, 227, 173, 42];
// https://github.com/MeteoraAg/zap-program/blob/main/idls/jup_v6.json#L257-L266
#[constant]
pub const JUP_V6_SHARED_ACCOUNT_ROUTE_DISC: [u8; 8] = [193, 32, 155, 51, 65, 214, 156, 129];

pub const DLMM: Pubkey = pubkey!("LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo");
// https://github.com/MeteoraAg/zap-program/blob/main/idls/dlmm.json#L3413-L3422
#[constant]
pub const DLMM_SWAP2_DISC: [u8; 8] = [65, 75, 63, 76, 235, 91, 91, 136];

#[constant]
pub const WHITELISTED_AMM_PROGRAMS: [(Pubkey, [u8; 8]); 5] = [
    (DAMM_V1, DAMM_V1_SWAP_DISC),
    (DAMM_V2, DAMM_V2_SWAP_DISC),
    (DLMM, DLMM_SWAP2_DISC),
    (JUP_V6, JUP_V6_ROUTE_DISC),
    (JUP_V6, JUP_V6_SHARED_ACCOUNT_ROUTE_DISC),
];

#[constant]
pub const MAX_BASIS_POINT: u16 = 10_000;

pub mod seeds {
    use anchor_lang::constant;

    #[constant]
    pub const USER_LEDGER_PREFIX: &[u8] = b"user_ledger";
}
