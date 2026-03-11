const LIB_RS: &str = include_str!("../src/lib.rs");

fn assert_explicit_marker(marker: &str) {
    assert!(
        LIB_RS.contains(marker),
        "crate root missing explicit public-surface marker: {marker}"
    );
}

#[test]
fn crate_root_rejects_glob_reexports() {
    assert!(
        !LIB_RS.contains("::*;"),
        "crate root must not use broad glob re-exports"
    );
}

#[test]
fn crate_root_contains_reviewed_explicit_public_surface_markers() {
    for marker in [
        "pub use data_layer_m10_archival_retry::{",
        "DataLayerM10ArchivalRetryError",
        "data_layer_m10_project_archival_retry_decision",
        "pub use data_layer_m10_partition_month_policy::{",
        "DataLayerM10PartitionMonthPolicyError",
        "pub use data_layer_m10_partition_registry_state_machine::{",
        "DataLayerM10PartitionRegistryStateMachine",
        "pub use data_layer_m7_billing_reconciliation::{",
        "DataLayerM7BillingReconciliationDecision",
        "pub use data_layer_m11_closure_evidence::{",
        "DataLayerM11ClosureEvidenceReport",
        "pub use data_layer_prd_critical_scenario_conformance::{",
        "DataLayerPrdCriticalScenarioConformanceReport",
    ] {
        assert_explicit_marker(marker);
    }
}
