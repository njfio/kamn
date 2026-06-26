use super::support::assert_doc_contains_all;

#[test]
fn doc_contains_sbom_provenance_artifact_generator_contract_markers() {
    assert_doc_contains_all(
        sbom_artifact_generator_markers(),
        "sbom provenance artifact",
    );
}

#[test]
fn doc_contains_sbom_provenance_release_gonogo_checker_contract_markers() {
    assert_doc_contains_all(
        sbom_release_gonogo_markers(),
        "sbom provenance release gonogo",
    );
}

#[test]
fn doc_contains_dependency_ci_smoke_advisory_fixture_contract_markers() {
    assert_doc_contains_all(
        dependency_ci_smoke_advisory_fixture_markers(),
        "dependency ci smoke advisory fixture",
    );
}

#[test]
fn doc_contains_dependency_ci_smoke_checker_threshold_parity_markers() {
    assert_doc_contains_all(
        dependency_ci_smoke_checker_threshold_markers(),
        "dependency ci smoke checker threshold parity",
    );
}

#[test]
fn doc_contains_cargo_audit_runner_impact_measurement_markers() {
    assert_doc_contains_all(cargo_audit_runner_markers(), "cargo audit runner impact");
}

#[test]
fn doc_contains_anti_flake_rerun_policy_reason_taxonomy_markers() {
    assert_doc_contains_all(anti_flake_policy_markers(), "anti flake rerun policy");
}

fn sbom_artifact_generator_markers() -> &'static [&'static str] {
    &[
        "## SBOM-Provenance Artifact Generator Contract (Issue #4036)",
        "cargo run -p kamn-core --bin sbom_provenance_artifact_generator_contract -- --profile baseline --mode dry-run --ci-fast-gate PASS --max-seconds 120 --output-json /tmp/sbom-provenance-baseline.json",
        "cargo run -p kamn-core --bin sbom_provenance_artifact_generator_contract -- --profile injected-drift --mode dry-run --ci-fast-gate PASS --max-seconds 120 --output-json /tmp/sbom-provenance-injected-drift.json",
        "kamn.runtime.sbom-provenance-artifact-report.v1",
        "kamn.runtime.sbom-provenance-artifact-schema.v1",
        "kamn.ci.sbom-provenance-artifact-fixture-matrix.v1",
        "kamn.runtime.sbom-provenance-artifact-reason-taxonomy.v1",
        "sbom_provenance_reason_codes_csv=sbom_provenance_profile_contract_violation,sbom_provenance_runtime_budget_exceeded",
        "sbom_schema_version=spdx-2.3",
        "provenance_schema_version=slsa-v1",
        "release_manifest_required_artifact_id=sbom_provenance",
        "cargo test -p kamn-core --test sbom_provenance_artifact_generator_contract -- --nocapture",
        "Regression: #4036",
    ]
}

fn sbom_release_gonogo_markers() -> &'static [&'static str] {
    &[
        "## SBOM-Provenance Release Go-No-Go Checker Contract (Issue #4037)",
        "python3 scripts/deploy/sbom_provenance_release_gonogo_checker_contract.py --artifact-json /tmp/sbom-provenance-baseline.json --ci-strategy-doc docs/ci/strategy.md --ops-configuration-doc docs/ops/configuration.md --max-seconds 120 --output-json /tmp/sbom-provenance-release-gonogo-checker.json",
        "sbom_provenance_release_gonogo_checker_schema_version=kamn.runtime.sbom-provenance-release-gonogo-checker-report.v1",
        "sbom_provenance_release_gonogo_checker_reason_taxonomy_version=kamn.runtime.sbom-provenance-release-gonogo-checker-reason-taxonomy.v1",
        "sbom_provenance_release_gonogo_checker_reason_codes_csv=sbom_provenance_artifact_marker_missing,sbom_provenance_artifact_marker_invalid,sbom_provenance_artifact_decision_not_go,sbom_provenance_docs_parity_marker_missing,sbom_provenance_runtime_budget_exceeded",
        "sbom_provenance_release_gonogo_required_artifact_markers_csv=schema_version,artifact_schema_version,fixture_schema_version,reason_taxonomy_version,release_manifest_required_artifact_id,status,final_decision,reason_code",
        "sbom_provenance_release_gonogo_docs_parity_required_markers_csv=sbom_provenance_release_gonogo_checker_schema_version,sbom_provenance_release_gonogo_checker_reason_taxonomy_version,sbom_provenance_release_gonogo_checker_reason_codes_csv,sbom_provenance_release_gonogo_required_artifact_markers_csv",
        "cargo test -p kamn-core --test sbom_provenance_release_gonogo_checker_contract -- --nocapture",
        "Regression: #4037",
    ]
}

