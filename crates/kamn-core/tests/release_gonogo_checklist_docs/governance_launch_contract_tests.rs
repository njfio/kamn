use super::support::assert_checklist_contains_all;

const CHECKLIST_CONTAINS_GOVERNANCE_STAKE_SLASH_RISK_THRESHOLD_CONTRACT_MARKERS: &[&str] = &[
    "## Governance Stake/Slash Risk Threshold Contract",
    "generate_stake_slash_risk_evidence_bundle.sh",
    "check_stake_slash_risk_policy.sh",
    "stake_slash_risk_contract_lane_contract.py",
    "framework.contract_lane_helpers",
    "run_stake_slash_risk_contract_lane.sh",
    "run_stake_slash_risk_deep_lane.sh",
    "run_stake_slash_risk_matrix.py",
    "fixtures/governance_stake_slash/risk_threshold_cases.json",
];

#[test]
fn checklist_contains_governance_stake_slash_risk_threshold_contract() {
    assert_checklist_contains_all(CHECKLIST_CONTAINS_GOVERNANCE_STAKE_SLASH_RISK_THRESHOLD_CONTRACT_MARKERS, "checklist_contains_governance_stake_slash_risk_threshold_contract");
}

const CHECKLIST_CONTAINS_REPUTATION_DISPUTE_EVIDENCE_CONTRACT_MARKERS: &[&str] = &[
    "## Reputation Dispute Evidence Contract",
    "reputation_dispute_contract_lane_contract.py",
    "framework.contract_lane_helpers",
    "generate_reputation_dispute_evidence_bundle.sh",
    "check_reputation_dispute_policy.sh",
    "run_reputation_dispute_contract_lane.sh",
    "run_reputation_dispute_deep_lane.sh",
    "run_reputation_dispute_matrix.py",
    "fixtures/reputation_dispute/replay_cases.json",
];

#[test]
fn checklist_contains_reputation_dispute_evidence_contract() {
    assert_checklist_contains_all(CHECKLIST_CONTAINS_REPUTATION_DISPUTE_EVIDENCE_CONTRACT_MARKERS, "checklist_contains_reputation_dispute_evidence_contract");
}

const CHECKLIST_CONTAINS_TOKEN_LAUNCH_HANDOFF_EVIDENCE_CONTRACT_MARKERS: &[&str] = &[
    "## Token Launch Handoff Evidence Contract",
    "generate_token_launch_handoff_evidence_bundle.sh",
    "check_token_launch_handoff_policy.sh",
    "run_token_launch_handoff_contract_lane.sh",
    "run_token_launch_handoff_deep_lane.sh",
];

#[test]
fn checklist_contains_token_launch_handoff_evidence_contract() {
    assert_checklist_contains_all(CHECKLIST_CONTAINS_TOKEN_LAUNCH_HANDOFF_EVIDENCE_CONTRACT_MARKERS, "checklist_contains_token_launch_handoff_evidence_contract");
}

const CHECKLIST_CONTAINS_TREASURY_DISBURSEMENT_APPROVAL_EVIDENCE_CONTRACT_MARKERS: &[&str] = &[
    "## Treasury Disbursement Approval Evidence Contract",
    "generate_treasury_disbursement_evidence_bundle.sh",
    "check_treasury_disbursement_policy.sh",
    "treasury_disbursement_contract_lane_contract.py",
    "run_treasury_disbursement_contract_lane.sh",
];

#[test]
fn checklist_contains_treasury_disbursement_approval_evidence_contract() {
    assert_checklist_contains_all(CHECKLIST_CONTAINS_TREASURY_DISBURSEMENT_APPROVAL_EVIDENCE_CONTRACT_MARKERS, "checklist_contains_treasury_disbursement_approval_evidence_contract");
}

const CHECKLIST_CONTAINS_MAINNET_CUTOVER_MANIFEST_CONTRACT_MARKERS: &[&str] = &[
    "## Mainnet Cutover Manifest Validation Contract",
    "fixtures/mainnet_cutover/mainnet_cutover_manifest.schema.json",
    "validate_mainnet_cutover_manifest.py",
    "run_mainnet_cutover_contract_lane.sh",
];

#[test]
fn checklist_contains_mainnet_cutover_manifest_contract() {
    assert_checklist_contains_all(CHECKLIST_CONTAINS_MAINNET_CUTOVER_MANIFEST_CONTRACT_MARKERS, "checklist_contains_mainnet_cutover_manifest_contract");
}

const CHECKLIST_CONTAINS_CUTOVER_ROLLBACK_EVIDENCE_CONTRACT_MARKERS: &[&str] = &[
    "## Cutover Rollback Evidence Contract",
    "generate_cutover_rollback_evidence_bundle.sh",
    "check_cutover_rollback_evidence_policy.sh",
    "run_cutover_rollback_contract_lane.sh",
    "run_cutover_rollback_deep_lane.sh",
];

#[test]
fn checklist_contains_cutover_rollback_evidence_contract() {
    assert_checklist_contains_all(CHECKLIST_CONTAINS_CUTOVER_ROLLBACK_EVIDENCE_CONTRACT_MARKERS, "checklist_contains_cutover_rollback_evidence_contract");
}

const CHECKLIST_CONTAINS_LAUNCH_CANARY_CRITICAL_PATH_CONTRACT_MARKERS: &[&str] = &[
    "## Launch Canary Critical-Path Contract",
    "fixtures/launch_canary/critical_path_probe_cases.json",
    "run_launch_canary_matrix.py",
    "launch_canary_contract_lane_contract.py",
    "run_launch_canary_contract_lane.sh",
    "run_launch_canary_deep_lane.sh",
];

#[test]
fn checklist_contains_launch_canary_critical_path_contract() {
    assert_checklist_contains_all(CHECKLIST_CONTAINS_LAUNCH_CANARY_CRITICAL_PATH_CONTRACT_MARKERS, "checklist_contains_launch_canary_critical_path_contract");
}

const CHECKLIST_CONTAINS_POST_CUTOVER_SLO_EVIDENCE_CONTRACT_MARKERS: &[&str] = &[
    "## Post-Cutover SLO Gate Evidence Contract",
    "generate_post_cutover_slo_evidence_bundle.sh",
    "check_post_cutover_slo_policy.sh",
    "post_cutover_slo_contract_lane_contract.py",
    "run_post_cutover_slo_contract_lane.sh",
    "run_post_cutover_slo_deep_lane.sh",
    "alert_rule_promotion_gate_status=verified",
    "burn_rate_parity_status=verified",
    "ci_local_promotion_budget_boundary_status=verified",
    "KAMN_POST_CUTOVER_SLO_CI_LOCAL_PROMOTION_MAX_SECONDS",
    "KAMN_POST_CUTOVER_SLO_DEEP_LOCAL_ONLY",
];

#[test]
fn checklist_contains_post_cutover_slo_evidence_contract() {
    assert_checklist_contains_all(CHECKLIST_CONTAINS_POST_CUTOVER_SLO_EVIDENCE_CONTRACT_MARKERS, "checklist_contains_post_cutover_slo_evidence_contract");
}

const REGRESSION_REQUIRES_ROLLBACK_PRECHECK_IN_CHECKLIST_MARKERS: &[&str] = &[
    "Rollback precheck result: PASS",
];

#[test]
fn regression_requires_rollback_precheck_in_checklist() {
    assert_checklist_contains_all(REGRESSION_REQUIRES_ROLLBACK_PRECHECK_IN_CHECKLIST_MARKERS, "regression_requires_rollback_precheck_in_checklist");
}
