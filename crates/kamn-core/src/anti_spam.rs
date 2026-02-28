//! Compatibility re-exports for anti-spam contracts extracted from `kamn-core`.

#[deprecated(
    since = "0.1.0",
    note = "use kamn_runtime_guards::anti_spam::* directly; kamn_core::anti_spam module shim is scheduled for removal in R61"
)]
pub use kamn_runtime_guards::anti_spam::*;
