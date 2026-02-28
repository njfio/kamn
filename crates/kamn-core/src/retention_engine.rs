//! Compatibility re-exports for retention-engine contracts extracted from `kamn-core`.

#[deprecated(
    since = "0.1.0",
    note = "use kamn_runtime_guards::retention_engine::* directly; kamn_core::retention_engine module shim is scheduled for removal in R61"
)]
pub use kamn_runtime_guards::retention_engine::*;
