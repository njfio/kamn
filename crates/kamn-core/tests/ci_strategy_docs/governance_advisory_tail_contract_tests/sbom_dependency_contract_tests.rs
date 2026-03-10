use super::super::DOC;

#[test]
fn doc_contains_sbom_provenance_artifact_generator_contract_markers() {
    assert!(DOC.contains("## SBOM-Provenance Artifact Generator Contract (Issue #4036)"));
    assert!(DOC.contains(
        "cargo run -p kamn-core --bin sbom_provenance_artifact_generator_contract -- --profile baseline --mode dry-run --ci-fast-gate PASS --max-seconds 120 --output-json /tmp/sbom-provenance-baseline.json"
    ));
    assert!(DOC.contains(
        "cargo run -p kamn-core --bin sbom_provenance_artifact_generator_contract -- --profile injected-drift --mode dry-run --ci-fast-gate PASS --max-seconds 120 --output-json /tmp/sbom-provenance-injected-drift.json"
    ));
    assert!(DOC.contains("kamn.runtime.sbom-provenance-artifact-report.v1"));
    assert!(DOC.contains("kamn.runtime.sbom-provenance-artifact-schema.v1"));
    assert!(DOC.contains("kamn.ci.sbom-provenance-artifact-fixture-matrix.v1"));
    assert!(DOC.contains("kamn.runtime.sbom-provenance-artifact-reason-taxonomy.v1"));
    assert!(DOC.contains(
        "sbom_provenance_reason_codes_csv=sbom_provenance_profile_contract_violation,sbom_provenance_runtime_budget_exceeded"
    ));
    assert!(DOC.contains("sbom_schema_version=spdx-2.3"));
    assert!(DOC.contains("provenance_schema_version=slsa-v1"));
    assert!(DOC.contains("release_manifest_required_artifact_id=sbom_provenance"));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test sbom_provenance_artifact_generator_contract -- --nocapture"
    ));
    assert!(DOC.contains("Regression: #4036"));
}

#[test]
fn doc_contains_sbom_provenance_release_gonogo_checker_contract_markers() {
    assert!(DOC.contains("## SBOM-Provenance Release Go-No-Go Checker Contract (Issue #4037)"));
    assert!(DOC.contains(
        "python3 scripts/deploy/sbom_provenance_release_gonogo_checker_contract.py --artifact-json /tmp/sbom-provenance-baseline.json --ci-strategy-doc docs/ci/strategy.md --ops-configuration-doc docs/ops/configuration.md --max-seconds 120 --output-json /tmp/sbom-provenance-release-gonogo-checker.json"
    ));
    assert!(DOC.contains(
        "sbom_provenance_release_gonogo_checker_schema_version=kamn.runtime.sbom-provenance-release-gonogo-checker-report.v1"
    ));
    assert!(DOC.contains(
        "sbom_provenance_release_gonogo_checker_reason_taxonomy_version=kamn.runtime.sbom-provenance-release-gonogo-checker-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "sbom_provenance_release_gonogo_checker_reason_codes_csv=sbom_provenance_artifact_marker_missing,sbom_provenance_artifact_marker_invalid,sbom_provenance_artifact_decision_not_go,sbom_provenance_docs_parity_marker_missing,sbom_provenance_runtime_budget_exceeded"
    ));
    assert!(DOC.contains(
        "sbom_provenance_release_gonogo_required_artifact_markers_csv=schema_version,artifact_schema_version,fixture_schema_version,reason_taxonomy_version,release_manifest_required_artifact_id,status,final_decision,reason_code"
    ));
    assert!(DOC.contains(
        "sbom_provenance_release_gonogo_docs_parity_required_markers_csv=sbom_provenance_release_gonogo_checker_schema_version,sbom_provenance_release_gonogo_checker_reason_taxonomy_version,sbom_provenance_release_gonogo_checker_reason_codes_csv,sbom_provenance_release_gonogo_required_artifact_markers_csv"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test sbom_provenance_release_gonogo_checker_contract -- --nocapture"
    ));
    assert!(DOC.contains("Regression: #4037"));
}

