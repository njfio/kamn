//! Compatibility facade for cross-chain receipt normalization extracted to `kamn-bridges`.

#[deprecated(
    since = "0.1.0",
    note = "use kamn_bridges::cross_chain_receipt::* directly; kamn_core::cross_chain_receipt module shim is scheduled for removal in R61"
)]
pub use kamn_bridges::cross_chain_receipt::*;
