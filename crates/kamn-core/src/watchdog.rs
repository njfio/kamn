//! Compatibility re-exports for watchdog contracts extracted from `kamn-core`.

#[deprecated(
    since = "0.1.0",
    note = "use kamn_runtime_guards::watchdog::* directly; kamn_core::watchdog module shim is scheduled for removal in R61"
)]
pub use kamn_runtime_guards::watchdog::*;
