const CI_STRATEGY_DOC: &str = include_str!("../../../docs/ci/strategy.md");
const PRODUCTION_NEXT_STEPS_DOC: &str =
    include_str!("../../../docs/plans/2026-02-14-production-service-next-steps.md");
const DEPLOYMENT_PREFLIGHT_POLICY_CHECKER: &str =
    include_str!("../../../scripts/kolme/check_local_kolme_live_deployment_preflight_policy.py");

const RUNBOOK_REASON_CODES_CSV: &str =
    "deployment_preflight_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch";

const REQUIRED_POLICY_CHECKER_MARKERS: &[&str] = &[
    "DEPLOYMENT_PREFLIGHT_RUNBOOK_REASON_TAXONOMY_VERSION",
    "kamn.kolme.local-live-deployment-preflight-runbook-reason-taxonomy.v1",
    "DEPLOYMENT_PREFLIGHT_RUNBOOK_REASON_CODES = (",
    "deployment_preflight_taxonomy_mapping_drift_detected",
    "runbook_marker_parity_mismatch",
    "parser.add_argument(\"--ci-fast-gate\", required=True, choices=[\"PASS\", \"FAIL\"])",
    "parser.add_argument(\"--require-reason-code\", action=\"append\", default=[])",
    "deployment_preflight_runbook_marker_parity_status",
    "deployment_preflight_runbook_reason_taxonomy_version",
    "deployment_preflight_runbook_reason_codes_csv",
];

const REQUIRED_CI_STRATEGY_MARKERS: &[&str] = &[
    "deployment preflight signer/runtime checks remain fast and ci-fast-gate eligible.",
    "run_local_kolme_live_deployment_preflight_lane.sh --mode dry-run --output-json /tmp/kolme-local-live-deployment-preflight-summary.json",
    "python3 scripts/kolme/check_local_kolme_live_deployment_preflight_policy.py --report-file /tmp/kolme-local-live-deployment-preflight-summary.json --expected-final-decision GO --ci-fast-gate PASS --require-reason-code dry_run_no_commands_executed --output-json /tmp/kolme-local-live-deployment-preflight-policy.json",
    "run_local_kolme_live_deployment_preflight_contract_lane.sh --output-json /tmp/kolme-local-live-deployment-preflight-summary.json --policy-output-json /tmp/kolme-local-live-deployment-preflight-policy.json",
];

const REQUIRED_NEXT_STEPS_MARKERS: &[&str] = &[
    "deployment_hardening_ci_dry_run_contract_chain=#3950->#3954->#3962",
    "deployment_hardening_ci_dry_run_contract_guard_command=cargo test -p kamn-core --test deployment_hardening_ci_dry_run_contract -- --nocapture",
    "deployment_hardening_ci_dry_run_governance_checker=python3 scripts/kolme/check_local_kolme_live_deployment_preflight_policy.py --report-file /tmp/kolme-local-live-deployment-preflight-summary.json --expected-final-decision GO --ci-fast-gate PASS --require-reason-code dry_run_no_commands_executed --output-json /tmp/kolme-local-live-deployment-preflight-policy.json",
    "deployment_hardening_ci_dry_run_runbook_reason_codes_csv=deployment_preflight_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch",
];

#[test]
fn unit_policy_checker_declares_ci_dry_run_and_runbook_parity_markers() {
    for marker in REQUIRED_POLICY_CHECKER_MARKERS {
        assert!(
            DEPLOYMENT_PREFLIGHT_POLICY_CHECKER.contains(marker),
            "deployment preflight policy checker missing required marker: {marker}"
        );
    }
}

#[test]
fn functional_ci_strategy_declares_ci_dry_run_governance_markers() {
    for marker in REQUIRED_CI_STRATEGY_MARKERS {
        assert!(
            CI_STRATEGY_DOC.contains(marker),
            "ci strategy doc missing required deployment hardening ci dry-run marker: {marker}"
        );
    }
}

#[test]
fn integration_next_steps_declares_3962_closure_chain_markers() {
    for marker in REQUIRED_NEXT_STEPS_MARKERS {
        assert!(
            PRODUCTION_NEXT_STEPS_DOC.contains(marker),
            "production next-steps doc missing required #3962 closure marker: {marker}"
        );
    }
}

#[test]
fn regression_ci_strategy_and_next_steps_keep_runbook_parity_fail_closed_markers() {
    assert!(
        CI_STRATEGY_DOC.contains("runbook_marker_parity_mismatch"),
        "ci strategy doc missing runbook marker parity fail-closed marker"
    );
    assert!(
        PRODUCTION_NEXT_STEPS_DOC.contains(RUNBOOK_REASON_CODES_CSV),
        "production next-steps doc missing deployment preflight runbook reason csv marker"
    );
}
