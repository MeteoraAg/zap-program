//! Macro functions
macro_rules! zap_authority_seeds {
    () => {
        &[
            crate::constants::ZAP_AUTHORITY_PREFIX,
            &[crate::const_pda::zap_authority::BUMP],
        ]
    };
}
