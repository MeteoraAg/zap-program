//! Macro functions
macro_rules! zap_authority_seeds {
    ($bump:expr) => {
        &[b"zap_authority".as_ref(), &[$bump]]
    };
}
