#![warn(missing_docs)]
//! Extracted data-layer shared helpers from `kamn-core`.

/// Shared SHA-256 helpers used by data-layer hash-chain contracts.
pub mod data_layer_hashing;
/// M10 archival retry projection contracts extracted from core data-layer module.
pub mod data_layer_m10_archival_retry;
/// M10 compliance-projection bookkeeping extracted from core.
pub mod data_layer_m10_compliance_projection_bookkeeping;
/// M10 compliance projection seam contracts shared by extraction adapters.
pub mod data_layer_m10_compliance_projection_port;
/// M10 partition month-id parsing and naming policy extracted from core.
pub mod data_layer_m10_partition_month_policy;
/// M10 deterministic partition registry lifecycle state machine extracted from core.
pub mod data_layer_m10_partition_registry_state_machine;
/// M10 phase-6 compliance seam contracts shared by extraction adapters.
pub mod data_layer_m10_phase6_compliance_port;
/// M10 phase-6 policy evaluator contracts extracted from core.
pub mod data_layer_m10_phase6_policy_evaluator;
/// M10 phase-6 runtime evidence projector contracts extracted from core.
pub mod data_layer_m10_phase6_runtime_evidence;
/// M11 hardening matrix contracts for scenario tracking and operator readiness decisions.
pub mod data_layer_m11_hardening_readiness;
/// M1 batch scheduler trigger policy extracted from core.
pub mod data_layer_m1_batch_scheduler;

pub use data_layer_m10_archival_retry::*;
pub use data_layer_m10_compliance_projection_bookkeeping::*;
pub use data_layer_m10_compliance_projection_port::*;
pub use data_layer_m10_partition_month_policy::*;
pub use data_layer_m10_partition_registry_state_machine::*;
pub use data_layer_m10_phase6_compliance_port::*;
pub use data_layer_m10_phase6_policy_evaluator::*;
pub use data_layer_m11_hardening_readiness::*;
pub use data_layer_m1_batch_scheduler::*;
