use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(prefix: &str) -> Self {
        let unique_counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let unique_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "kamn-{prefix}-{}-{unique_counter}-{unique_time}",
            std::process::id(),
        ));
        fs::create_dir_all(&dir).expect("failed to create temporary directory");
        Self { path: dir }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn run_command(mut command: Command, context: &str) -> Output {
    command.current_dir(repo_root());
    command
        .output()
        .unwrap_or_else(|error| panic!("failed to run command for {context}: {error}"))
}

fn output_text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn assert_success(output: &Output, context: &str) {
    assert!(
        output.status.success(),
        "{context} failed unexpectedly:\n{}",
        output_text(output)
    );
}

fn assert_failure(output: &Output, context: &str) {
    assert!(
        !output.status.success(),
        "{context} succeeded unexpectedly:\n{}",
        output_text(output)
    );
}

fn load_text(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read file {}: {error}", path.display()))
}

fn extract_json_field_value<'a>(json_text: &'a str, field: &str) -> Option<&'a str> {
    let key = format!("\"{field}\"");
    let key_index = json_text.find(&key)?;
    let after_key = &json_text[key_index + key.len()..];
    let colon_index = after_key.find(':')?;
    Some(after_key[colon_index + 1..].trim_start())
}

fn json_string_field(json_text: &str, field: &str) -> Option<String> {
    let value = extract_json_field_value(json_text, field)?;
    if !value.starts_with('"') {
        return None;
    }
    let tail = &value[1..];
    let end_index = tail.find('"')?;
    Some(tail[..end_index].to_owned())
}

fn write_text(path: &Path, text: &str) {
    fs::write(path, text)
        .unwrap_or_else(|error| panic!("failed to write file {}: {error}", path.display()));
}

fn repo_path(relative_path: &str) -> PathBuf {
    repo_root().join(relative_path)
}

fn generate_sqlite_crash_recovery_reports(temp_dir: &TempDir) -> (PathBuf, PathBuf, PathBuf) {
    let summary_report = temp_dir.path().join("sqlite-crash-recovery-summary.json");
    let policy_report = temp_dir.path().join("sqlite-crash-recovery-policy.json");
    let contract_lane_report = temp_dir
        .path()
        .join("sqlite-crash-recovery-contract-lane-report.json");

    let summary_output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg("scripts/runtime/validate_sqlite_crash_recovery_live.sh")
                .arg("--mode")
                .arg("dry-run")
                .arg("--ci-fast-gate")
                .arg("PASS")
                .arg("--output-json")
                .arg(&summary_report);
            command
        },
        "generate sqlite crash-recovery dry-run summary report",
    );
    assert_success(
        &summary_output,
        "generate sqlite crash-recovery dry-run summary report",
    );

    let policy_output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg("scripts/runtime/check_sqlite_crash_recovery_live_policy.sh")
                .arg("--report-file")
                .arg(&summary_report)
                .arg("--expected-final-decision")
                .arg("GO")
                .arg("--ci-fast-gate")
                .arg("PASS")
                .arg("--output-json")
                .arg(&policy_report);
            command
        },
        "generate sqlite crash-recovery dry-run policy report",
    );
    assert_success(
        &policy_output,
        "generate sqlite crash-recovery dry-run policy report",
    );

    let contract_lane_output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg("scripts/runtime/validate_sqlite_crash_recovery_live_contract_lane.sh")
                .arg("--mode")
                .arg("dry-run")
                .arg("--ci-fast-gate")
                .arg("PASS")
                .arg("--output-json")
                .arg(&contract_lane_report)
                .arg("--policy-output-json")
                .arg(
                    temp_dir
                        .path()
                        .join("sqlite-crash-recovery-contract-lane-policy.json"),
                )
                .arg("--summary-output-json")
                .arg(
                    temp_dir
                        .path()
                        .join("sqlite-crash-recovery-contract-lane-summary.json"),
                )
                .arg("--convergence-output-json")
                .arg(
                    temp_dir
                        .path()
                        .join("sqlite-crash-recovery-contract-lane-convergence.json"),
                );
            command
        },
        "generate sqlite crash-recovery dry-run contract-lane report",
    );
    assert_success(
        &contract_lane_output,
        "generate sqlite crash-recovery dry-run contract-lane report",
    );

    (summary_report, policy_report, contract_lane_report)
}

struct CheckerRunInputs<'a> {
    summary_report: &'a Path,
    policy_report: &'a Path,
    contract_lane_report: &'a Path,
    threshold_file: &'a Path,
    strategy_doc: &'a Path,
    ops_doc: &'a Path,
    workflow_file: &'a Path,
    ci_tools_file: &'a Path,
    output_json: &'a Path,
}

