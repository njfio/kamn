use super::super::support::*;

const PASS_REPORT_JSON: &str = r#"{
  "latency_p50_ms": 88,
  "latency_p99_ms": 300,
  "throughput_tps": 12500,
  "availability_pct": 99.95,
  "baseline_provenance_artifact_version": "kamn.ci.performance-baseline.v1",
  "baseline_provenance_source_commit": "abc123def456",
  "baseline_provenance_source_run_id": "run-1001",
  "baseline_provenance_generated_at_utc": "2026-02-20T12:00:00Z",
  "baseline_provenance_generator": "scripts/ci/generate_performance_smoke_report.sh",
  "drift_threshold_seed_id": "smoke-seed-v1",
  "drift_threshold_seed_max_latency_p50_ms": 95,
  "drift_threshold_seed_max_latency_p99_ms": 350,
  "drift_threshold_seed_min_throughput_tps": 12000,
  "drift_threshold_seed_min_availability_pct": 99.9
}
"#;

const FAIL_REPORT_JSON: &str = r#"{
  "latency_p50_ms": 101,
  "latency_p99_ms": 540,
  "throughput_tps": 9800,
  "availability_pct": 99.7,
  "baseline_provenance_artifact_version": "kamn.ci.performance-baseline.v1",
  "baseline_provenance_source_commit": "abc123def456",
  "baseline_provenance_source_run_id": "run-1001",
  "baseline_provenance_generated_at_utc": "2026-02-20T12:00:00Z",
  "baseline_provenance_generator": "scripts/ci/generate_performance_smoke_report.sh",
  "drift_threshold_seed_id": "smoke-seed-v1",
  "drift_threshold_seed_max_latency_p50_ms": 95,
  "drift_threshold_seed_max_latency_p99_ms": 350,
  "drift_threshold_seed_min_throughput_tps": 12000,
  "drift_threshold_seed_min_availability_pct": 99.9
}
"#;

const INVALID_REPORT_JSON: &str = r#"{
  "latency_p50_ms": 90,
  "throughput_tps": 12000
}
"#;

#[test]
fn spec_c03_performance_threshold_checker_contract() {
    let checker = repo_path("scripts/ci/check_performance_thresholds.sh");
    let profile_file = repo_path(".ci/performance-targets.env");
    assert!(
        checker.is_file() && profile_file.is_file(),
        "performance threshold checker and profile fixture must exist"
    );

    let tmp = TempDir::new("performance-thresholds");
    assert_pass_case(&checker, &profile_file, tmp.path());
    assert_fail_case(&checker, &profile_file, tmp.path());
    assert_invalid_case(&checker, &profile_file, tmp.path());
}

fn write_report(tmp_root: &Path, file_name: &str, contents: &str) -> PathBuf {
    let report = tmp_root.join(file_name);
    fs::write(&report, contents).expect("failed to write performance report fixture");
    report
}

fn run_threshold_case(
    checker: &Path,
    profile_file: &Path,
    report: &Path,
    label: &str,
) -> CommandOutput {
    run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg(checker)
                .arg("--report-json")
                .arg(report)
                .arg("--profile-file")
                .arg(profile_file)
                .arg("--lane")
                .arg("smoke");
            command
        },
        label,
    )
}

fn assert_pass_case(checker: &Path, profile_file: &Path, tmp_root: &Path) {
    let report = write_report(tmp_root, "pass.json", PASS_REPORT_JSON);
    let output = run_threshold_case(
        checker,
        profile_file,
        &report,
        "performance threshold pass case",
    );
    assert_success(&output, "performance threshold pass case");
    assert!(
        output_text(&output).contains("status=pass"),
        "pass case must emit status=pass"
    );
}

fn assert_fail_case(checker: &Path, profile_file: &Path, tmp_root: &Path) {
    let report = write_report(tmp_root, "fail.json", FAIL_REPORT_JSON);
    let output = run_threshold_case(
        checker,
        profile_file,
        &report,
        "performance threshold fail case",
    );
    assert_failure(&output, "performance threshold fail case");
    let output_text = output_text(&output);
    assert_contains_all(
        &output_text,
        &[
            "status=fail",
            "performance_ci_smoke_reason_codes_value=",
            "performance_ci_smoke_latency_p50_threshold_exceeded",
            "performance_ci_smoke_throughput_threshold_below_minimum",
        ],
        "performance threshold fail markers",
    );
}

fn assert_invalid_case(checker: &Path, profile_file: &Path, tmp_root: &Path) {
    let report = write_report(tmp_root, "invalid.json", INVALID_REPORT_JSON);
    let output = run_threshold_case(
        checker,
        profile_file,
        &report,
        "performance threshold invalid-schema case",
    );
    assert_failure(&output, "performance threshold invalid-schema case");
    assert!(
        output_text(&output).contains("performance_ci_smoke_report_contract_violation"),
        "invalid-schema case must emit deterministic report-contract violation reason"
    );
}
