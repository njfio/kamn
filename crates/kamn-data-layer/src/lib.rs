#![warn(missing_docs)]
//! Extracted data-layer shared helpers from `kamn-core`.

/// Shared SHA-256 helpers used by data-layer hash-chain contracts.
pub mod data_layer_hashing;
/// M10 archival retry projection contracts extracted from core data-layer module.
pub mod data_layer_m10_archival_retry;
/// M10 compliance projection seam contracts shared by extraction adapters.
pub mod data_layer_m10_compliance_projection_port;
/// M11 hardening matrix contracts for scenario tracking and operator readiness decisions.
pub mod data_layer_m11_hardening_readiness;

pub use data_layer_m10_archival_retry::*;
pub use data_layer_m10_compliance_projection_port::*;
pub use data_layer_m11_hardening_readiness::*;
