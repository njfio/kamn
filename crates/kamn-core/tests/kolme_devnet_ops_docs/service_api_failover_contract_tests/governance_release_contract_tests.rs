use super::super::docs_assert_support::{assert_plan_contains_all};

const PLAN_CONTAINS_MILESTONE_UPGRADE_LINEAGE_POLICY_MARKERS_PLAN_MARKERS: &[&str] = &[
    "## Milestone Review Aggregate Evidence Bundle (Issue #3247)",
    "python3 scripts/deploy/check_upgrade_rehearsal_lineage_policy.py --bundle-file /tmp/gonogo-milestone.json --expected-final-decision GO",
    "upgrade_lineage_reason_taxonomy_version=kamn.release.gonogo-live-evidence-convergence-reason-taxonomy.v1",
    "upgrade_lineage_reason_codes_csv=none|<csv>",
    "upgrade_lineage_reason_codes_value=none|<csv>",
    "promotion_gate_reason_taxonomy_version=kamn.release.gonogo-live-evidence-convergence-reason-taxonomy.v1",
    "promotion_gate_reason_codes_csv=none|<csv>",
    "promotion_gate_reason_codes_value=none|<csv>",
    "promotion gate reason mapping mismatch",
    "milestone review bundle lineage mismatch",
];

#[test]
fn plan_contains_milestone_upgrade_lineage_policy_markers() {
    assert_plan_contains_all(PLAN_CONTAINS_MILESTONE_UPGRADE_LINEAGE_POLICY_MARKERS_PLAN_MARKERS, "plan_contains_milestone_upgrade_lineage_policy_markers");
}

const PLAN_CONTAINS_RELEASE_PROMOTION_EVIDENCE_CONVERGENCE_INTEGRITY_MARKERS_PLAN_MARKERS: &[&str] = &[
    "Convergence integrity markers:",
    "`required_artifact_ids` must include `local_full_runtime_convergence`.",
    "release_manifest_missing_required_artifact:local_full_runtime_convergence",
    "release_manifest_success_marker_mismatch:local_full_runtime_convergence",
    "Regression: #4199",
];

#[test]
fn plan_contains_release_promotion_evidence_convergence_integrity_markers() {
    assert_plan_contains_all(PLAN_CONTAINS_RELEASE_PROMOTION_EVIDENCE_CONVERGENCE_INTEGRITY_MARKERS_PLAN_MARKERS, "plan_contains_release_promotion_evidence_convergence_integrity_markers");
}

const PLAN_CONTAINS_R27_21_UPGRADE_COMPATIBILITY_CI_SMOKE_GOVERNANCE_CLOSURE_MARKERS_PLAN_MARKERS: &[&str] = &[
    "## R27.21 Upgrade Compatibility CI Smoke Governance Closure (Issue #4187)",
    "check_upgrade_compatibility_ci_smoke_convergence.py --workflow-file .github/workflows/ci-fast-gate.yml --ci-tools-file scripts/ci/test_ci_tools.sh --strategy-doc docs/ci/strategy.md --plan-doc docs/plans/2026-02-14-production-service-next-steps.md --max-seconds 120 --output-json /tmp/upgrade-compatibility-ci-smoke-convergence-report.json",
    "test_check_upgrade_compatibility_ci_smoke_convergence.sh",
    "upgrade_compatibility_ci_smoke_convergence_status=verified",
    "upgrade_compatibility_ci_smoke_reason_taxonomy_version=kamn.ci.upgrade-compatibility-ci-smoke-convergence-reason-taxonomy.v1",
    "upgrade_compatibility_ci_smoke_reason_codes_csv=upgrade_compatibility_fork_evidence_ci_smoke_composition_missing,upgrade_compatibility_fork_policy_ci_smoke_composition_missing,upgrade_compatibility_replay_command_leaked_in_fast_mode,ci_fast_gate_upgrade_compatibility_replay_command_not_excluded,ci_strategy_upgrade_compatibility_convergence_markers_missing,production_plan_upgrade_compatibility_convergence_markers_missing,upgrade_compatibility_ci_smoke_seconds_exceeded",
    "upgrade_compatibility_ci_smoke_max_seconds=120",
    "upgrade_compatibility_local_heavy_max_seconds=900",
];

#[test]
fn plan_contains_r27_21_upgrade_compatibility_ci_smoke_governance_closure_markers() {
    assert_plan_contains_all(PLAN_CONTAINS_R27_21_UPGRADE_COMPATIBILITY_CI_SMOKE_GOVERNANCE_CLOSURE_MARKERS_PLAN_MARKERS, "plan_contains_r27_21_upgrade_compatibility_ci_smoke_governance_closure_markers");
}

const PLAN_CONTAINS_R27_22_FULL_STACK_CI_SMOKE_GOVERNANCE_CLOSURE_MARKERS_PLAN_MARKERS: &[&str] = &[
    "## R27.22 Full-Stack CI Smoke Governance Closure (Issue #4202)",
    "check_local_full_stack_integration_ci_smoke_convergence.py --workflow-file .github/workflows/ci-fast-gate.yml --ci-tools-file scripts/ci/test_ci_tools.sh --strategy-doc docs/ci/strategy.md --plan-doc docs/plans/2026-02-14-production-service-next-steps.md --max-seconds 120 --output-json /tmp/local-full-stack-ci-smoke-convergence-report.json",
    "test_check_local_full_stack_integration_ci_smoke_convergence.sh",
    "local_full_stack_ci_smoke_convergence_status=verified",
    "local_full_stack_ci_smoke_reason_taxonomy_version=kamn.ci.local-full-stack-integration-ci-smoke-convergence-reason-taxonomy.v1",
    "local_full_stack_ci_smoke_reason_codes_csv=local_full_stack_exclusion_policy_ci_smoke_composition_missing,local_full_stack_validate_ci_smoke_composition_missing,local_full_stack_policy_ci_smoke_composition_missing,local_full_stack_contract_lane_ci_smoke_composition_missing,local_full_stack_run_mode_command_leaked_in_fast_mode,ci_fast_gate_local_full_stack_run_mode_not_excluded,ci_strategy_local_full_stack_convergence_markers_missing,production_plan_local_full_stack_convergence_markers_missing,local_full_stack_ci_smoke_seconds_exceeded",
    "local_full_stack_ci_smoke_max_seconds=120",
    "local_full_stack_local_heavy_max_seconds=900",
];

#[test]
fn plan_contains_r27_22_full_stack_ci_smoke_governance_closure_markers() {
    assert_plan_contains_all(PLAN_CONTAINS_R27_22_FULL_STACK_CI_SMOKE_GOVERNANCE_CLOSURE_MARKERS_PLAN_MARKERS, "plan_contains_r27_22_full_stack_ci_smoke_governance_closure_markers");
}
