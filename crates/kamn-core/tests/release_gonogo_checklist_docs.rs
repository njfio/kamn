const CHECKLIST: &str = include_str!("../../../docs/foundation/release-gonogo-checklist.md");

#[test]
fn checklist_contains_preflight_gates() {
    assert!(CHECKLIST.contains("## Preflight Gates"));
    assert!(CHECKLIST.contains("Migration plan reviewed and signed"));
    assert!(CHECKLIST.contains("Compatibility matrix validated"));
    assert!(CHECKLIST.contains("CI fast gate and deferred deep lane both green"));
    assert!(CHECKLIST.contains("Rollback runbook version pinned"));
}

#[test]
fn checklist_contains_dry_run_workflow() {
    assert!(CHECKLIST.contains("## Deterministic Dry-Run Workflow"));
    assert!(CHECKLIST.contains("1. Create release candidate tag"));
    assert!(CHECKLIST.contains("2. Rehearse migration on staging snapshot"));
    assert!(CHECKLIST.contains("3. Execute bounded smoke and invariant suites"));
    assert!(CHECKLIST.contains("4. Capture and sign dry-run evidence bundle"));
}

#[test]
fn checklist_contains_go_no_go_evidence_template() {
    assert!(CHECKLIST.contains("## Go/No-Go Evidence Template"));
    assert!(CHECKLIST.contains("Release candidate:"));
    assert!(CHECKLIST.contains("Schema target version:"));
    assert!(CHECKLIST.contains("Rollback trigger status:"));
    assert!(CHECKLIST.contains("Final decision: GO | NO-GO"));
}

#[test]
fn checklist_contains_machine_readable_bundle_contract() {
    assert!(CHECKLIST.contains("## Machine-Readable Evidence Bundle Contract"));
    assert!(CHECKLIST.contains("generate_gonogo_evidence_bundle.sh"));
    assert!(CHECKLIST.contains("check_gonogo_evidence_policy.sh"));
    assert!(CHECKLIST.contains("run_gonogo_evidence_contract_lane.sh"));
    assert!(CHECKLIST.contains("run_gonogo_evidence_deep_lane.sh"));
}

#[test]
fn checklist_contains_staging_rehearsal_contract() {
    assert!(CHECKLIST.contains("## Staging Deploy + Rollback Rehearsal Contract"));
    assert!(CHECKLIST.contains("generate_staging_rehearsal_bundle.sh"));
    assert!(CHECKLIST.contains("check_staging_rehearsal_policy.sh"));
    assert!(CHECKLIST.contains("run_staging_rehearsal_contract_lane.sh"));
    assert!(CHECKLIST.contains("run_staging_rehearsal_deep_lane.sh"));
}

#[test]
fn checklist_contains_durable_guard_recovery_evidence() {
    assert!(CHECKLIST.contains("## Durable Guard Migration + Recovery Matrix Evidence"));
    assert!(CHECKLIST.contains("run_durable_guard_recovery_contract_lane.sh"));
    assert!(CHECKLIST.contains("run_durable_guard_recovery_deep_lane.sh"));
    assert!(CHECKLIST.contains("performance_durable_guard_recovery_contract_lane_budget"));
    assert!(CHECKLIST.contains("performance_durable_guard_recovery_matrix_deep_lane"));
    assert!(CHECKLIST.contains("performance_bundle_contract_lane_budget"));
    assert!(CHECKLIST.contains("performance_bundle_store_deep_lane_stress"));
}

#[test]
fn regression_requires_rollback_precheck_in_checklist() {
    // Regression: #173
    assert!(CHECKLIST.contains("Rollback precheck result: PASS"));
}

#[test]
fn regression_requires_staging_rehearsal_mismatch_guard() {
    // Regression: #623
    assert!(CHECKLIST.contains(
        "rollback target hash mismatch and incomplete rehearsal evidence force `NO-GO` (`Regression: #623`)."
    ));
}
