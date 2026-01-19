use anchor_lang::constant;
use anchor_lang::prelude::Pubkey;

use protocol_zap::constants::{
    DAMM_V2, DAMM_V2_SWAP_DISC, DLMM, DLMM_SWAP2_DISC, JUP_V6, JUP_V6_ROUTE_DISC,
    JUP_V6_SHARED_ACCOUNT_ROUTE_DISC,
};

#[constant]
pub const WHITELISTED_AMM_PROGRAMS: [(Pubkey, [u8; 8]); 4] = [
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
