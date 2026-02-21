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

fn repo_path(relative_path: &str) -> PathBuf {
    repo_root().join(relative_path)
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

fn generate_runtime_smoke_report(temp_dir: &TempDir) -> PathBuf {
    let report = temp_dir.path().join("runtime-smoke-report.json");
    let output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg("scripts/ci/generate_performance_smoke_report.sh")
                .arg("--lane")
                .arg("smoke")
                .arg("--workload")
                .arg("runtime")
                .arg("--output-json")
                .arg(&report);
            command
        },
        "generate runtime smoke report",
    );
    assert_success(&output, "generate runtime smoke report");
    report
}

struct CheckerRunInputs<'a> {
    report_file: &'a Path,
    profile_file: &'a Path,
    ci_tools_file: &'a Path,
    workflow_file: &'a Path,
    strategy_doc: &'a Path,
    max_seconds: &'a str,
}

fn run_checker(inputs: CheckerRunInputs<'_>) -> Output {
    run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg("scripts/ci/check_performance_thresholds.sh")
                .arg("--lane")
                .arg("smoke")
                .arg("--report-json")
                .arg(inputs.report_file)
                .arg("--profile-file")
                .arg(inputs.profile_file)
                .arg("--ci-tools-file")
                .arg(inputs.ci_tools_file)
                .arg("--workflow-file")
                .arg(inputs.workflow_file)
                .arg("--strategy-doc")
                .arg(inputs.strategy_doc)
                .arg("--max-seconds")
                .arg(inputs.max_seconds);
            command
        },
        "run performance ci smoke checker",
    )
}

#[test]
fn unit_performance_ci_smoke_checker_accepts_valid_smoke_report() {
    let tmp = TempDir::new("performance-ci-smoke-unit");
    let report = generate_runtime_smoke_report(&tmp);

    let checker_output = run_checker(CheckerRunInputs {
        report_file: &report,
        profile_file: Path::new(".ci/performance-targets.env"),
        ci_tools_file: Path::new("scripts/ci/test_ci_tools.sh"),
        workflow_file: Path::new(".github/workflows/ci-fast-gate.yml"),
        strategy_doc: Path::new("docs/ci/strategy.md"),
        max_seconds: "120",
    });
    assert_success(&checker_output, "performance ci smoke checker baseline");

    let text = output_text(&checker_output);
    assert!(text.contains("status=pass"));
    assert!(text.contains("final_decision=GO"));
    assert!(
        text.contains(
            "performance_ci_smoke_reason_taxonomy_version=kamn.ci.performance-ci-smoke-threshold-reason-taxonomy.v1"
        ),
        "reason taxonomy marker missing from checker output"
    );
    assert!(
        text.contains("performance_ci_smoke_reason_codes_value=none"),
        "baseline checker run must report no reason codes"
    );
    assert!(
        text.contains("performance_ci_smoke_selector_status=verified"),
        "selector contract status must remain verified"
    );
    assert!(
        text.contains("performance_ci_smoke_workflow_status=verified"),
        "workflow contract status must remain verified"
    );
    assert!(
        text.contains("performance_ci_smoke_docs_status=verified"),
        "docs marker contract status must remain verified"
    );
    assert!(
        text.contains("performance_ci_smoke_docs_remediation_status=verified"),
        "docs remediation contract status must remain verified"
    );
}

