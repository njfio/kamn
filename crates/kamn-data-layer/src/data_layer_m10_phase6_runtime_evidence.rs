//! M10 phase6 runtime evidence projector extracted from core.
//!
//! This module hosts deterministic projection logic that shapes one phase6
//! scheduler-cycle report plus runtime counters into canonical runtime evidence.

#[path = "data_layer_m10_phase6_runtime_evidence/projector.rs"]
mod projector;
#[path = "data_layer_m10_phase6_runtime_evidence/types.rs"]
mod types;

pub use projector::*;
pub use types::*;
