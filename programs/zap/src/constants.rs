pub const ZAP_AUTHORITY_PREFIX: &[u8] = b"zap_authority";
pub const TOKEN_LEDGER_PREFIX: &[u8] = b"token_ledger";

pub const ACTION_TYPE_INDEX: usize = 0;
pub const PAYLOAD_DATA_START_INDEX: usize = 1;

pub mod amm_program_id {
    use anchor_lang::{prelude::Pubkey, solana_program::pubkey};

    pub const DAMM_V2: Pubkey = pubkey!("cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG");

    #[cfg(not(feature = "local"))]
    pub const DLMM: Pubkey = pubkey!("LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo");

    #[cfg(feature = "local")]
    pub const DLMM: Pubkey = pubkey!("LbVRzDTvBDEcrthxfZ4RL6yiq3uZw8bS6MwtdY6UhFQ");
}
