//! Compatibility re-exports for fairness-policy contracts extracted from `kamn-core`.

#[deprecated(
    since = "0.1.0",
    note = "use kamn_runtime_guards::fairness_policy::* directly; kamn_core::fairness_policy module shim is scheduled for removal in R61"
)]
pub use kamn_runtime_guards::fairness_policy::*;
