use super::*;

#[test]
fn service_api_ops_configuration_contains_sbom_provenance_artifact_generator_markers() {
    assert_doc_contains_all(&["## SBOM-Provenance Artifact Generator Contract (Issue #4036)", "sbom_provenance_lane_schema_version=kamn.runtime.sbom-provenance-artifact-report.v1", "sbom_provenance_artifact_schema_version=kamn.runtime.sbom-provenance-artifact-schema.v1", "sbom_provenance_fixture_schema_version=kamn.ci.sbom-provenance-artifact-fixture-matrix.v1", "sbom_provenance_reason_taxonomy_version=kamn.runtime.sbom-provenance-artifact-reason-taxonomy.v1", "sbom_provenance_reason_codes_csv=sbom_provenance_profile_contract_violation,sbom_provenance_runtime_budget_exceeded", "sbom_schema_version=spdx-2.3", "provenance_schema_version=slsa-v1", "release_manifest_required_artifact_id=sbom_provenance", "cargo run -p kamn-core --bin sbom_provenance_artifact_generator_contract -- --profile baseline --mode dry-run --ci-fast-gate PASS --max-seconds 120 --output-json /tmp/sbom-provenance-baseline.json", "cargo run -p kamn-core --bin sbom_provenance_artifact_generator_contract -- --profile injected-drift --mode dry-run --ci-fast-gate PASS --max-seconds 120 --output-json /tmp/sbom-provenance-injected-drift.json", "cargo test -p kamn-core --test sbom_provenance_artifact_generator_contract -- --nocapture", "Regression: #4036"]);
}
#[test]
fn service_api_ops_configuration_contains_sbom_provenance_release_gonogo_checker_markers() {
    assert!(DOC.contains("## SBOM-Provenance Release Go-No-Go Checker Contract (Issue #4037)"));
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
        "python3 scripts/deploy/sbom_provenance_release_gonogo_checker_contract.py --artifact-json /tmp/sbom-provenance-baseline.json --ci-strategy-doc docs/ci/strategy.md --ops-configuration-doc docs/ops/configuration.md --max-seconds 120 --output-json /tmp/sbom-provenance-release-gonogo-checker.json"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test sbom_provenance_release_gonogo_checker_contract -- --nocapture"
    ));
    assert!(DOC.contains("Regression: #4037"));
}
