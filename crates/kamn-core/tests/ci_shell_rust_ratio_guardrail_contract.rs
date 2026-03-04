use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

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
            "kamn-{}-{}-{}-{}",
            prefix,
            std::process::id(),
            unique_counter,
            unique_time
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

fn repo_path(relative: &str) -> PathBuf {
    repo_root().join(relative)
}

fn read_text(relative: &str) -> String {
    fs::read_to_string(repo_path(relative))
        .unwrap_or_else(|error| panic!("failed to read {relative}: {error}"))
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

fn run_guardrail_checker(threshold_file: &Path, report_file: &Path) -> Output {
    let checker = repo_path("scripts/ci/check_shell_rust_ratio_guardrail.sh");
    run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg(&checker)
                .arg("--repo-root")
                .arg(repo_root())
                .arg("--threshold-file")
                .arg(threshold_file)
                .arg("--output-json")
                .arg(report_file);
            command
        },
        "shell-rust ratio guardrail checker",
    )
}

fn write_threshold_file(path: &Path, warn: Option<&str>, fail: Option<&str>) {
    let mut content = String::new();
    if let Some(warn_value) = warn {
        content.push_str(&format!("WARN_SHELL_RUST_RATIO_MAX={warn_value}\n"));
    }
    if let Some(fail_value) = fail {
        content.push_str(&format!("FAIL_SHELL_RUST_RATIO_MAX={fail_value}\n"));
    }
    fs::write(path, content).unwrap_or_else(|error| {
        panic!("failed to write threshold file {}: {error}", path.display())
    });
}

#[test]
fn spec_c01_ci_tools_fast_mode_routes_guardrail_regression_lane_to_rust() {
    let ci_tools = read_text("scripts/ci/test_ci_tools.sh");
    assert!(
        ci_tools.contains("cargo test -p kamn-core --test ci_shell_rust_ratio_guardrail_contract"),
        "fast-mode CI tools selector must include Rust guardrail contract lane"
    );
    assert!(
        !ci_tools
            .contains("bash \"$ROOT_DIR/scripts/ci/test_check_shell_rust_ratio_guardrail.sh\""),
        "fast-mode CI tools selector must not call retired shell guardrail regression wrapper"
    );
}

#[test]
fn spec_c02_guardrail_wrapper_remains_thin_delegate() {
    let checker = repo_path("scripts/ci/check_shell_rust_ratio_guardrail.sh");
    assert!(checker.is_file(), "guardrail checker wrapper must exist");
    let checker_text = fs::read_to_string(&checker).expect("failed to read guardrail wrapper");
    let line_count = checker_text.lines().count();
    assert!(
        line_count <= 20,
        "guardrail wrapper shell surface must remain <=20 lines (found {line_count})"
    );
    assert!(
        checker_text.contains("check_shell_rust_ratio_guardrail.py"),
        "guardrail wrapper must delegate to python implementation"
    );
}

#[test]
fn spec_c03_guardrail_pass_path_contract() {
    let tmp = TempDir::new("shell-rust-ratio-pass");
    let threshold_file = tmp.path().join("pass-thresholds.env");
    let report_file = tmp.path().join("pass-report.json");
    write_threshold_file(&threshold_file, Some("999"), Some("1000"));

    let output = run_guardrail_checker(&threshold_file, &report_file);
    assert_success(&output, "guardrail pass path");
    let text = output_text(&output);
    assert!(
        text.contains("status=ok"),
        "expected status=ok marker on pass path:\n{text}"
    );
    assert!(
        text.contains("final_decision=GO"),
        "expected final_decision=GO marker on pass path:\n{text}"
    );
    assert!(
        text.contains("reason_codes=none"),
        "expected reason_codes=none marker on pass path:\n{text}"
    );
    assert!(
        text.contains(
            "reason_taxonomy_version=kamn.ci.shell-rust-ratio-guardrail-reason-taxonomy.v1"
        ),
        "expected deterministic taxonomy marker on pass path:\n{text}"
    );
    assert!(
        text.contains("python_line_total="),
        "expected python_line_total marker on pass path:\n{text}"
    );
    assert!(
        text.contains("tracked_python_file_count="),
        "expected tracked_python_file_count marker on pass path:\n{text}"
    );
    let report_text =
        fs::read_to_string(&report_file).expect("failed to read guardrail pass report");
    assert!(
        report_text.contains("\"final_decision\": \"GO\""),
        "expected GO final_decision in pass report JSON"
    );
    assert!(
        report_text.contains("\"python_line_total\""),
        "expected python_line_total metric in pass report JSON"
    );
    assert!(
        report_text.contains("\"tracked_python_file_count\""),
        "expected tracked_python_file_count metric in pass report JSON"
    );
}

