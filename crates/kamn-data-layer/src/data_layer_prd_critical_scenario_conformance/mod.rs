//! PRD critical-scenario conformance contracts (`18.2`, scenarios `62..71`).
//!
//! This module keeps scenario recording and conformance evaluation deterministic
//! and side-effect free so runtime policy and closure-evidence projections can
//! share one fail-closed contract surface.

mod error;
mod helpers;
mod matrix;
mod types;

pub use error::*;
pub use matrix::*;
pub use types::*;

pub(crate) const DATA_LAYER_PRD_REQUIRED_CRITICAL_SCENARIO_IDS: [u8; 10] =
    [62, 63, 64, 65, 66, 67, 68, 69, 70, 71];
