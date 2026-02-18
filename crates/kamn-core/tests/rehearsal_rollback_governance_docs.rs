const PLAN: &str = include_str!("../../../docs/plans/2026-02-14-production-service-next-steps.md");
const CI_STRATEGY: &str = include_str!("../../../docs/ci/strategy.md");
const INCIDENT_READINESS: &str = include_str!("../../../docs/ops/incident-readiness.md");

#[test]
fn plan_contains_r27_19_rehearsal_rollback_ci_smoke_closure_markers() {
    assert!(PLAN.contains("### R27.19 Rehearsal/Rollback CI Smoke Governance Closure"));
    assert!(PLAN.contains("Active chain: `#4145 -> #4149 -> (#4156, #4157)`."));
    assert!(PLAN.contains("rehearsal_promotion_ci_smoke_convergence_status=verified"));
    assert!(PLAN.contains(
        "rehearsal_promotion_ci_smoke_reason_taxonomy_version=kamn.ci.rehearsal-promotion-ci-smoke-convergence-reason-taxonomy.v1"
    ));
    assert!(PLAN.contains("rehearsal_promotion_ci_smoke_max_seconds=120"));
    assert!(PLAN.contains("rehearsal_promotion_local_heavy_max_seconds=900"));
    assert!(PLAN.contains(
        "python3 scripts/deploy/check_upgrade_rehearsal_lineage_policy.py --bundle-file /tmp/gonogo-milestone.json --expected-final-decision GO"
    ));
    assert!(PLAN.contains("bash scripts/deploy/test_run_staging_rehearsal_contract_lane.sh"));
}

#[test]
fn ci_and_ops_docs_keep_rehearsal_boundary_markers_in_sync() {
    let required_markers = [
        "rehearsal_boundary_reason_codes_csv=rehearsal_boundary_ci_smoke_seconds_exceeded,rehearsal_boundary_local_heavy_opt_in_missing,rehearsal_runbook_contract_parity_mismatch",
        "rehearsal_boundary_ci_smoke_max_seconds=120",
        "rehearsal_boundary_local_heavy_max_seconds=900",
    ];

    for marker in required_markers {
        assert!(
            CI_STRATEGY.contains(marker),
            "docs/ci/strategy.md missing marker: {marker}"
        );
        assert!(
            INCIDENT_READINESS.contains(marker),
            "docs/ops/incident-readiness.md missing marker: {marker}"
        );
    }
}
