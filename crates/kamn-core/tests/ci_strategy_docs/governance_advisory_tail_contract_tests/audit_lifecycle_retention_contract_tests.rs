use super::super::{
    audit_integrity_reason_codes, lifecycle_ci_dry_run_reason_codes,
    AUDIT_INTEGRITY_REASON_CODES_CSV, AUDIT_INTEGRITY_REASON_TAXONOMY_VERSION, DOC,
    LIFECYCLE_CI_DRY_RUN_REASON_CODES_CSV, LIFECYCLE_CI_DRY_RUN_REASON_TAXONOMY_VERSION,
};
use super::support::assert_doc_contains_all;

#[test]
fn doc_contains_audit_integrity_dry_run_governance_markers() {
    assert_doc_contains_all(audit_integrity_core_markers(), "audit integrity governance");
    assert_audit_integrity_reason_markers();
}

#[test]
fn doc_contains_lifecycle_ci_dry_run_governance_markers() {
    assert_doc_contains_all(
        lifecycle_ci_dry_run_core_markers(),
        "lifecycle ci dry-run governance",
    );
    assert_lifecycle_ci_dry_run_reason_markers();
}

#[test]
fn doc_contains_retention_policy_checker_taxonomy_contract_markers() {
    assert_doc_contains_all(
        &[
            "### Retention Policy Checker Taxonomy Contract",
            "retention_policy_checker_reason_taxonomy_version=kamn.runtime.retention-policy-reason-taxonomy.v1",
            "retention_policy_checker_reason_codes_csv=retention_domain_unknown,retention_window_non_positive,retention_record_expired",
            "retention_policy_checker_fixture_schema_version=kamn.runtime.retention-policy-fixture-matrix.v1",
            "retention_policy_checker_fixture_path=fixtures/runtime/retention_policy_fixture_matrix.txt",
            "cargo test -p kamn-core --test retention_policy_checker_contract",
            "cargo test -p kamn-core --test retention_policy_fixture_parser_contract",
            "reason_codes=public_api_surface_fail_threshold_exceeded_unwaived",
            "reason_codes=waiver_cap_exceeded",
            "set `mitigation_issue=#<issue-id>` and a bounded `max_total_delta`",
        ],
        "retention policy taxonomy",
    );
}

fn audit_integrity_core_markers() -> &'static [&'static str] {
    &[
        "### Audit-Integrity Dry-Run Governance Contract",
        "bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh",
        "bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh",
        "bash scripts/deploy/generate_gonogo_evidence_bundle.sh --output-file /tmp/gonogo-audit-integrity.json",
        "bash scripts/deploy/check_gonogo_evidence_policy.sh --bundle-file /tmp/gonogo-audit-integrity.json",
        "audit_integrity_reason_codes_value=none|<csv>",
        "audit_integrity_gate_final_decision=GO|NO-GO",
        "audit integrity gate convergence mismatch",
        "cargo test -p kamn-core --test audit_evidence_integrity_contract spec_c01_audit_integrity_generate_bundle_emits_deterministic_go_markers -- --exact",
        "cargo test -p kamn-core --test ci_strategy_docs doc_contains_audit_integrity_dry_run_governance_markers -- --exact",
        "Regression: #4059",
    ]
}

fn assert_audit_integrity_reason_markers() {
    assert!(DOC.contains(&format!(
        "audit_integrity_reason_taxonomy_version={AUDIT_INTEGRITY_REASON_TAXONOMY_VERSION}"
    )));
    assert!(DOC.contains(&format!(
        "audit_integrity_reason_codes_csv={AUDIT_INTEGRITY_REASON_CODES_CSV}"
    )));
    for reason_code in audit_integrity_reason_codes() {
        assert!(
            DOC.contains(reason_code),
            "missing audit-integrity fail-closed reason marker {reason_code}"
        );
    }
}

fn lifecycle_ci_dry_run_core_markers() -> &'static [&'static str] {
    &[
        "### Lifecycle Artifact CI Dry-Run Governance Contract",
        "bash scripts/runtime/generate_lifecycle_artifact_integrity_evidence_bundle.sh --output-file /tmp/lifecycle-artifact-integrity-baseline.json --artifact-id lifecycle-artifact-baseline --lifecycle-stage retention --profile baseline --record-count 42 --ci-fast-gate PASS",
        "bash scripts/runtime/run_go_no_go_gate_lane.sh --mode dry-run --max-seconds 120 --output-json /tmp/go-no-go-gate-report.json",
        "python3 scripts/ci/check_lifecycle_ci_dry_run_governance.py --lifecycle-artifact-bundle-file /tmp/lifecycle-artifact-integrity-baseline.json --go-no-go-gate-report-file /tmp/go-no-go-gate-report.json --threshold-file fixtures/ci/lifecycle_ci_dry_run_governance_thresholds.env --strategy-doc docs/ci/strategy.md --ops-doc docs/ops/configuration.md --workflow-file .github/workflows/ci-fast-gate.yml --ci-tools-file scripts/ci/test_ci_tools.sh --output-json /tmp/lifecycle-ci-dry-run-governance-report.json",
        "cargo test -p kamn-core --test lifecycle_ci_dry_run_governance_contract -- --nocapture",
        "lifecycle_ci_dry_run_threshold_fixture_path=fixtures/ci/lifecycle_ci_dry_run_governance_thresholds.env",
        "lifecycle_ci_dry_run_max_seconds=120",
        "lifecycle_ci_dry_run_fast_mode_required_entry=cargo test -p kamn-core --test lifecycle_ci_dry_run_governance_contract -- --nocapture",
        "lifecycle_ci_dry_run_fast_mode_forbidden_entry=bash \"$ROOT_DIR/scripts/runtime/run_go_no_go_gate_lane.sh\" --mode run",
        "lifecycle_ci_dry_run_workflow_forbidden_entry=bash scripts/runtime/run_go_no_go_gate_lane.sh --mode run",
        "lifecycle_ci_dry_run_remediation_map_version=v1",
        "cargo test -p kamn-core --test quota_policy_checker_contract",
        "cargo test -p kamn-core --test quota_policy_fixture_parser_contract",
        "Regression: #4091",
    ]
}

fn assert_lifecycle_ci_dry_run_reason_markers() {
    assert!(DOC.contains(&format!(
        "lifecycle_ci_dry_run_reason_taxonomy_version={LIFECYCLE_CI_DRY_RUN_REASON_TAXONOMY_VERSION}"
    )));
    assert!(DOC.contains(&format!(
        "lifecycle_ci_dry_run_reason_codes_csv={LIFECYCLE_CI_DRY_RUN_REASON_CODES_CSV}"
    )));
    for reason_code in lifecycle_ci_dry_run_reason_codes() {
        assert!(
            DOC.contains(reason_code),
            "missing lifecycle ci dry-run reason marker {reason_code}"
        );
        assert!(
            DOC.contains(&format!("lifecycle_ci_dry_run_remediation.{reason_code}=")),
            "missing lifecycle ci dry-run remediation marker {reason_code}"
        );
    }
}
