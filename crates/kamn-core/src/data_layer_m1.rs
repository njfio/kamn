//! M1 trust-anchor contracts for merkle batching, proof generation, and Kolme anchoring.
//!
//! This module builds on M0 content-hash records and provides deterministic
//! merkle root assembly, inclusion-proof verification, and an idempotent
//! anchoring worker that targets the existing Kolme runtime-commit client.

mod anchoring;
mod batch;
mod models;
mod support;
mod verification;
#[cfg(test)] mod tests;

pub use anchoring::DataLayerM1KolmeAnchoringWorker;
pub use batch::DataLayerM1MerkleBatch;
pub use models::*;
pub use verification::*;
