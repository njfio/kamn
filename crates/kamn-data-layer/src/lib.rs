#![warn(missing_docs)]
//! Extracted data-layer shared helpers from `kamn-core`.

/// Shared SHA-256 helpers used by data-layer hash-chain contracts.
pub mod data_layer_hashing;
/// M11 hardening matrix contracts for scenario tracking and operator readiness decisions.
pub mod data_layer_m11_hardening_readiness;

pub use data_layer_m11_hardening_readiness::*;
