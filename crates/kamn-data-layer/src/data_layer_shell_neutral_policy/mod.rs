//! Shell-neutral orchestration and ratio-budget policy contracts.
//!
//! This module evaluates shell-neutral evidence and ratio-budget markers using
//! deterministic decision semantics and fail-closed threshold validation.

mod error;
mod evaluator;
mod types;

pub use error::*;
pub use evaluator::*;
pub use types::*;
