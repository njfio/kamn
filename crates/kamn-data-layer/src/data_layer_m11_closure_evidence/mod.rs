//! M11 closure-evidence acceptance contracts.
//!
//! This module composes extracted hardening and PRD conformance reports into a
//! deterministic release-closure acceptance report.

mod error;
mod evaluator;
mod types;

pub use error::*;
pub use evaluator::*;
pub use types::*;
