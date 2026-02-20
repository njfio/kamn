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

fn write_text(path: &Path, text: &str) {
    fs::write(path, text)
        .unwrap_or_else(|error| panic!("failed to write file {}: {error}", path.display()));
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

fn repo_path(relative_path: &str) -> PathBuf {
    repo_root().join(relative_path)
}

fn generate_capacity_reports(temp_dir: &TempDir) -> (PathBuf, PathBuf) {
    let performance_report = temp_dir.path().join("performance-smoke-runtime.json");
    let go_no_go_report = temp_dir.path().join("go-no-go-gate-report.json");

    let performance_output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg("scripts/ci/generate_performance_smoke_report.sh")
                .arg("--lane")
                .arg("smoke")
                .arg("--workload")
                .arg("runtime")
                .arg("--output-json")
                .arg(&performance_report);
            command
        },
        "generate performance smoke report",
    );
    assert_success(&performance_output, "generate performance smoke report");

    let go_no_go_output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg("scripts/runtime/run_go_no_go_gate_lane.sh")
                .arg("--mode")
                .arg("dry-run")
                .arg("--max-seconds")
                .arg("120")
                .arg("--output-json")
                .arg(&go_no_go_report);
            command
        },
        "generate go/no-go gate dry-run report",
    );
    assert_success(&go_no_go_output, "generate go/no-go gate dry-run report");

    (performance_report, go_no_go_report)
}

struct CheckerRunInputs<'a> {
    performance_report: &'a Path,
    go_no_go_report: &'a Path,
    threshold_file: &'a Path,
    strategy_doc: &'a Path,
    workflow_file: &'a Path,
    ci_tools_file: &'a Path,
    output_json: &'a Path,
}

fn run_checker(inputs: CheckerRunInputs<'_>) -> Output {
    run_command(
        {
            let mut command = Command::new("python3");
            command
                .arg("scripts/ci/check_capacity_ci_dry_run_governance.py")
                .arg("--performance-report-file")
                .arg(inputs.performance_report)
                .arg("--go-no-go-gate-report-file")
                .arg(inputs.go_no_go_report)
                .arg("--threshold-file")
                .arg(inputs.threshold_file)
                .arg("--strategy-doc")
                .arg(inputs.strategy_doc)
                .arg("--workflow-file")
                .arg(inputs.workflow_file)
                .arg("--ci-tools-file")
                .arg(inputs.ci_tools_file)
                .arg("--output-json")
                .arg(inputs.output_json);
            command
        },
        "run capacity ci dry-run governance checker",
    )
}

#[test]
fn unit_capacity_ci_dry_run_checker_accepts_valid_reports() {
    let tmp = TempDir::new("capacity-ci-dry-run-unit");
    let (performance_report, go_no_go_report) = generate_capacity_reports(&tmp);
    let checker_output_file = tmp.path().join("checker-report-unit.json");

    let checker_output = run_checker(CheckerRunInputs {
        performance_report: &performance_report,
        go_no_go_report: &go_no_go_report,
        threshold_file: Path::new("fixtures/ci/capacity_ci_dry_run_governance_thresholds.env"),
        strategy_doc: Path::new("docs/ci/strategy.md"),
        workflow_file: Path::new(".github/workflows/ci-fast-gate.yml"),
        ci_tools_file: Path::new("scripts/ci/test_ci_tools.sh"),
        output_json: &checker_output_file,
    });
    assert_success(&checker_output, "capacity ci dry-run governance checker baseline");

    let text = output_text(&checker_output);
    assert!(text.contains("status=pass"));
    assert!(text.contains("final_decision=GO"));
    assert!(text.contains("reason_codes_value=none"));
    assert!(text.contains("capacity_ci_dry_run_contract_status=verified"));
    assert!(text.contains("capacity_ci_dry_run_docs_status=verified"));
    assert!(text.contains("capacity_ci_dry_run_docs_remediation_status=verified"));

    let payload = load_text(&checker_output_file);
    assert_eq!(
        json_string_field(&payload, "schema_version").as_deref(),
        Some("kamn.ci.capacity-ci-dry-run-governance-report.v1")
    );
    assert_eq!(
        json_string_field(&payload, "final_decision").as_deref(),
        Some("GO")
    );
}

#[test]
fn functional_capacity_ci_dry_run_checker_rejects_tampered_go_no_go_report() {
    let tmp = TempDir::new("capacity-ci-dry-run-functional");
    let (performance_report, go_no_go_report) = generate_capacity_reports(&tmp);
    let checker_output_file = tmp.path().join("checker-report-functional.json");

    let tampered_go_no_go_report = tmp.path().join("go-no-go-gate-report-tampered.json");
    let tampered_payload = load_text(&go_no_go_report).replacen(
        "\"final_decision\": \"GO\"",
        "\"final_decision\": \"NO-GO\"",
        1,
    );
    assert!(
        tampered_payload.contains("\"final_decision\": \"NO-GO\""),
        "expected tampered final_decision marker to be present"
    );
    write_text(&tampered_go_no_go_report, &tampered_payload);

    let checker_output = run_checker(CheckerRunInputs {
        performance_report: &performance_report,
        go_no_go_report: &tampered_go_no_go_report,
        threshold_file: Path::new("fixtures/ci/capacity_ci_dry_run_governance_thresholds.env"),
        strategy_doc: Path::new("docs/ci/strategy.md"),
        workflow_file: Path::new(".github/workflows/ci-fast-gate.yml"),
        ci_tools_file: Path::new("scripts/ci/test_ci_tools.sh"),
        output_json: &checker_output_file,
    });
    assert_failure(
        &checker_output,
        "capacity ci dry-run governance checker with tampered go/no-go report",
    );
    assert!(
        output_text(&checker_output).contains("capacity_ci_dry_run_go_no_go_marker_parity_drift"),
        "tampered go/no-go report must fail closed with marker parity drift reason"
    );
}

