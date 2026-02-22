#![warn(missing_docs)]
//! Shared agent library for authenticated KAMN operations.

/// Authentication helpers.
pub mod auth;
/// Typed service client facade.
pub mod client;
/// Envelope construction helpers.
pub mod envelope;
/// Error taxonomy.
pub mod errors;
/// Identity utilities.
pub mod identity;
/// Kolme proof verification adapter.
pub mod kolme;
/// Monotonic nonce tracking.
pub mod nonce;

/// Top-level agent handle facade.
pub struct KamnAgentHandle;
