use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

const POLICY_SCHEMA_VERSION: &str =
    "kamn.runtime.dependency-local-heavy-deep-scan-policy-report.v1";
const POLICY_REASON_TAXONOMY_VERSION: &str =
    "kamn.runtime.dependency-local-heavy-deep-scan-policy-reason-taxonomy.v1";
const POLICY_REASON_CODES_CSV: &str = "dependency_local_heavy_deep_scan_policy_required_field_missing,dependency_local_heavy_deep_scan_policy_marker_mismatch,dependency_local_heavy_deep_scan_policy_reason_taxonomy_mismatch,dependency_local_heavy_deep_scan_policy_profile_contract_mismatch,dependency_local_heavy_deep_scan_policy_docs_marker_parity_mismatch,dependency_local_heavy_deep_scan_policy_ci_dry_run_selector_drift,dependency_local_heavy_deep_scan_policy_ci_dry_run_workflow_drift,ci_fast_gate_failed,dependency_local_heavy_deep_scan_policy_expected_decision_mismatch,dependency_local_heavy_deep_scan_policy_violation";

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

fn repo_path(relative_path: &str) -> PathBuf {
    repo_root().join(relative_path)
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

fn run_lane(profile: &str, output_file: &Path) -> Output {
    run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg("scripts/runtime/run_dependency_local_heavy_deep_scan_lane.sh")
                .arg("--profile")
                .arg(profile)
                .arg("--mode")
                .arg("dry-run")
                .arg("--ci-fast-gate")
                .arg("PASS")
                .arg("--max-seconds")
                .arg("180")
                .arg("--output-json")
                .arg(output_file);
            command
        },
        "dependency local-heavy deep scan lane",
    )
}

fn run_policy(
    report_file: &Path,
    output_json: &Path,
    strategy_doc: Option<&Path>,
    ops_doc: Option<&Path>,
    ci_tools_file: Option<&Path>,
    workflow_file: Option<&Path>,
) -> Output {
    run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg("scripts/runtime/check_dependency_local_heavy_deep_scan_policy.sh")
                .arg("--report-file")
                .arg(report_file)
                .arg("--expected-final-decision")
                .arg("GO")
                .arg("--ci-fast-gate")
                .arg("PASS")
                .arg("--output-json")
                .arg(output_json);
            if let Some(path) = strategy_doc {
                command.arg("--strategy-doc").arg(path);
            }
            if let Some(path) = ops_doc {
                command.arg("--ops-doc").arg(path);
            }
            if let Some(path) = ci_tools_file {
                command.arg("--ci-tools-file").arg(path);
            }
            if let Some(path) = workflow_file {
                command.arg("--workflow-file").arg(path);
            }
            command
        },
        "dependency local-heavy deep scan policy checker",
    )
}

#[test]
fn unit_dependency_local_heavy_deep_scan_policy_checker_accepts_valid_baseline_report() {
    let tmp = TempDir::new("dependency-deep-scan-policy-unit");
    let lane_report = tmp.path().join("lane-report.json");
    let policy_report = tmp.path().join("policy-report.json");

    let lane_output = run_lane("baseline", &lane_report);
    assert_success(&lane_output, "generate baseline deep-scan lane report");

    let policy_output = run_policy(&lane_report, &policy_report, None, None, None, None);
    assert_success(&policy_output, "deep-scan policy checker baseline");

    let text = output_text(&policy_output);
    assert!(text.contains("status=pass"));
    assert!(text.contains("final_decision=GO"));
    assert!(text.contains("dependency_local_heavy_deep_scan_policy_status=verified"));
    assert!(
        text.contains("dependency_local_heavy_deep_scan_policy_docs_marker_parity_status=verified")
    );
    assert!(text
        .contains("dependency_local_heavy_deep_scan_policy_ci_dry_run_selector_status=verified"));
    assert!(text.contains("reason_codes_value=none"));

    let payload = load_text(&policy_report);
    assert_eq!(
        json_string_field(&payload, "schema_version").as_deref(),
        Some(POLICY_SCHEMA_VERSION)
    );
    assert_eq!(
        json_string_field(&payload, "reason_taxonomy_version").as_deref(),
        Some(POLICY_REASON_TAXONOMY_VERSION)
    );
    assert_eq!(
        json_string_field(&payload, "reason_codes_csv").as_deref(),
        Some(POLICY_REASON_CODES_CSV)
    );
}

