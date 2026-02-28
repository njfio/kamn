//! Compatibility re-exports for delivery-guard contracts extracted from `kamn-core`.

#[deprecated(
    since = "0.1.0",
    note = "use kamn_runtime_guards::message_delivery_guards::* directly; kamn_core::message_delivery_guards module shim is scheduled for removal in R61"
)]
pub use kamn_runtime_guards::message_delivery_guards::*;
