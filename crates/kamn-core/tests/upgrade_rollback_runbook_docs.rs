const RUNBOOK: &str = include_str!("../../../docs/foundation/upgrade-rollback-runbook.md");

#[test]
fn runbook_contains_rollback_triggers() {
    assert!(RUNBOOK.contains("## Rollback Triggers"));
    assert!(RUNBOOK.contains("State migration checksum mismatch"));
    assert!(RUNBOOK.contains("Quorum health degraded below threshold"));
    assert!(RUNBOOK.contains("Critical post-upgrade verification failure"));
}

#[test]
fn runbook_contains_deterministic_rollback_procedure() {
    assert!(RUNBOOK.contains("## Rollback Procedure"));
    assert!(RUNBOOK.contains("1. Freeze upgrade pipeline and block new proposals"));
    assert!(RUNBOOK.contains("2. Confirm rollback trigger evidence"));
    assert!(RUNBOOK.contains("3. Restore last known-good state snapshot"));
    assert!(RUNBOOK.contains("4. Rehydrate node roles with pinned release image"));
    assert!(RUNBOOK.contains("5. Re-run migration consistency checks"));
    assert!(RUNBOOK.contains("6. Resume controlled traffic"));
}

#[test]
fn runbook_contains_post_upgrade_verification_checklist() {
    assert!(RUNBOOK.contains("## Post-Upgrade Verification Checklist"));
    assert!(RUNBOOK.contains("App-state schema version matches expected target"));
    assert!(RUNBOOK.contains("Processor, Listener, and Approver roles report healthy wiring"));
    assert!(RUNBOOK.contains("No stale state-hash acceptance detected"));
}

#[test]
fn runbook_contains_failure_simulation_scenarios() {
    assert!(RUNBOOK.contains("## Failure Simulation Scenarios"));
    assert!(RUNBOOK.contains("Schema mismatch rollback drill"));
    assert!(RUNBOOK.contains("Partial node upgrade divergence drill"));
    assert!(RUNBOOK.contains("Quorum degradation during upgrade drill"));
}

#[test]
fn runbook_contains_watchdog_incident_response_flow() {
    assert!(RUNBOOK.contains("## Watchdog Incident Response Flow"));
    assert!(RUNBOOK.contains("1. Confirm incident attestation severity and fingerprint."));
    assert!(RUNBOOK.contains("2. Capture incident evidence payload before mitigation."));
    assert!(RUNBOOK.contains("3. Execute containment actions from the approved response tier."));
    assert!(RUNBOOK.contains("4. Run rollback procedure when severity is `critical`."));
    assert!(RUNBOOK.contains("5. Record closure summary with deterministic incident fields."));
}

#[test]
fn runbook_contains_dr_evidence_and_slo_gate_contract() {
    assert!(RUNBOOK.contains("## DR Drill Evidence and Release SLO Gate Contract"));
    assert!(RUNBOOK.contains("generate_dr_evidence_bundle.sh"));
    assert!(RUNBOOK.contains("check_release_slo_gates.sh"));
    assert!(RUNBOOK.contains("run_dr_evidence_contract_lane.sh"));
    assert!(RUNBOOK.contains("run_dr_evidence_deep_lane.sh"));
}

#[test]
fn runbook_contains_live_network_pilot_rollback_evidence_gate() {
    assert!(RUNBOOK.contains("## Live-Network Pilot Rollback Evidence Gate"));
    assert!(RUNBOOK.contains("run_live_network_pilot_deep_lane.sh"));
    assert!(RUNBOOK.contains("check_live_network_pilot_artifact_summary_policy.sh"));
    assert!(RUNBOOK.contains("run_live_network_pilot_deep_contract_lane.sh"));
}

#[test]
fn regression_requires_watchdog_evidence_capture_and_fast_lane() {
    // Regression: #383
    assert!(RUNBOOK.contains(
        "Capture incident evidence payload with expected/observed state hash, quorum sample, and censorship delivery ratios before rollback action."
    ));
    assert!(RUNBOOK.contains("## Fast and Cost-Effective Watchdog Validation Lane"));
    assert!(RUNBOOK.contains("cargo test -p kamn-core --test runtime_watchdog_attestation_docs"));
    assert!(RUNBOOK.contains("cargo test -p kamn-core --test upgrade_rollback_runbook_docs"));
}

#[test]
fn regression_requires_dr_evidence_and_slo_gate_guard() {
    // Regression: #623
    assert!(RUNBOOK.contains(
        "missing/incomplete DR evidence and SLO threshold violations force `NO-GO` (`Regression: #623`)."
    ));
}

#[test]
fn regression_requires_live_network_pilot_rollback_guard_marker() {
    // Regression: #830
    assert!(RUNBOOK.contains(
        "pilot rollback remains mandatory when deep-lane evidence is stale, missing, or policy-invalid (`Regression: #830`)."
    ));
}
