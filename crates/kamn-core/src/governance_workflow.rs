//! Compatibility re-exports for governance workflow contracts extracted from `kamn-core`.

#[deprecated(
    since = "0.1.0",
    note = "use kamn_governance::governance_workflow::* directly; kamn_core::governance_workflow module shim is scheduled for removal in a follow-up extraction wave"
)]
pub use kamn_governance::governance_workflow::*;