#[test]
fn functional_performance_ci_smoke_checker_rejects_threshold_breach() {
    let tmp = TempDir::new("performance-ci-smoke-functional");
    let report = generate_runtime_smoke_report(&tmp);

    let breached_report = tmp.path().join("runtime-smoke-report-breached.json");
    let breached_payload =
        load_text(&report).replacen("\"latency_p50_ms\": 92", "\"latency_p50_ms\": 999", 1);
    assert_ne!(
        load_text(&report),
        breached_payload,
        "threshold breach fixture should mutate latency marker"
    );
    write_text(&breached_report, &breached_payload);

    let checker_output = run_checker(CheckerRunInputs {
        report_file: &breached_report,
        profile_file: Path::new(".ci/performance-targets.env"),
        ci_tools_file: Path::new("scripts/ci/test_ci_tools.sh"),
        workflow_file: Path::new(".github/workflows/ci-fast-gate.yml"),
        strategy_doc: Path::new("docs/ci/strategy.md"),
        max_seconds: "120",
    });
    assert_failure(
        &checker_output,
        "performance ci smoke checker with breached threshold",
    );
    assert!(
        output_text(&checker_output)
            .contains("performance_ci_smoke_latency_p50_threshold_exceeded"),
        "threshold breach must fail closed with deterministic p50 reason code"
    );
}

#[test]
fn integration_performance_ci_smoke_checker_detects_selector_and_workflow_drift() {
    let tmp = TempDir::new("performance-ci-smoke-integration");
    let report = generate_runtime_smoke_report(&tmp);

    let selector_drift_file = tmp.path().join("selector-drift-test-ci-tools.sh");
    let selector_text = load_text(&repo_path("scripts/ci/test_ci_tools.sh")).replacen(
        "cargo test -p kamn-core --test performance_ci_smoke_governance_contract -- --nocapture\n",
        "",
        1,
    );
    write_text(&selector_drift_file, &selector_text);

    let selector_output = run_checker(CheckerRunInputs {
        report_file: &report,
        profile_file: Path::new(".ci/performance-targets.env"),
        ci_tools_file: &selector_drift_file,
        workflow_file: Path::new(".github/workflows/ci-fast-gate.yml"),
        strategy_doc: Path::new("docs/ci/strategy.md"),
        max_seconds: "120",
    });
    assert_failure(
        &selector_output,
        "performance ci smoke checker selector drift fixture",
    );
    assert!(
        output_text(&selector_output)
            .contains("performance_ci_smoke_selector_missing_checker_entry"),
        "selector drift must fail closed with deterministic selector reason"
    );

    let workflow_drift_file = tmp.path().join("workflow-drift-ci-fast-gate.yml");
    let mut workflow_text = load_text(&repo_path(".github/workflows/ci-fast-gate.yml"));
    workflow_text.push_str(
        "\n      - name: leaked performance deep checker\n        run: bash scripts/ci/check_performance_thresholds.sh --lane deep --report-json performance-smoke-report.json --profile-file .ci/performance-targets.env\n",
    );
    write_text(&workflow_drift_file, &workflow_text);

    let workflow_output = run_checker(CheckerRunInputs {
        report_file: &report,
        profile_file: Path::new(".ci/performance-targets.env"),
        ci_tools_file: Path::new("scripts/ci/test_ci_tools.sh"),
        workflow_file: &workflow_drift_file,
        strategy_doc: Path::new("docs/ci/strategy.md"),
        max_seconds: "120",
    });
    assert_failure(
        &workflow_output,
        "performance ci smoke checker workflow drift fixture",
    );
    assert!(
        output_text(&workflow_output)
            .contains("performance_ci_smoke_workflow_forbidden_entry_present"),
        "workflow drift must fail closed with deterministic workflow reason"
    );

    let docs_marker_drift_file = tmp.path().join("strategy-marker-drift.md");
    let docs_marker_text = load_text(&repo_path("docs/ci/strategy.md")).replacen(
        "performance_ci_smoke_workflow_forbidden_entry=bash scripts/ci/check_performance_thresholds.sh --lane deep",
        "performance_ci_smoke_workflow_forbidden_entry=drifted-marker",
        1,
    );
    write_text(&docs_marker_drift_file, &docs_marker_text);

    let docs_marker_output = run_checker(CheckerRunInputs {
        report_file: &report,
        profile_file: Path::new(".ci/performance-targets.env"),
        ci_tools_file: Path::new("scripts/ci/test_ci_tools.sh"),
        workflow_file: Path::new(".github/workflows/ci-fast-gate.yml"),
        strategy_doc: &docs_marker_drift_file,
        max_seconds: "120",
    });
    assert_failure(
        &docs_marker_output,
        "performance ci smoke checker docs marker drift fixture",
    );
    assert!(
        output_text(&docs_marker_output).contains("performance_ci_smoke_docs_marker_parity_drift"),
        "docs marker drift must fail closed with deterministic docs parity reason"
    );
}