#[test]
fn spec_c04_guardrail_warn_path_contract() {
    let tmp = TempDir::new("shell-rust-ratio-warn");
    let threshold_file = tmp.path().join("warn-thresholds.env");
    let report_file = tmp.path().join("warn-report.json");
    write_threshold_file(&threshold_file, Some("0.10"), Some("1000"));

    let output = run_guardrail_checker(&threshold_file, &report_file);
    assert_success(&output, "guardrail warn path");
    let text = output_text(&output);
    assert!(
        text.contains("status=ok"),
        "expected status=ok marker on warn path:\n{text}"
    );
    assert!(
        text.contains("final_decision=WARN"),
        "expected final_decision=WARN marker on warn path:\n{text}"
    );
    assert!(
        text.contains("reason_codes=shell_rust_ratio_warn_threshold_exceeded"),
        "expected warn-threshold reason marker on warn path:\n{text}"
    );
    assert!(
        text.contains("python_line_total="),
        "expected python_line_total marker on warn path:\n{text}"
    );
    assert!(
        text.contains("tracked_python_file_count="),
        "expected tracked_python_file_count marker on warn path:\n{text}"
    );
}

#[test]
fn spec_c05_guardrail_fail_threshold_contract() {
    let tmp = TempDir::new("shell-rust-ratio-fail");
    let threshold_file = tmp.path().join("fail-thresholds.env");
    let report_file = tmp.path().join("fail-report.json");
    write_threshold_file(&threshold_file, Some("0.01"), Some("0.10"));

    let output = run_guardrail_checker(&threshold_file, &report_file);
    assert_failure(&output, "guardrail fail threshold path");
    let text = output_text(&output);
    assert!(
        text.contains("status=fail"),
        "expected status=fail marker on fail path:\n{text}"
    );
    assert!(
        text.contains("final_decision=NO-GO"),
        "expected final_decision=NO-GO marker on fail path:\n{text}"
    );
    assert!(
        text.contains("reason_codes=shell_rust_ratio_fail_threshold_exceeded"),
        "expected fail-threshold reason marker on fail path:\n{text}"
    );
    assert!(
        text.contains("python_line_total="),
        "expected python_line_total marker on fail path:\n{text}"
    );
    assert!(
        text.contains("tracked_python_file_count="),
        "expected tracked_python_file_count marker on fail path:\n{text}"
    );
}

#[test]
fn spec_c06_guardrail_threshold_validation_contract() {
    let tmp = TempDir::new("shell-rust-ratio-validate");

    let missing_key_threshold_file = tmp.path().join("missing-key-thresholds.env");
    let missing_key_report_file = tmp.path().join("missing-key-report.json");
    write_threshold_file(&missing_key_threshold_file, Some("0.90"), None);

    let missing_key_output =
        run_guardrail_checker(&missing_key_threshold_file, &missing_key_report_file);
    assert_failure(&missing_key_output, "guardrail missing-key path");
    let missing_key_text = output_text(&missing_key_output);
    assert!(
        missing_key_text.contains("reason_codes=shell_rust_ratio_threshold_key_missing"),
        "expected missing-key reason marker:\n{missing_key_text}"
    );
    assert!(
        missing_key_text.contains("python_line_total=unknown"),
        "expected unknown python_line_total marker on missing-key path:\n{missing_key_text}"
    );
    assert!(
        missing_key_text.contains("tracked_python_file_count=unknown"),
        "expected unknown tracked_python_file_count marker on missing-key path:\n{missing_key_text}"
    );

    let order_threshold_file = tmp.path().join("order-thresholds.env");
    let order_report_file = tmp.path().join("order-report.json");
    write_threshold_file(&order_threshold_file, Some("1.00"), Some("1.00"));

    let order_output = run_guardrail_checker(&order_threshold_file, &order_report_file);
    assert_failure(&order_output, "guardrail threshold-order path");
    let order_text = output_text(&order_output);
    assert!(
        order_text.contains("reason_codes=shell_rust_ratio_threshold_order_invalid"),
        "expected threshold-order reason marker:\n{order_text}"
    );
    assert!(
        order_text.contains("python_line_total=unknown"),
        "expected unknown python_line_total marker on threshold-order path:\n{order_text}"
    );
    assert!(
        order_text.contains("tracked_python_file_count=unknown"),
        "expected unknown tracked_python_file_count marker on threshold-order path:\n{order_text}"
    );
}
