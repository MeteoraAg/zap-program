pub const ZAP_AUTHORITY_PREFIX: &[u8] = b"zap_authority";
pub const TOKEN_LEDGER_PREFIX: &[u8] = b"token_ledger";

pub const ACTION_TYPE_INDEX: usize = 0;
pub const PAYLOAD_DATA_START_INDEX: usize = 1;
pub const DAMM_V2_SWAP_DATA_PAYLOAD_LEN: usize = 8;
pub const DLMM_PAYLOAD_DATA_OFFSET: usize = 8;
pub const VEC_LENGTH_PREFIX_SIZE: usize = 4; // 4 bytes length prefix
pub const VEC_DATA_START_OFFSET: usize = 4; // Data starts after length prefix
pub const SLICE_SIZE: usize = 2; // accounts_type(1 byte) + length(1 byte)

pub mod amm_program_id {
    use anchor_lang::{prelude::Pubkey, solana_program::pubkey};

    pub const DAMM_V2: Pubkey = pubkey!("cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG");

    #[cfg(not(feature = "local"))]
    pub const DLMM: Pubkey = pubkey!("LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo");

    #[cfg(feature = "local")]
    pub const DLMM: Pubkey = pubkey!("LbVRzDTvBDEcrthxfZ4RL6yiq3uZw8bS6MwtdY6UhFQ");
}