#[test]
fn regression_performance_ci_smoke_checker_rejects_report_contract_drift() {
    let tmp = TempDir::new("performance-ci-smoke-regression");
    let report = generate_runtime_smoke_report(&tmp);

    let drifted_report = tmp.path().join("runtime-smoke-report-drifted.json");
    let drifted_payload = load_text(&report).replace(
        "\"baseline_provenance_artifact_version\": \"r27.9-v1\",\n",
        "",
    );
    assert_ne!(
        load_text(&report),
        drifted_payload,
        "regression fixture should remove baseline marker"
    );
    write_text(&drifted_report, &drifted_payload);

    let checker_output = run_checker(CheckerRunInputs {
        report_file: &drifted_report,
        profile_file: Path::new(".ci/performance-targets.env"),
        ci_tools_file: Path::new("scripts/ci/test_ci_tools.sh"),
        workflow_file: Path::new(".github/workflows/ci-fast-gate.yml"),
        strategy_doc: Path::new("docs/ci/strategy.md"),
        max_seconds: "120",
    });
    assert_failure(
        &checker_output,
        "performance ci smoke checker with report marker drift",
    );
    assert!(
        output_text(&checker_output).contains("performance_ci_smoke_report_contract_violation"),
        "report marker drift must fail closed with report contract reason"
    );
}

#[test]
fn regression_performance_ci_smoke_checker_rejects_docs_remediation_drift() {
    let tmp = TempDir::new("performance-ci-smoke-regression-remediation");
    let report = generate_runtime_smoke_report(&tmp);

    let docs_remediation_drift_file = tmp.path().join("strategy-missing-remediation.md");
    let docs_remediation_text = load_text(&repo_path("docs/ci/strategy.md")).replacen(
        "performance_ci_smoke_remediation.performance_ci_smoke_runtime_budget_exceeded=reduce checker/report overhead or raise max-seconds with explicit review evidence",
        "",
        1,
    );
    write_text(&docs_remediation_drift_file, &docs_remediation_text);

    let checker_output = run_checker(CheckerRunInputs {
        report_file: &report,
        profile_file: Path::new(".ci/performance-targets.env"),
        ci_tools_file: Path::new("scripts/ci/test_ci_tools.sh"),
        workflow_file: Path::new(".github/workflows/ci-fast-gate.yml"),
        strategy_doc: &docs_remediation_drift_file,
        max_seconds: "120",
    });
    assert_failure(
        &checker_output,
        "performance ci smoke checker with docs remediation drift",
    );
    assert!(
        output_text(&checker_output)
            .contains("performance_ci_smoke_docs_remediation_marker_missing"),
        "missing docs remediation marker must fail closed with deterministic reason"
    );
}

#[test]
fn performance_performance_ci_smoke_checker_stays_within_budget() {
    let tmp = TempDir::new("performance-ci-smoke-performance");
    let report = generate_runtime_smoke_report(&tmp);

    let started = Instant::now();
    let checker_output = run_checker(CheckerRunInputs {
        report_file: &report,
        profile_file: Path::new(".ci/performance-targets.env"),
        ci_tools_file: Path::new("scripts/ci/test_ci_tools.sh"),
        workflow_file: Path::new(".github/workflows/ci-fast-gate.yml"),
        strategy_doc: Path::new("docs/ci/strategy.md"),
        max_seconds: "120",
    });
    assert_success(
        &checker_output,
        "performance ci smoke checker performance baseline",
    );

    let elapsed = started.elapsed();
    assert!(
        elapsed.as_secs() <= 5,
        "checker runtime exceeded 5s budget window: {elapsed:?}"
    );
}
