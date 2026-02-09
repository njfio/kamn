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
fn checklist_contains_settlement_reconciliation_evidence_contract() {
    assert!(CHECKLIST.contains("## Settlement Reconciliation Evidence Contract"));
    assert!(CHECKLIST.contains("generate_settlement_reconciliation_evidence_bundle.sh",));
    assert!(CHECKLIST.contains("check_settlement_reconciliation_evidence_policy.sh",));
    assert!(CHECKLIST.contains("run_settlement_reconciliation_contract_lane.sh"));
    assert!(CHECKLIST.contains("run_settlement_reconciliation_deep_lane.sh"));
    assert!(CHECKLIST.contains("run_settlement_reconciliation_race_matrix.py"));
    assert!(CHECKLIST.contains("fixtures/escrow_reconciliation/finality_race_cases.json"));
    assert!(CHECKLIST.contains("--ledger-reference-id"));
}

#[test]
fn checklist_contains_soc2_control_evidence_contract() {
    assert!(CHECKLIST.contains("## SOC2 Control Evidence Contract"));
    assert!(CHECKLIST.contains("generate_soc2_control_evidence_bundle.sh"));
    assert!(CHECKLIST.contains("check_soc2_control_evidence_policy.sh"));
    assert!(CHECKLIST.contains("run_soc2_control_evidence_contract_lane.sh"));
    assert!(CHECKLIST.contains("run_soc2_control_evidence_deep_lane.sh"));
    assert!(CHECKLIST.contains("run_soc2_control_evidence_replay_matrix.py"));
    assert!(CHECKLIST.contains("fixtures/compliance_soc2/control_evidence_replay_cases.json"));
}

#[test]
fn checklist_contains_dsar_legal_hold_evidence_contract() {
    assert!(CHECKLIST.contains("## DSAR Legal-Hold Evidence Contract"));
    assert!(CHECKLIST.contains("generate_dsar_legal_hold_evidence_bundle.sh"));
    assert!(CHECKLIST.contains("check_dsar_legal_hold_policy.sh"));
    assert!(CHECKLIST.contains("run_dsar_legal_hold_contract_lane.sh"));
    assert!(CHECKLIST.contains("run_dsar_legal_hold_deep_lane.sh"));
    assert!(CHECKLIST.contains("run_dsar_legal_hold_matrix.py"));
    assert!(CHECKLIST.contains("fixtures/compliance_dsar/legal_hold_precedence_cases.json"));
}

#[test]
fn checklist_contains_governance_simulation_and_human_veto_evidence_contract() {
    assert!(CHECKLIST.contains("## Governance Simulation and Human-Veto Evidence Contract"));
    assert!(CHECKLIST.contains("generate_governance_simulation_evidence_bundle.sh"));
    assert!(CHECKLIST.contains("check_governance_simulation_policy.sh"));
    assert!(CHECKLIST.contains("run_governance_simulation_contract_lane.sh"));
    assert!(CHECKLIST.contains("run_governance_simulation_deep_lane.sh"));
    assert!(CHECKLIST.contains("run_governance_simulation_matrix.py"));
    assert!(CHECKLIST.contains("fixtures/governance_simulation/veto_timelock_cases.json"));
}

#[test]
fn checklist_contains_governance_stake_slash_risk_threshold_contract() {
    assert!(CHECKLIST.contains("## Governance Stake/Slash Risk Threshold Contract"));
    assert!(CHECKLIST.contains("generate_stake_slash_risk_evidence_bundle.sh"));
    assert!(CHECKLIST.contains("check_stake_slash_risk_policy.sh"));
    assert!(CHECKLIST.contains("run_stake_slash_risk_contract_lane.sh"));
    assert!(CHECKLIST.contains("run_stake_slash_risk_deep_lane.sh"));
    assert!(CHECKLIST.contains("run_stake_slash_risk_matrix.py"));
    assert!(CHECKLIST.contains("fixtures/governance_stake_slash/risk_threshold_cases.json"));
}

#[test]
fn checklist_contains_token_launch_handoff_evidence_contract() {
    assert!(CHECKLIST.contains("## Token Launch Handoff Evidence Contract"));
    assert!(CHECKLIST.contains("generate_token_launch_handoff_evidence_bundle.sh"));
    assert!(CHECKLIST.contains("check_token_launch_handoff_policy.sh"));
    assert!(CHECKLIST.contains("run_token_launch_handoff_contract_lane.sh"));
    assert!(CHECKLIST.contains("run_token_launch_handoff_deep_lane.sh"));
}

#[test]
fn checklist_contains_treasury_disbursement_approval_evidence_contract() {
    assert!(CHECKLIST.contains("## Treasury Disbursement Approval Evidence Contract"));
    assert!(CHECKLIST.contains("generate_treasury_disbursement_evidence_bundle.sh"));
    assert!(CHECKLIST.contains("check_treasury_disbursement_policy.sh"));
    assert!(CHECKLIST.contains("run_treasury_disbursement_contract_lane.sh"));
}

#[test]
fn checklist_contains_mainnet_cutover_manifest_contract() {
    assert!(CHECKLIST.contains("## Mainnet Cutover Manifest Validation Contract"));
    assert!(CHECKLIST.contains("fixtures/mainnet_cutover/mainnet_cutover_manifest.schema.json"));
    assert!(CHECKLIST.contains("validate_mainnet_cutover_manifest.py"));
    assert!(CHECKLIST.contains("run_mainnet_cutover_contract_lane.sh"));
}

