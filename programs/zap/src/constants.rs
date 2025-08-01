use anchor_lang::{prelude::Pubkey, pubkey};

pub const ZAP_AUTHORITY_PREFIX: &[u8] = b"zap_authority";
pub const TOKEN_LEDGER_PREFIX: &[u8] = b"token_ledger";

pub const DAMM_V2: Pubkey = pubkey!("cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG");
// https://github.com/MeteoraAg/zap-program/blob/main/idls/damm_v2.json#L3512-L3521
pub const DAMM_V2_SWAP_DISC: [u8; 8] = [248, 198, 158, 145, 225, 117, 135, 200];

pub const JUP_V6: Pubkey = pubkey!("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4");
// https://github.com/MeteoraAg/zap-program/blob/main/idls/jup_v6.json#L14-L23
pub const JUP_V6_ROUTE_DISC: [u8; 8] = [229, 23, 203, 151, 122, 227, 173, 42];
// https://github.com/MeteoraAg/zap-program/blob/main/idls/jup_v6.json#L257-L266
pub const JUP_V6_SHARED_ACCOUNT_ROUTE_DISC: [u8; 8] = [193, 32, 155, 51, 65, 214, 156, 129];

#[cfg(not(feature = "local"))]
pub const DLMM: Pubkey = pubkey!("LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo");
// https://github.com/MeteoraAg/zap-program/blob/main/idls/dlmm.json#L3413-L3422
pub const DLMM_SWAP2_DISC: [u8; 8] = [65, 75, 63, 76, 235, 91, 91, 136];
#[cfg(feature = "local")]
pub const DLMM: Pubkey = pubkey!("LbVRzDTvBDEcrthxfZ4RL6yiq3uZw8bS6MwtdY6UhFQ");

pub const WHITELISTED_AMM_PROGRAMS: [(Pubkey, [u8; 8]); 4] = [
    (DAMM_V2, DAMM_V2_SWAP_DISC),
    (DLMM, DLMM_SWAP2_DISC),
    (JUP_V6, JUP_V6_ROUTE_DISC),
    (JUP_V6, JUP_V6_SHARED_ACCOUNT_ROUTE_DISC),
];
