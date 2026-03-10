use super::*;
use super::fairness_deletion_support::assert_contains_all;

#[test]
fn doc_contains_performance_baseline_provenance_contract_markers() {
    assert_contains_all(
        DOC,
        &[
            "## Performance Baseline Artifact Provenance Contract",
            "fixtures/ci/performance_hot_path_fixture_matrix.json",
            "baseline_provenance.artifact_version",
            "baseline_provenance.source_commit",
            "baseline_provenance.source_run_id",
            "baseline_provenance.generated_at_utc",
            "baseline_provenance.generator",
            "drift_threshold_seed_id",
            "drift_threshold_seed.max_latency_p50_ms",
            "drift_threshold_seed.max_latency_p99_ms",
            "drift_threshold_seed.min_throughput_tps",
            "drift_threshold_seed.min_availability_pct",
            "performance_baseline_refresh_policy=manual_on_contract_change",
            "performance_baseline_refresh_contract=update fixture provenance + seed markers in the same PR as threshold-contract changes",
            "missing required baseline marker: baseline_provenance_artifact_version",
            "bash scripts/ci/test_generate_performance_smoke_report.sh",
            "bash scripts/ci/test_check_performance_thresholds.sh",
        ],
        "performance baseline",
    );
}

#[test]
fn doc_contains_performance_ci_smoke_docs_parity_and_remediation_markers() {
    assert_performance_ci_smoke_doc_headers();
    assert_performance_ci_smoke_doc_status_markers();
    assert_performance_ci_smoke_doc_commands();
    assert!(DOC.contains("Regression: #4002, #4003"));
}

#[test]
fn doc_enforces_performance_ci_smoke_docs_remediation_markers_cover_reason_codes() {
    for reason_code in performance_ci_smoke_reason_codes() {
        assert!(
            DOC.contains(&format!("performance_ci_smoke_remediation.{reason_code}=")),
            "missing performance-ci-smoke remediation marker for {reason_code}"
        );
    }
}

fn assert_performance_ci_smoke_doc_headers() {
    assert_contains_all(
        DOC,
        &[
            "## Performance CI Smoke Threshold Governance Contract",
            "bash scripts/ci/check_performance_thresholds.sh --lane smoke --report-json /tmp/performance-smoke-report.json --profile-file .ci/performance-targets.env --ci-tools-file scripts/ci/test_ci_tools.sh --workflow-file .github/workflows/ci-fast-gate.yml --strategy-doc docs/ci/strategy.md --max-seconds 120",
            "performance_ci_smoke_docs_status=verified|violation",
            "performance_ci_smoke_docs_remediation_status=verified|violation",
            "performance_ci_smoke_remediation_map_version=v1",
        ],
        "performance ci smoke",
    );
    assert!(DOC.contains(&format!(
        "performance_ci_smoke_reason_taxonomy_version={PERFORMANCE_CI_SMOKE_REASON_TAXONOMY_VERSION}"
    )));
    assert!(DOC.contains(&format!(
        "performance_ci_smoke_reason_codes_csv={PERFORMANCE_CI_SMOKE_REASON_CODES_CSV}"
    )));
}

fn assert_performance_ci_smoke_doc_status_markers() {
    assert_contains_all(
        DOC,
        &[
            "cargo test -p kamn-core --test ci_strategy_docs doc_contains_performance_ci_smoke_docs_parity_and_remediation_markers -- --exact",
            "cargo test -p kamn-core --test ci_strategy_docs doc_enforces_performance_ci_smoke_docs_remediation_markers_cover_reason_codes -- --exact",
        ],
        "performance ci smoke command",
    );
}

fn assert_performance_ci_smoke_doc_commands() {
    assert!(!performance_ci_smoke_reason_codes().is_empty());
}
