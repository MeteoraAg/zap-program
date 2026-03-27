use solana_pubkey::Pubkey;

pub const JUP_V6: Pubkey = Pubkey::from_str_const("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4");
// https://github.com/MeteoraAg/zap-program/blob/4b67bfc64e5a023a1b74386be8b82c3908934a0b/idls/jupiter.json#L161
pub const JUP_V6_ROUTE_DISC: [u8; 8] = [229, 23, 203, 151, 122, 227, 173, 42];
// https://github.com/MeteoraAg/zap-program/blob/4b67bfc64e5a023a1b74386be8b82c3908934a0b/idls/jupiter.json#L270
pub const JUP_V6_SHARED_ACCOUNT_ROUTE_DISC: [u8; 8] = [193, 32, 155, 51, 65, 214, 156, 129];

pub const DLMM: Pubkey = Pubkey::from_str_const("LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo");
// https://github.com/MeteoraAg/zap-program/blob/4b67bfc64e5a023a1b74386be8b82c3908934a0b/idls/dlmm.json#L5242-L5251
pub const DLMM_SWAP2_DISC: [u8; 8] = [65, 75, 63, 76, 235, 91, 91, 136];

pub const DAMM_V2: Pubkey = Pubkey::from_str_const("cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG");
// https://github.com/MeteoraAg/zap-program/blob/4b67bfc64e5a023a1b74386be8b82c3908934a0b/idls/damm_v2.json#L2154
pub const DAMM_V2_SWAP_DISC: [u8; 8] = [248, 198, 158, 145, 225, 117, 135, 200];
