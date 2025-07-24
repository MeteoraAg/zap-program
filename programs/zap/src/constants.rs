pub const ZAP_AUTHORITY_PREFIX: &[u8] = b"zap_authority";
pub const TOKEN_LEDGER_PREFIX: &[u8] = b"token_ledger";

// Offset for amount_in in reverse order of jupiter Route instruction data:
// amount_in(u64) + quoted_out_amount(64) + slippage_bps(u16) + platform_fee_bps(u8) = 19 bytes
pub const AMOUNT_IN_REVERSE_OFFSET: usize = 19;

pub mod amm_program_id {
    use anchor_lang::{prelude::Pubkey, solana_program::pubkey};

    pub const DAMM_V2: Pubkey = pubkey!("cpamdpZCGKUy5JxQXB4dcpGPiikHawvSWAd6mEn1sGG");

    pub const JUP_V6: Pubkey = pubkey!("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4");

    #[cfg(not(feature = "local"))]
    pub const DLMM: Pubkey = pubkey!("LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo");

    #[cfg(feature = "local")]
    pub const DLMM: Pubkey = pubkey!("LbVRzDTvBDEcrthxfZ4RL6yiq3uZw8bS6MwtdY6UhFQ");
}