#[test]
fn integration_capacity_ci_dry_run_checker_enforces_selector_and_workflow_exclusion() {
    let tmp = TempDir::new("capacity-ci-dry-run-integration");
    let (performance_report, go_no_go_report) = generate_capacity_reports(&tmp);
    let checker_output_file = tmp.path().join("checker-report-integration.json");

    let leaked_workflow = tmp.path().join("ci-fast-gate-leaked.yml");
    let mut leaked_workflow_text = load_text(&repo_path(".github/workflows/ci-fast-gate.yml"));
    leaked_workflow_text.push_str(
        "\n      - name: leaked-capacity-run-mode\n        run: bash scripts/runtime/run_go_no_go_gate_lane.sh --mode run --output-json /tmp/go-no-go-run.json\n",
    );
    write_text(&leaked_workflow, &leaked_workflow_text);

    let checker_output = run_checker(CheckerRunInputs {
        performance_report: &performance_report,
        go_no_go_report: &go_no_go_report,
        threshold_file: Path::new("fixtures/ci/capacity_ci_dry_run_governance_thresholds.env"),
        strategy_doc: Path::new("docs/ci/strategy.md"),
        workflow_file: &leaked_workflow,
        ci_tools_file: Path::new("scripts/ci/test_ci_tools.sh"),
        output_json: &checker_output_file,
    });
    assert_failure(
        &checker_output,
        "capacity ci dry-run governance checker with leaked workflow command",
    );
    assert!(
        output_text(&checker_output).contains("capacity_ci_dry_run_workflow_exclusion_drift"),
        "workflow leakage must fail closed with workflow exclusion drift reason"
    );
}

#[test]
fn regression_capacity_ci_dry_run_checker_rejects_docs_remediation_parity_drift() {
    let tmp = TempDir::new("capacity-ci-dry-run-regression");
    let (performance_report, go_no_go_report) = generate_capacity_reports(&tmp);
    let checker_output_file = tmp.path().join("checker-report-regression.json");

    let strategy_doc_drifted = tmp.path().join("strategy-drift.md");
    let strategy_text = load_text(&repo_path("docs/ci/strategy.md"));
    let drifted_strategy_text = strategy_text.replacen(
        "capacity_ci_dry_run_remediation.capacity_ci_dry_run_report_contract_violation=",
        "capacity_ci_dry_run_removed_marker.capacity_ci_dry_run_report_contract_violation=",
        1,
    );
    assert_ne!(
        strategy_text, drifted_strategy_text,
        "strategy drift fixture should mutate at least one remediation marker"
    );
    write_text(&strategy_doc_drifted, &drifted_strategy_text);

    let checker_output = run_checker(CheckerRunInputs {
        performance_report: &performance_report,
        go_no_go_report: &go_no_go_report,
        threshold_file: Path::new("fixtures/ci/capacity_ci_dry_run_governance_thresholds.env"),
        strategy_doc: &strategy_doc_drifted,
        workflow_file: Path::new(".github/workflows/ci-fast-gate.yml"),
        ci_tools_file: Path::new("scripts/ci/test_ci_tools.sh"),
        output_json: &checker_output_file,
    });
    assert_failure(
        &checker_output,
        "capacity ci dry-run governance checker with drifted remediation marker",
    );
    assert!(
        output_text(&checker_output)
            .contains("capacity_ci_dry_run_docs_remediation_marker_missing"),
        "missing remediation marker must fail closed with deterministic reason"
    );
}

#[test]
fn performance_capacity_ci_dry_run_checker_stays_within_budget() {
    let tmp = TempDir::new("capacity-ci-dry-run-performance");
    let (performance_report, go_no_go_report) = generate_capacity_reports(&tmp);
    let checker_output_file = tmp.path().join("checker-report-performance.json");

    let started = Instant::now();
    let checker_output = run_checker(CheckerRunInputs {
        performance_report: &performance_report,
        go_no_go_report: &go_no_go_report,
        threshold_file: Path::new("fixtures/ci/capacity_ci_dry_run_governance_thresholds.env"),
        strategy_doc: Path::new("docs/ci/strategy.md"),
        workflow_file: Path::new(".github/workflows/ci-fast-gate.yml"),
        ci_tools_file: Path::new("scripts/ci/test_ci_tools.sh"),
        output_json: &checker_output_file,
    });
    assert_success(
        &checker_output,
        "capacity ci dry-run governance checker performance run",
    );

    let elapsed = started.elapsed();
    assert!(
        elapsed.as_secs() <= 5,
        "checker runtime exceeded 5s budget window: {elapsed:?}"
    );
}