fn run_checker(inputs: CheckerRunInputs<'_>) -> Output {
    run_command(
        {
            let mut command = Command::new("python3");
            command
                .arg("scripts/ci/check_sqlite_crash_recovery_ci_dry_run_governance.py")
                .arg("--sqlite-crash-recovery-summary-report-file")
                .arg(inputs.summary_report)
                .arg("--sqlite-crash-recovery-policy-report-file")
                .arg(inputs.policy_report)
                .arg("--sqlite-crash-recovery-contract-lane-report-file")
                .arg(inputs.contract_lane_report)
                .arg("--threshold-file")
                .arg(inputs.threshold_file)
                .arg("--strategy-doc")
                .arg(inputs.strategy_doc)
                .arg("--ops-doc")
                .arg(inputs.ops_doc)
                .arg("--workflow-file")
                .arg(inputs.workflow_file)
                .arg("--ci-tools-file")
                .arg(inputs.ci_tools_file)
                .arg("--output-json")
                .arg(inputs.output_json);
            command
        },
        "run sqlite crash-recovery ci dry-run governance checker",
    )
}

#[test]
fn unit_sqlite_crash_recovery_ci_dry_run_checker_accepts_valid_reports() {
    let tmp = TempDir::new("sqlite-crash-recovery-ci-dry-run-unit");
    let (summary_report, policy_report, contract_lane_report) =
        generate_sqlite_crash_recovery_reports(&tmp);
    let checker_output_file = tmp.path().join("checker-report.json");

    let checker_output = run_checker(CheckerRunInputs {
        summary_report: &summary_report,
        policy_report: &policy_report,
        contract_lane_report: &contract_lane_report,
        threshold_file: Path::new(
            "fixtures/ci/sqlite_crash_recovery_ci_dry_run_governance_thresholds.env",
        ),
        strategy_doc: Path::new("docs/ci/strategy.md"),
        ops_doc: Path::new("docs/ops/configuration.md"),
        workflow_file: Path::new(".github/workflows/ci-fast-gate.yml"),
        ci_tools_file: Path::new("scripts/ci/test_ci_tools.sh"),
        output_json: &checker_output_file,
    });
    assert_success(
        &checker_output,
        "sqlite crash-recovery ci dry-run governance checker baseline",
    );

    let text = output_text(&checker_output);
    assert!(text.contains("status=pass"));
    assert!(text.contains("final_decision=GO"));
    assert!(text.contains("reason_codes_value=none"));
    assert!(text.contains("sqlite_crash_recovery_ci_dry_run_contract_status=verified"));
    assert!(text.contains("sqlite_crash_recovery_ci_dry_run_docs_status=verified"));
    assert!(text.contains("sqlite_crash_recovery_ci_dry_run_docs_remediation_status=verified"));

    let payload = load_text(&checker_output_file);
    assert_eq!(
        json_string_field(&payload, "schema_version").as_deref(),
        Some("kamn.ci.sqlite-crash-recovery-ci-dry-run-governance-report.v1")
    );
    assert_eq!(
        json_string_field(&payload, "final_decision").as_deref(),
        Some("GO")
    );
}

#[test]
fn functional_sqlite_crash_recovery_ci_dry_run_checker_rejects_tampered_report_contract() {
    let tmp = TempDir::new("sqlite-crash-recovery-ci-dry-run-functional");
    let (summary_report, policy_report, contract_lane_report) =
        generate_sqlite_crash_recovery_reports(&tmp);
    let checker_output_file = tmp.path().join("checker-report-functional.json");

    let tampered_summary_report = tmp
        .path()
        .join("sqlite-crash-recovery-summary-tampered.json");
    let tampered_payload = load_text(&summary_report).replacen(
        "\"schema_version\": \"kamn.runtime.sqlite-crash-recovery-live-report.v1\"",
        "\"schema_version\": \"kamn.runtime.sqlite-crash-recovery-live-report.v999\"",
        1,
    );
    assert!(
        tampered_payload.contains("v999"),
        "expected tampered schema marker to be present"
    );
    write_text(&tampered_summary_report, &tampered_payload);

    let checker_output = run_checker(CheckerRunInputs {
        summary_report: &tampered_summary_report,
        policy_report: &policy_report,
        contract_lane_report: &contract_lane_report,
        threshold_file: Path::new(
            "fixtures/ci/sqlite_crash_recovery_ci_dry_run_governance_thresholds.env",
        ),
        strategy_doc: Path::new("docs/ci/strategy.md"),
        ops_doc: Path::new("docs/ops/configuration.md"),
        workflow_file: Path::new(".github/workflows/ci-fast-gate.yml"),
        ci_tools_file: Path::new("scripts/ci/test_ci_tools.sh"),
        output_json: &checker_output_file,
    });
    assert_failure(
        &checker_output,
        "sqlite crash-recovery ci dry-run governance checker with tampered report",
    );
    assert!(
        output_text(&checker_output)
            .contains("sqlite_crash_recovery_ci_dry_run_report_contract_violation"),
        "tampered report must fail closed with report contract drift reason"
    );
}

