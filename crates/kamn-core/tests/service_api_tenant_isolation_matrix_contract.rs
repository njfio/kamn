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
        .unwrap_or_else(|error| panic!("failed to read json file {}: {error}", path.display()))
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

fn json_u64_field(json_text: &str, field: &str) -> Option<u64> {
    let value = extract_json_field_value(json_text, field)?;
    let digits: String = value
        .chars()
        .skip_while(|character| character.is_whitespace())
        .take_while(|character| character.is_ascii_digit())
        .collect();
    if digits.is_empty() {
        return None;
    }
    digits.parse::<u64>().ok()
}

#[test]
fn unit_tenant_isolation_matrix_lane_dry_run_emits_deterministic_schema_and_markers() {
    let tmp = TempDir::new("tenant-isolation-unit");
    let report_file = tmp.path().join("tenant-isolation-summary.json");

    let output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg("scripts/runtime/validate_service_api_tenant_isolation_matrix_live.sh")
                .arg("--mode")
                .arg("dry-run")
                .arg("--output-json")
                .arg(&report_file);
            command
        },
        "tenant isolation matrix dry-run lane",
    );
    assert_success(&output, "tenant isolation matrix dry-run lane");

    let text = output_text(&output);
    assert!(text.contains("status=pass"));
    assert!(text.contains("final_decision=GO"));
    assert!(text.contains("lane_mode=dry-run"));
    assert!(text.contains("tenant_isolation_status=verified"));
    assert!(text.contains("cross_tenant_leakage_status=verified"));
    assert!(text.contains("execution_reason_code=dry_run_no_commands_executed"));
    assert!(text.contains("command_count=0"));

    let payload = load_text(&report_file);
    assert_eq!(
        json_string_field(&payload, "schema_version").as_deref(),
        Some("kamn.runtime.service-api-tenant-isolation-matrix-live-report.v1")
    );
    assert_eq!(
        json_string_field(&payload, "matrix_schema_version").as_deref(),
        Some("kamn.runtime.service-api-tenant-isolation-matrix.v1")
    );
    assert_eq!(
        json_string_field(&payload, "lane_mode").as_deref(),
        Some("dry-run")
    );
    assert_eq!(json_u64_field(&payload, "leakage_attempt_count"), Some(4));
    assert_eq!(payload.matches("\"row_id\"").count(), 4);
}

#[test]
fn functional_tenant_isolation_matrix_lane_run_mode_requires_explicit_opt_in() {
    let tmp = TempDir::new("tenant-isolation-opt-in");
    let report_file = tmp.path().join("tenant-isolation-summary-run.json");

    let output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg("scripts/runtime/validate_service_api_tenant_isolation_matrix_live.sh")
                .arg("--mode")
                .arg("run")
                .arg("--output-json")
                .arg(&report_file);
            command
        },
        "tenant isolation matrix run-mode opt-in guard",
    );
    assert_failure(&output, "tenant isolation matrix run-mode opt-in guard");
    assert!(
        output_text(&output).contains(
            "run mode requires explicit local-only opt-in via KAMN_SERVICE_API_TENANT_ISOLATION_MATRIX_OPT_IN=1"
        ),
        "run-mode rejection must include deterministic opt-in guard message"
    );
}

#[test]
fn functional_tenant_isolation_matrix_policy_accepts_deterministic_report() {
    let tmp = TempDir::new("tenant-isolation-policy-pass");
    let report_file = tmp.path().join("tenant-isolation-summary.json");
    let policy_file = tmp.path().join("tenant-isolation-policy.json");

    let lane_output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg("scripts/runtime/validate_service_api_tenant_isolation_matrix_live.sh")
                .arg("--mode")
                .arg("dry-run")
                .arg("--output-json")
                .arg(&report_file);
            command
        },
        "tenant isolation matrix policy pass lane",
    );
    assert_success(&lane_output, "tenant isolation matrix policy pass lane");

    let policy_output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg("scripts/runtime/check_service_api_tenant_isolation_matrix_live_policy.sh")
                .arg("--report-file")
                .arg(&report_file)
                .arg("--expected-final-decision")
                .arg("GO")
                .arg("--ci-fast-gate")
                .arg("PASS")
                .arg("--output-json")
                .arg(&policy_file);
            command
        },
        "tenant isolation matrix policy pass checker",
    );
    assert_success(
        &policy_output,
        "tenant isolation matrix policy pass checker",
    );

    let text = output_text(&policy_output);
    assert!(text.contains("status=ok"));
    assert!(text.contains("final_decision=GO"));
    assert!(text.contains("service_api_tenant_isolation_matrix_policy_status=verified"));

    let payload = load_text(&policy_file);
    assert_eq!(
        json_string_field(&payload, "schema_version").as_deref(),
        Some("kamn.runtime.service-api-tenant-isolation-matrix-live-policy-report.v1")
    );
    assert_eq!(
        json_string_field(&payload, "final_decision").as_deref(),
        Some("GO")
    );
}