#[test]
fn functional_dependency_local_heavy_deep_scan_policy_checker_rejects_tampered_profile_marker() {
    let tmp = TempDir::new("dependency-deep-scan-policy-functional");
    let lane_report = tmp.path().join("lane-report-functional.json");
    let policy_report = tmp.path().join("policy-report-functional.json");
    let tampered_report = tmp.path().join("lane-report-functional-tampered.json");

    let lane_output = run_lane("baseline", &lane_report);
    assert_success(&lane_output, "generate baseline deep-scan lane report");

    let baseline_payload = load_text(&lane_report);
    let tampered_payload = baseline_payload.replacen(
        "\"profile_status\": \"verified\"",
        "\"profile_status\": \"failed\"",
        1,
    );
    assert_ne!(
        baseline_payload, tampered_payload,
        "tampered payload should mutate profile marker"
    );
    write_text(&tampered_report, &tampered_payload);

    let policy_output = run_policy(&tampered_report, &policy_report, None, None, None, None);
    assert_failure(
        &policy_output,
        "deep-scan policy checker with tampered profile marker",
    );
    assert!(
        output_text(&policy_output)
            .contains("dependency_local_heavy_deep_scan_policy_profile_contract_mismatch"),
        "tampered profile marker must fail closed with deterministic mismatch reason"
    );
}

#[test]
fn integration_dependency_local_heavy_deep_scan_policy_checker_enforces_ci_dry_run_governance() {
    let tmp = TempDir::new("dependency-deep-scan-policy-integration");
    let lane_report = tmp.path().join("lane-report-integration.json");
    let policy_report = tmp.path().join("policy-report-integration.json");

    let lane_output = run_lane("baseline", &lane_report);
    assert_success(&lane_output, "generate baseline deep-scan lane report");

    let policy_output = run_policy(&lane_report, &policy_report, None, None, None, None);
    assert_success(
        &policy_output,
        "deep-scan policy checker integration baseline",
    );
    let text = output_text(&policy_output);
    assert!(text
        .contains("dependency_local_heavy_deep_scan_policy_ci_dry_run_selector_status=verified"));
    assert!(text
        .contains("dependency_local_heavy_deep_scan_policy_ci_dry_run_workflow_status=verified"));
    assert!(text.contains("promotion_decision_reason_mapping_status=verified"));
}

#[test]
fn regression_dependency_local_heavy_deep_scan_policy_checker_rejects_strategy_parity_drift() {
    let tmp = TempDir::new("dependency-deep-scan-policy-regression");
    let lane_report = tmp.path().join("lane-report-regression.json");
    let policy_report = tmp.path().join("policy-report-regression.json");
    let drifted_strategy = tmp.path().join("strategy-drifted.md");

    let lane_output = run_lane("baseline", &lane_report);
    assert_success(&lane_output, "generate baseline deep-scan lane report");

    let strategy_text = load_text(&repo_path("docs/ci/strategy.md"));
    let drifted_text = strategy_text.replacen(
        "dependency_local_heavy_deep_scan_policy_reason_taxonomy_version=kamn.runtime.dependency-local-heavy-deep-scan-policy-reason-taxonomy.v1",
        "dependency_local_heavy_deep_scan_policy_reason_taxonomy_version=drifted",
        1,
    );
    assert_ne!(
        strategy_text, drifted_text,
        "strategy fixture should mutate taxonomy marker"
    );
    write_text(&drifted_strategy, &drifted_text);

    let policy_output = run_policy(
        &lane_report,
        &policy_report,
        Some(&drifted_strategy),
        None,
        None,
        None,
    );
    assert_failure(
        &policy_output,
        "deep-scan policy checker with strategy docs parity drift",
    );
    assert!(
        output_text(&policy_output)
            .contains("dependency_local_heavy_deep_scan_policy_docs_marker_parity_mismatch"),
        "strategy parity drift must fail closed with deterministic docs mismatch reason"
    );
}

#[test]
fn performance_dependency_local_heavy_deep_scan_policy_checker_stays_within_budget() {
    let tmp = TempDir::new("dependency-deep-scan-policy-performance");
    let lane_report = tmp.path().join("lane-report-performance.json");
    let policy_report = tmp.path().join("policy-report-performance.json");

    let lane_output = run_lane("baseline", &lane_report);
    assert_success(&lane_output, "generate baseline deep-scan lane report");

    let started = Instant::now();
    let policy_output = run_policy(&lane_report, &policy_report, None, None, None, None);
    assert_success(
        &policy_output,
        "deep-scan policy checker performance baseline",
    );
    let elapsed_millis = started.elapsed().as_millis();
    assert!(
        elapsed_millis < 5000,
        "deep-scan policy checker exceeded 5s budget: {elapsed_millis}ms"
    );
}