#[test]
fn integration_sqlite_crash_recovery_ci_dry_run_checker_enforces_selector_and_workflow_exclusion() {
    let tmp = TempDir::new("sqlite-crash-recovery-ci-dry-run-integration");
    let (summary_report, policy_report, contract_lane_report) =
        generate_sqlite_crash_recovery_reports(&tmp);
    let checker_output_file = tmp.path().join("checker-report-integration.json");

    let leaked_workflow = tmp.path().join("ci-fast-gate-leaked.yml");
    let mut leaked_workflow_text = load_text(&repo_path(".github/workflows/ci-fast-gate.yml"));
    leaked_workflow_text.push_str(
        "\n      - name: leaked-sqlite-run\n        run: bash scripts/runtime/validate_sqlite_crash_recovery_live.sh --mode run --ci-fast-gate FAIL --output-json /tmp/sqlite-run.json\n",
    );
    write_text(&leaked_workflow, &leaked_workflow_text);

    let checker_output = run_checker(CheckerRunInputs {
        summary_report: &summary_report,
        policy_report: &policy_report,
        contract_lane_report: &contract_lane_report,
        threshold_file: Path::new(
            "fixtures/ci/sqlite_crash_recovery_ci_dry_run_governance_thresholds.env",
        ),
        strategy_doc: Path::new("docs/ci/strategy.md"),
        ops_doc: Path::new("docs/ops/configuration.md"),
        workflow_file: &leaked_workflow,
        ci_tools_file: Path::new("scripts/ci/test_ci_tools.sh"),
        output_json: &checker_output_file,
    });
    assert_failure(
        &checker_output,
        "sqlite crash-recovery ci dry-run governance checker with leaked workflow command",
    );
    assert!(
        output_text(&checker_output)
            .contains("sqlite_crash_recovery_ci_dry_run_workflow_exclusion_drift"),
        "workflow leakage must fail closed with workflow exclusion drift reason"
    );
}

#[test]
fn regression_sqlite_crash_recovery_ci_dry_run_checker_rejects_docs_remediation_parity_drift() {
    let tmp = TempDir::new("sqlite-crash-recovery-ci-dry-run-regression");
    let (summary_report, policy_report, contract_lane_report) =
        generate_sqlite_crash_recovery_reports(&tmp);
    let checker_output_file = tmp.path().join("checker-report-regression.json");

    let strategy_doc_drifted = tmp.path().join("strategy-drift.md");
    let strategy_text = load_text(&repo_path("docs/ci/strategy.md"));
    let drifted_strategy_text = strategy_text.replacen(
        "sqlite_crash_recovery_ci_dry_run_remediation.sqlite_crash_recovery_ci_dry_run_report_contract_violation=",
        "sqlite_crash_recovery_ci_dry_run_removed_marker.sqlite_crash_recovery_ci_dry_run_report_contract_violation=",
        1,
    );
    assert_ne!(
        strategy_text, drifted_strategy_text,
        "strategy drift fixture should mutate at least one remediation marker"
    );
    write_text(&strategy_doc_drifted, &drifted_strategy_text);

    let checker_output = run_checker(CheckerRunInputs {
        summary_report: &summary_report,
        policy_report: &policy_report,
        contract_lane_report: &contract_lane_report,
        threshold_file: Path::new(
            "fixtures/ci/sqlite_crash_recovery_ci_dry_run_governance_thresholds.env",
        ),
        strategy_doc: &strategy_doc_drifted,
        ops_doc: Path::new("docs/ops/configuration.md"),
        workflow_file: Path::new(".github/workflows/ci-fast-gate.yml"),
        ci_tools_file: Path::new("scripts/ci/test_ci_tools.sh"),
        output_json: &checker_output_file,
    });
    assert_failure(
        &checker_output,
        "sqlite crash-recovery ci dry-run governance checker with drifted remediation marker",
    );
    assert!(
        output_text(&checker_output)
            .contains("sqlite_crash_recovery_ci_dry_run_docs_remediation_marker_missing"),
        "missing remediation marker must fail closed with deterministic reason"
    );
}

#[test]
fn performance_sqlite_crash_recovery_ci_dry_run_checker_stays_within_budget() {
    let tmp = TempDir::new("sqlite-crash-recovery-ci-dry-run-performance");
    let (summary_report, policy_report, contract_lane_report) =
        generate_sqlite_crash_recovery_reports(&tmp);
    let checker_output_file = tmp.path().join("checker-report-performance.json");

    let started = Instant::now();
    let checker_output = run_checker(CheckerRunInputs {
        summary_report: &summary_report,
        policy_report: &policy_report,
        contract_lane_report: &contract_lane_report,
        threshold_file: Path::new(
            "fixtures/ci/sqlite_crash_recovery_ci_dry_run_governance_thresholds.env",
        ),
        strategy_doc: Path::new("docs/ci/strategy.md"),
        ops_doc: Path::new("docs/ops/configuration.md"),
        workflow_file: Path::new(".github/workflows/ci-fast-gate.yml"),
        ci_tools_file: Path::new("scripts/ci/test_ci_tools.sh"),
        output_json: &checker_output_file,
    });
    assert_success(
        &checker_output,
        "sqlite crash-recovery ci dry-run governance checker performance run",
    );

    let elapsed = started.elapsed();
    assert!(
        elapsed.as_secs() <= 5,
        "checker runtime exceeded 5s budget window: {elapsed:?}"
    );
}
