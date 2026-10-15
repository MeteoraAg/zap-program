use anchor_lang::constant;
use anchor_lang::prelude::Pubkey;

use zap_sdk::constants::{
    DAMM_V2, DAMM_V2_SWAP_DISC, DLMM, DLMM_SWAP2_DISC, JUP_V6, JUP_V6_ROUTE_V2_DISC,
    JUP_V6_SHARED_ACCOUNT_ROUTE_V2_DISC,
};
#[allow(deprecated)]
use zap_sdk::constants::{JUP_V6_ROUTE_DISC, JUP_V6_SHARED_ACCOUNT_ROUTE_DISC};

pub const INSTRUCTION_DISCRIMINATOR_SIZE: usize = 8;

pub const JUP_V6_SHARED_ACCOUNT_ROUTE_V2_ID_SIZE: usize = 1;

#[deprecated(
    note = "Jupiter deprecated `route` and `shared_accounts_route`. Use `route_v2` or `shared_accounts_route_v2`."
)]
/// Jupiter route and shared_accounts_route place in_amount behind the variable length
/// route_plan, and end with
/// - in_amount: u64
/// - quoted_out_amount: u64
/// - slippage_bps: u16
/// - platform_fee_bps: u8
/// so in_amount starts 8 + 8 + 2 + 1 = 19 bytes counting from the end of the payload.
pub const JUP_V6_LEGACY_ROUTE_IN_AMOUNT_OFFSET_FROM_END: usize = 8 + 8 + 2 + 1;

// Jupiter `route` and `shared_accounts_route` are deprecated in favor of their v2 variants.
#[allow(deprecated)]
#[constant]
pub const WHITELISTED_AMM_PROGRAMS: [(Pubkey, [u8; INSTRUCTION_DISCRIMINATOR_SIZE]); 6] = [
    (DAMM_V2, DAMM_V2_SWAP_DISC),
    (DLMM, DLMM_SWAP2_DISC),
    (JUP_V6, JUP_V6_ROUTE_DISC),
    (JUP_V6, JUP_V6_SHARED_ACCOUNT_ROUTE_DISC),
    (JUP_V6, JUP_V6_ROUTE_V2_DISC),
    (JUP_V6, JUP_V6_SHARED_ACCOUNT_ROUTE_V2_DISC),
];

#[constant]
pub const MAX_BASIS_POINT: u16 = 10_000;

pub mod seeds {
    use anchor_lang::constant;

    #[constant]
    pub const USER_LEDGER_PREFIX: &[u8] = b"user_ledger";
}