#[test]
fn regression_tenant_isolation_matrix_policy_rejects_tampered_leakage_marker() {
    let tmp = TempDir::new("tenant-isolation-policy-fail");
    let report_file = tmp.path().join("tenant-isolation-summary.json");
    let tampered_file = tmp.path().join("tenant-isolation-summary-tampered.json");
    let policy_file = tmp.path().join("tenant-isolation-policy-tampered.json");

    let lane_output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg("scripts/runtime/validate_service_api_tenant_isolation_matrix_live.sh")
                .arg("--mode")
                .arg("dry-run")
                .arg("--output-json")
                .arg(&report_file);
            command
        },
        "tenant isolation matrix tamper setup lane",
    );
    assert_success(&lane_output, "tenant isolation matrix tamper setup lane");

    let payload = load_text(&report_file);
    let tampered_payload = payload.replacen(
        "\"scenario_status\": \"verified\"",
        "\"scenario_status\": \"missing\"",
        1,
    );
    assert_ne!(
        tampered_payload, payload,
        "tamper mutation should alter at least one scenario_status field"
    );
    fs::write(&tampered_file, tampered_payload).expect("write tampered payload");

    let policy_output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg("scripts/runtime/check_service_api_tenant_isolation_matrix_live_policy.sh")
                .arg("--report-file")
                .arg(&tampered_file)
                .arg("--expected-final-decision")
                .arg("GO")
                .arg("--ci-fast-gate")
                .arg("PASS")
                .arg("--output-json")
                .arg(&policy_file);
            command
        },
        "tenant isolation matrix tampered policy checker",
    );
    assert_failure(
        &policy_output,
        "tenant isolation matrix tampered policy checker",
    );
    assert!(
        output_text(&policy_output)
            .contains("service_api_tenant_isolation_policy_matrix_row_status_mismatch"),
        "tampered report must emit deterministic matrix-row status mismatch reason code"
    );
}

#[test]
fn integration_tenant_isolation_matrix_contract_lane_composes_lane_policy_and_docs_parity() {
    let tmp = TempDir::new("tenant-isolation-contract-integration");
    let lane_report = tmp.path().join("tenant-isolation-contract-lane.json");
    let policy_report = tmp.path().join("tenant-isolation-contract-policy.json");

    let output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg("scripts/runtime/validate_service_api_tenant_isolation_matrix_live_contract_lane.sh")
                .arg("--output-json")
                .arg(&lane_report)
                .arg("--policy-output-json")
                .arg(&policy_report);
            command
        },
        "tenant isolation matrix contract lane integration",
    );
    assert_success(&output, "tenant isolation matrix contract lane integration");

    let text = output_text(&output);
    assert!(text.contains("status=pass"));
    assert!(text.contains("final_decision=GO"));
    assert!(text.contains("service_api_tenant_isolation_matrix_contract_status=verified"));
    assert!(text.contains("service_api_tenant_isolation_matrix_policy_status=verified"));
    assert!(text.contains("docs_contract_status=verified"));

    let lane_payload = load_text(&lane_report);
    assert_eq!(
        json_string_field(&lane_payload, "schema_version").as_deref(),
        Some("kamn.runtime.service-api-tenant-isolation-matrix-live-contract-lane-report.v1")
    );
    assert_eq!(
        json_string_field(
            &lane_payload,
            "service_api_tenant_isolation_matrix_contract_status"
        )
        .as_deref(),
        Some("verified")
    );

    let policy_payload = load_text(&policy_report);
    assert_eq!(
        json_string_field(&policy_payload, "schema_version").as_deref(),
        Some("kamn.runtime.service-api-tenant-isolation-matrix-live-policy-report.v1")
    );
    assert_eq!(
        json_string_field(&policy_payload, "final_decision").as_deref(),
        Some("GO")
    );
}

#[test]
fn performance_tenant_isolation_matrix_contract_lane_dry_run_stays_within_budget() {
    let tmp = TempDir::new("tenant-isolation-performance");
    let lane_report = tmp
        .path()
        .join("tenant-isolation-contract-lane-performance.json");
    let started = Instant::now();

    let output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg("scripts/runtime/validate_service_api_tenant_isolation_matrix_live_contract_lane.sh")
                .arg("--max-seconds")
                .arg("120")
                .arg("--output-json")
                .arg(&lane_report);
            command
        },
        "tenant isolation matrix contract lane performance",
    );
    assert_success(&output, "tenant isolation matrix contract lane performance");

    let elapsed = started.elapsed().as_secs();
    assert!(
        elapsed <= 30,
        "dry-run contract lane should remain cost-effective (observed {elapsed}s)"
    );

    let payload = load_text(&lane_report);
    let reported_elapsed =
        json_u64_field(&payload, "elapsed_seconds").expect("elapsed_seconds must be present");
    assert!(
        reported_elapsed <= 120,
        "reported elapsed_seconds should stay within configured budget"
    );
}
