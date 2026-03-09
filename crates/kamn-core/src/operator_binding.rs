//! Compatibility re-exports for operator binding contracts extracted from `kamn-core`.

#[deprecated(
    since = "0.1.0",
    note = "use kamn_governance::operator_binding::* directly; kamn_core::operator_binding module shim is scheduled for removal in a follow-up extraction wave"
)]
pub use kamn_governance::operator_binding::*;