#[test]
fn doc_contains_dependency_ci_smoke_advisory_fixture_contract_markers() {
    assert!(DOC.contains("## Dependency CI Smoke Advisory Fixture Contract"));
    assert!(DOC.contains(
        "dependency_ci_smoke_reason_taxonomy_version=kamn.ci.dependency-ci-smoke-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "dependency_ci_smoke_reason_codes_csv=dependency_advisory_severity_unknown,dependency_advisory_threshold_exceeded"
    ));
    assert!(DOC.contains(
        "dependency_ci_smoke_fixture_schema_version=kamn.ci.dependency-ci-smoke-advisory-fixture-matrix.v1"
    ));
    assert!(DOC.contains(
        "dependency_ci_smoke_fixture_path=fixtures/ci/dependency_ci_smoke_advisory_fixture_matrix.txt"
    ));
    assert!(DOC.contains("dependency_ci_smoke_threshold_max_severity=moderate"));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test dependency_ci_smoke_advisory_fixture_parser_contract"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test ci_strategy_docs doc_contains_dependency_ci_smoke_advisory_fixture_contract_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #4030"));
}

#[test]
fn doc_contains_dependency_ci_smoke_checker_threshold_parity_markers() {
    assert!(DOC.contains(
        "dependency_ci_smoke_checker_reason_taxonomy_version=kamn.ci.dependency-ci-smoke-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "dependency_ci_smoke_checker_reason_codes_csv=dependency_advisory_input_empty,dependency_advisory_severity_unknown,dependency_advisory_threshold_exceeded"
    ));
    assert!(DOC.contains(
        "dependency_ci_smoke_checker_fixture_schema_version=kamn.ci.dependency-ci-smoke-advisory-fixture-matrix.v1"
    ));
    assert!(DOC.contains(
        "dependency_ci_smoke_checker_fixture_path=fixtures/ci/dependency_ci_smoke_advisory_fixture_matrix.txt"
    ));
    assert!(DOC.contains("dependency_ci_smoke_checker_threshold_max_severity=moderate"));
    assert!(DOC.contains(
        "dependency_ci_smoke_checker_remediation.dependency_advisory_input_empty=provide at least one advisory record from the CI smoke advisory feed before evaluating thresholds"
    ));
    assert!(DOC.contains(
        "dependency_ci_smoke_checker_remediation.dependency_advisory_severity_unknown=normalize advisory severity to low|moderate|high|critical before evaluation"
    ));
    assert!(DOC.contains(
        "dependency_ci_smoke_checker_remediation.dependency_advisory_threshold_exceeded=reduce dependency advisory severity exposure or update approved threshold with review evidence"
    ));
    assert!(DOC.contains("cargo test -p kamn-core --test dependency_ci_smoke_checker_contract"));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test ci_strategy_docs doc_contains_dependency_ci_smoke_checker_threshold_parity_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #4031"));
}

#[test]
fn doc_contains_cargo_audit_runner_impact_measurement_markers() {
    assert!(
        DOC.contains("cargo_audit_runner_impact_method=github-actions-step-duration+policy-output")
    );
    assert!(DOC.contains("cargo_audit_fast_gate_observed_seconds=156"));
    assert!(DOC.contains("cargo_audit_workspace_premerge_observed_seconds=156"));
    assert!(DOC.contains("cargo_audit_runner_impact_baseline_captured_at=2026-03-05"));
    assert!(DOC.contains("cargo_audit_runner_impact_source_fast_gate_run=22707945091"));
    assert!(DOC.contains("cargo_audit_runner_impact_source_job=65838851460"));
    assert!(DOC.contains("cargo_audit_policy_elapsed_seconds=<float>"));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test ci_strategy_docs doc_contains_cargo_audit_runner_impact_measurement_markers -- --exact"
    ));
}

#[test]
fn doc_contains_anti_flake_rerun_policy_reason_taxonomy_markers() {
    assert!(DOC.contains(
        "anti_flake_policy_reason_taxonomy_version=kamn.ci.anti-flake-policy-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "anti_flake_policy_reason_codes_csv=no_active_flaky_entries,active_flaky_entries_within_budget,active_flaky_entries_exceed_max,registry_validation_failed,registry_file_missing,expected_final_decision_mismatch,rerun_policy_fast_workflow_missing,rerun_policy_deep_workflow_missing,rerun_policy_bounded_retry_missing,rerun_policy_invariant_non_retry_missing,rerun_policy_excessive_retry_detected"
    ));
    assert!(DOC.contains("anti_flake_policy_reason_codes_value=none|<csv>"));
    assert!(DOC.contains("anti_flake_policy_reason_class=stable|budgeted|violation"));
    assert!(DOC.contains(
        "check_anti_flake_policy.sh --registry-file .ci/flaky-tests.txt --expected-final-decision GO --max-active-entries 0 --fast-workflow-file .github/workflows/ci-fast-gate.yml --deep-workflow-file .github/workflows/ci-deep-validate.yml --output-json /tmp/anti-flake-policy-report.json"
    ));
}
