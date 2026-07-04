//! M9 realtime delivery contracts for presence and deterministic backpressure.
//!
//! This module models PRD M9 behavior as deterministic Rust contracts:
//! owner-scoped dispatch acknowledgements, scoped presence visibility, and
//! queue-cap backpressure escalation markers.

mod backpressure;
mod dispatch;
mod models;
mod presence;
mod validation;

#[cfg(test)]
mod tests;

pub use models::*;
