//! M4 escrow integration contracts for transitions, scoped visibility, and settlement evidence reconciliation.
//!
//! This module models the PRD M4 escrow surface as deterministic Rust contracts:
//! escrow lifecycle transitions, dispute-aware participant/auditor message visibility,
//! and append-only settlement evidence storage with hash-chain verification.

mod models;
mod settlement_evidence;
mod transitions;
mod validation;
mod visibility;

pub use models::*;

#[cfg(test)]
mod tests;
