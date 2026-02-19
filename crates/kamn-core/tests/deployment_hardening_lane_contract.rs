const CI_STRATEGY_DOC: &str = include_str!("../../../docs/ci/strategy.md");
const PRODUCTION_NEXT_STEPS_DOC: &str =
    include_str!("../../../docs/plans/2026-02-14-production-service-next-steps.md");
const DEPLOYMENT_PREFLIGHT_LANE_IMPL: &str =
    include_str!("../../../scripts/kolme/run_local_kolme_live_deployment_preflight_lane_impl.sh");

const REQUIRED_LANE_ARTIFACT_MARKERS: &[&str] = &[
    "signer_provenance_present",
    "signer_provenance_sha256_valid",
    "deployment_preflight_passed",
    "dry_run_no_commands_executed",
    "preflight_budget_exceeded",
    "signer_rotation_fresh",
    "signer_rotation_delta_epochs",
];

const REQUIRED_NEXT_STEPS_MARKERS: &[&str] = &[
    "deployment_hardening_local_heavy_contract_chain=#3950->#3954->(#3961,#3962)",
    "deployment_hardening_local_heavy_contract_guard_command=cargo test -p kamn-core --test deployment_hardening_lane_contract -- --nocapture",
    "deployment_hardening_local_heavy_required_reason_codes_csv=dry_run_no_commands_executed,deployment_preflight_passed,preflight_budget_exceeded,signer_rotation_rehearsal_drift_detected,signer_rotation_promotion_stalled",
];

#[test]
fn unit_deployment_preflight_lane_impl_declares_required_artifact_markers() {
    for marker in REQUIRED_LANE_ARTIFACT_MARKERS {
        assert!(
            DEPLOYMENT_PREFLIGHT_LANE_IMPL.contains(marker),
            "deployment preflight lane implementation missing required marker: {marker}"
        );
    }
}

#[test]
fn functional_ci_strategy_declares_local_heavy_boundary_for_deployment_preflight_lane() {
    assert!(CI_STRATEGY_DOC.contains(
        "run_local_kolme_live_deployment_preflight_lane.sh --mode dry-run --output-json /tmp/kolme-local-live-deployment-preflight-summary.json"
    ));
    assert!(CI_STRATEGY_DOC.contains(
        "run_local_kolme_live_deployment_preflight_lane.sh --mode run --runtime-mode kolme-live"
    ));
    assert!(CI_STRATEGY_DOC.contains(
        "deployment preflight signer/runtime checks remain fast and ci-fast-gate eligible."
    ));
    assert!(CI_STRATEGY_DOC.contains(
        "deployment preflight contract lane parity remains fail-closed (`Regression: #2226`)."
    ));
}

#[test]
fn integration_production_next_steps_declares_deployment_hardening_contract_markers() {
    for marker in REQUIRED_NEXT_STEPS_MARKERS {
        assert!(
            PRODUCTION_NEXT_STEPS_DOC.contains(marker),
            "production next-steps docs missing deployment hardening marker: {marker}"
        );
    }
}

#[test]
fn regression_production_next_steps_links_existing_deployment_preflight_guard_commands() {
    assert!(PRODUCTION_NEXT_STEPS_DOC.contains(
        "bash scripts/kolme/run_local_kolme_live_deployment_preflight_lane.sh --mode dry-run --output-json /tmp/kolme-local-live-deployment-preflight-summary.json"
    ));
    assert!(PRODUCTION_NEXT_STEPS_DOC.contains(
        "python3 scripts/kolme/check_local_kolme_live_deployment_preflight_policy.py --report-file /tmp/kolme-local-live-deployment-preflight-summary.json --expected-final-decision GO --ci-fast-gate PASS --require-reason-code dry_run_no_commands_executed --output-json /tmp/kolme-local-live-deployment-preflight-policy.json"
    ));
    assert!(PRODUCTION_NEXT_STEPS_DOC.contains(
        "bash scripts/kolme/run_local_kolme_live_deployment_preflight_contract_lane.sh --output-json /tmp/kolme-local-live-deployment-preflight-summary.json --policy-output-json /tmp/kolme-local-live-deployment-preflight-policy.json"
    ));
}