fn dependency_ci_smoke_advisory_fixture_markers() -> &'static [&'static str] {
    &[
        "## Dependency CI Smoke Advisory Fixture Contract",
        "dependency_ci_smoke_reason_taxonomy_version=kamn.ci.dependency-ci-smoke-reason-taxonomy.v1",
        "dependency_ci_smoke_reason_codes_csv=dependency_advisory_severity_unknown,dependency_advisory_threshold_exceeded",
        "dependency_ci_smoke_fixture_schema_version=kamn.ci.dependency-ci-smoke-advisory-fixture-matrix.v1",
        "dependency_ci_smoke_fixture_path=fixtures/ci/dependency_ci_smoke_advisory_fixture_matrix.txt",
        "dependency_ci_smoke_threshold_max_severity=moderate",
        "cargo test -p kamn-core --test dependency_ci_smoke_advisory_fixture_parser_contract",
        "cargo test -p kamn-core --test ci_strategy_docs doc_contains_dependency_ci_smoke_advisory_fixture_contract_markers -- --exact",
        "Regression: #4030",
    ]
}

fn dependency_ci_smoke_checker_threshold_markers() -> &'static [&'static str] {
    &[
        "dependency_ci_smoke_checker_reason_taxonomy_version=kamn.ci.dependency-ci-smoke-reason-taxonomy.v1",
        "dependency_ci_smoke_checker_reason_codes_csv=dependency_advisory_input_empty,dependency_advisory_severity_unknown,dependency_advisory_threshold_exceeded",
        "dependency_ci_smoke_checker_fixture_schema_version=kamn.ci.dependency-ci-smoke-advisory-fixture-matrix.v1",
        "dependency_ci_smoke_checker_fixture_path=fixtures/ci/dependency_ci_smoke_advisory_fixture_matrix.txt",
        "dependency_ci_smoke_checker_threshold_max_severity=moderate",
        "dependency_ci_smoke_checker_remediation.dependency_advisory_input_empty=provide at least one advisory record from the CI smoke advisory feed before evaluating thresholds",
        "dependency_ci_smoke_checker_remediation.dependency_advisory_severity_unknown=normalize advisory severity to low|moderate|high|critical before evaluation",
        "dependency_ci_smoke_checker_remediation.dependency_advisory_threshold_exceeded=reduce dependency advisory severity exposure or update approved threshold with review evidence",
        "cargo test -p kamn-core --test dependency_ci_smoke_checker_contract",
        "cargo test -p kamn-core --test ci_strategy_docs doc_contains_dependency_ci_smoke_checker_threshold_parity_markers -- --exact",
        "Regression: #4031",
    ]
}

fn cargo_audit_runner_markers() -> &'static [&'static str] {
    &[
        "cargo_audit_runner_impact_method=github-actions-step-duration+policy-output",
        "cargo_audit_fast_gate_observed_seconds=156",
        "cargo_audit_workspace_premerge_observed_seconds=156",
        "cargo_audit_runner_impact_baseline_captured_at=2026-03-05",
        "cargo_audit_runner_impact_source_fast_gate_run=22707945091",
        "cargo_audit_runner_impact_source_job=65838851460",
        "cargo_audit_policy_elapsed_seconds=<float>",
        "cargo test -p kamn-core --test ci_strategy_docs doc_contains_cargo_audit_runner_impact_measurement_markers -- --exact",
    ]
}

fn anti_flake_policy_markers() -> &'static [&'static str] {
    &[
        "anti_flake_policy_reason_taxonomy_version=kamn.ci.anti-flake-policy-reason-taxonomy.v1",
        "anti_flake_policy_reason_codes_csv=no_active_flaky_entries,active_flaky_entries_within_budget,active_flaky_entries_exceed_max,registry_validation_failed,registry_file_missing,expected_final_decision_mismatch,rerun_policy_fast_workflow_missing,rerun_policy_deep_workflow_missing,rerun_policy_bounded_retry_missing,rerun_policy_invariant_non_retry_missing,rerun_policy_excessive_retry_detected",
        "anti_flake_policy_reason_codes_value=none|<csv>",
        "anti_flake_policy_reason_class=stable|budgeted|violation",
        "check_anti_flake_policy.sh --registry-file .ci/flaky-tests.txt --expected-final-decision GO --max-active-entries 0 --fast-workflow-file .github/workflows/ci-fast-gate.yml --deep-workflow-file .github/workflows/ci-deep-validate.yml --output-json /tmp/anti-flake-policy-report.json",
    ]
}
