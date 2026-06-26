//! M8 compliance lifecycle contracts for crypto-shredding and retention controls.
//!
//! This module models PRD M8 behavior as deterministic Rust contracts:
//! owner-scoped message retention windows, legal-hold precedence, and
//! irreversible CEK shredding markers while preserving append-only integrity.

mod errors;
mod lifecycle;
mod models;
mod policy;
mod registry;
mod tests;

pub use errors::*;
pub use models::*;
pub use policy::*;