#[test]
fn checklist_contains_cutover_rollback_evidence_contract() {
    assert!(CHECKLIST.contains("## Cutover Rollback Evidence Contract"));
    assert!(CHECKLIST.contains("generate_cutover_rollback_evidence_bundle.sh"));
    assert!(CHECKLIST.contains("check_cutover_rollback_evidence_policy.sh"));
    assert!(CHECKLIST.contains("run_cutover_rollback_contract_lane.sh"));
    assert!(CHECKLIST.contains("run_cutover_rollback_deep_lane.sh"));
}

#[test]
fn checklist_contains_launch_canary_critical_path_contract() {
    assert!(CHECKLIST.contains("## Launch Canary Critical-Path Contract"));
    assert!(CHECKLIST.contains("fixtures/launch_canary/critical_path_probe_cases.json"));
    assert!(CHECKLIST.contains("run_launch_canary_matrix.py"));
    assert!(CHECKLIST.contains("run_launch_canary_contract_lane.sh"));
    assert!(CHECKLIST.contains("run_launch_canary_deep_lane.sh"));
}

#[test]
fn checklist_contains_post_cutover_slo_evidence_contract() {
    assert!(CHECKLIST.contains("## Post-Cutover SLO Gate Evidence Contract"));
    assert!(CHECKLIST.contains("generate_post_cutover_slo_evidence_bundle.sh"));
    assert!(CHECKLIST.contains("check_post_cutover_slo_policy.sh"));
    assert!(CHECKLIST.contains("run_post_cutover_slo_contract_lane.sh"));
    assert!(CHECKLIST.contains("run_post_cutover_slo_deep_lane.sh"));
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

#[test]
fn regression_requires_chain_receipt_evidence_guard_marker() {
    // Regression: #678
    assert!(CHECKLIST.contains(
        "missing or invalid chain receipt evidence forces `NO-GO` (`Regression: #678`)."
    ));
    assert!(CHECKLIST.contains(
        "timeout-before-finality pending receipts and failed receipts force `NO-GO` (`Regression: #678`)."
    ));
}

#[test]
fn regression_requires_ledger_reference_evidence_guard_marker() {
    // Regression: #717
    assert!(CHECKLIST.contains(
        "missing ledger reference evidence and ledger amount drift force `NO-GO` (`Regression: #717`)."
    ));
}

#[test]
fn regression_requires_soc2_control_evidence_guard_marker() {
    // Regression: #732
    assert!(CHECKLIST.contains(
        "tampered final decisions and incomplete/tampered control evidence force `NO-GO` (`Regression: #732`)."
    ));
}

#[test]
fn regression_requires_dsar_legal_hold_evidence_guard_marker() {
    // Regression: #732
    assert!(CHECKLIST.contains(
        "legal-hold bypass attempts and tampered DSAR evidence force `NO-GO` (`Regression: #732`)."
    ));
}

#[test]
fn regression_requires_governance_simulation_and_veto_guard_marker() {
    // Regression: #733
    assert!(CHECKLIST.contains(
        "simulation/veto bypass attempts and tampered evidence bundles force `NO-GO` (`Regression: #733`)."
    ));
}

#[test]
fn regression_requires_governance_stake_slash_risk_guard_marker() {
    // Regression: #733
    assert!(CHECKLIST.contains(
        "unsafe threshold bypass attempts and tampered risk evidence force `NO-GO` (`Regression: #733`)."
    ));
}

#[test]
fn regression_requires_token_launch_handoff_evidence_guard_marker() {
    // Regression: #714
    assert!(CHECKLIST.contains(
        "supply/allocation invariant drift and insufficient approvals force `NO-GO` (`Regression: #714`)."
    ));
}

#[test]
fn regression_requires_treasury_disbursement_approval_guard_marker() {
    // Regression: #716
    assert!(CHECKLIST.contains(
        "insufficient approvals, approval-window closure, and daily-limit overruns force `NO-GO` (`Regression: #716`)."
    ));
}

#[test]
fn regression_requires_mainnet_cutover_dependency_and_approval_guards() {
    // Regression: #705
    assert!(CHECKLIST.contains(
        "unresolved/non-prior dependencies and insufficient approvals force `NO-GO` (`Regression: #705`)."
    ));
}

#[test]
fn regression_requires_cutover_rollback_evidence_guard_marker() {
    // Regression: #708
    assert!(CHECKLIST.contains(
        "missing failed-checkpoint evidence and rollback-target hash mismatch force `NO-GO` (`Regression: #708`)."
    ));
}

#[test]
fn regression_requires_launch_canary_evidence_guard_marker() {
    // Regression: #710
    assert!(CHECKLIST.contains(
        "missing probe evidence and failing critical-path probes force `NO-GO` (`Regression: #710`)."
    ));
}

#[test]
fn regression_requires_post_cutover_slo_evidence_guard_marker() {
    // Regression: #711
    assert!(CHECKLIST.contains(
        "stale snapshots and incomplete SLO evidence force `NO-GO` (`Regression: #711`)."
    ));
}
