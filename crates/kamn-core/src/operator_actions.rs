//! Compatibility re-exports for operator action contracts extracted from `kamn-core`.

#[deprecated(
    since = "0.1.0",
    note = "use kamn_governance::operator_actions::* directly; kamn_core::operator_actions module shim is scheduled for removal in a follow-up extraction wave"
)]
pub use kamn_governance::operator_actions::*;
