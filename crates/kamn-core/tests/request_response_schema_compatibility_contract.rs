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
fn unit_request_response_schema_compatibility_lane_dry_run_emits_deterministic_markers() {
    let tmp = TempDir::new("schema-compatibility-unit");
    let report_file = tmp.path().join("schema-compatibility-summary.json");

    let output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg("scripts/runtime/validate_request_response_schema_compatibility_live.sh")
                .arg("--mode")
                .arg("dry-run")
                .arg("--output-json")
                .arg(&report_file);
            command
        },
        "request-response schema compatibility dry-run lane",
    );
    assert_success(
        &output,
        "request-response schema compatibility dry-run lane",
    );

    let text = output_text(&output);
    assert!(text.contains("status=pass"));
    assert!(text.contains("final_decision=GO"));
    assert!(text.contains("lane_mode=dry-run"));
    assert!(text.contains("compatible_pairs_status=verified"));
    assert!(text.contains("incompatible_pairs_status=verified"));
    assert!(text.contains("execution_reason_code=dry_run_no_commands_executed"));
    assert!(text.contains("command_count=0"));

    let payload = load_text(&report_file);
    assert_eq!(
        json_string_field(&payload, "schema_version").as_deref(),
        Some("kamn.runtime.request-response-schema-compatibility-report.v1")
    );
    assert_eq!(
        json_string_field(&payload, "fixture_schema_version").as_deref(),
        Some("kamn.runtime.request-response-schema-compatibility-fixture-matrix.v1")
    );
    assert_eq!(
        json_string_field(&payload, "lane_mode").as_deref(),
        Some("dry-run")
    );
    assert_eq!(json_u64_field(&payload, "fixture_row_count"), Some(4));
    assert_eq!(json_u64_field(&payload, "compatible_pair_count"), Some(2));
    assert_eq!(json_u64_field(&payload, "incompatible_pair_count"), Some(2));
}

#[test]
fn functional_request_response_schema_compatibility_lane_includes_compatible_and_incompatible_fixture_rows(
) {
    let tmp = TempDir::new("schema-compatibility-functional-rows");
    let report_file = tmp.path().join("schema-compatibility-summary.json");

    let lane_output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg("scripts/runtime/validate_request_response_schema_compatibility_live.sh")
                .arg("--mode")
                .arg("dry-run")
                .arg("--output-json")
                .arg(&report_file);
            command
        },
        "request-response schema compatibility functional rows lane",
    );
    assert_success(
        &lane_output,
        "request-response schema compatibility functional rows lane",
    );

    let payload = load_text(&report_file);
    assert!(payload.contains("\"row_id\": \"v1_to_v2_messages_send_optional_request_addition\""));
    assert!(payload.contains("\"row_id\": \"v1_to_v2_channels_create_optional_response_addition\""));
    assert!(payload.contains("\"row_id\": \"v1_to_v2_messages_get_required_response_removal\""));
    assert!(payload.contains("\"row_id\": \"v1_to_v2_tasks_create_required_request_removal\""));

    let compatible = payload
        .matches("\"schema_compatibility_status\": \"compatible\"")
        .count();
    let incompatible = payload
        .matches("\"schema_compatibility_status\": \"incompatible\"")
        .count();
    assert_eq!(compatible, 2, "expected two compatible pair rows");
    assert_eq!(incompatible, 2, "expected two incompatible pair rows");
    assert!(payload.contains("\"observed_final_decision\": \"NO-GO\""));
    assert!(payload.contains("\"observed_reason_code\": \"schema_pair_breaking_change_detected\""));
}

#[test]
fn functional_request_response_schema_compatibility_checker_accepts_valid_report() {
    let tmp = TempDir::new("schema-compatibility-policy-pass");
    let report_file = tmp.path().join("schema-compatibility-summary.json");
    let policy_file = tmp.path().join("schema-compatibility-policy.json");

    let lane_output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg("scripts/runtime/validate_request_response_schema_compatibility_live.sh")
                .arg("--mode")
                .arg("dry-run")
                .arg("--output-json")
                .arg(&report_file);
            command
        },
        "request-response schema compatibility policy pass lane",
    );
    assert_success(
        &lane_output,
        "request-response schema compatibility policy pass lane",
    );

    let policy_output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg("scripts/runtime/check_request_response_schema_compatibility_live_policy.sh")
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
        "request-response schema compatibility policy pass checker",
    );
    assert_success(
        &policy_output,
        "request-response schema compatibility policy pass checker",
    );

    let text = output_text(&policy_output);
    assert!(text.contains("status=ok"));
    assert!(text.contains("final_decision=GO"));
    assert!(text.contains("request_response_schema_compatibility_policy_status=verified"));

    let payload = load_text(&policy_file);
    assert_eq!(
        json_string_field(&payload, "schema_version").as_deref(),
        Some("kamn.runtime.request-response-schema-compatibility-policy-report.v1")
    );
    assert_eq!(
        json_string_field(&payload, "final_decision").as_deref(),
        Some("GO")
    );
}

#[test]
fn regression_request_response_schema_compatibility_checker_rejects_tampered_marker() {
    let tmp = TempDir::new("schema-compatibility-policy-fail");
    let report_file = tmp.path().join("schema-compatibility-summary.json");
    let tampered_file = tmp
        .path()
        .join("schema-compatibility-summary-tampered.json");
    let policy_file = tmp.path().join("schema-compatibility-policy-tampered.json");

    let lane_output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg("scripts/runtime/validate_request_response_schema_compatibility_live.sh")
                .arg("--mode")
                .arg("dry-run")
                .arg("--output-json")
                .arg(&report_file);
            command
        },
        "request-response schema compatibility tamper setup lane",
    );
    assert_success(
        &lane_output,
        "request-response schema compatibility tamper setup lane",
    );

    let payload = load_text(&report_file);
    let tampered_payload = payload.replacen(
        "\"row_status\": \"verified\"",
        "\"row_status\": \"missing\"",
        1,
    );
    assert_ne!(
        tampered_payload, payload,
        "tamper mutation should alter at least one row_status field"
    );
    fs::write(&tampered_file, tampered_payload).expect("write tampered payload");

    let policy_output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg("scripts/runtime/check_request_response_schema_compatibility_live_policy.sh")
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
        "request-response schema compatibility tampered policy checker",
    );
    assert_failure(
        &policy_output,
        "request-response schema compatibility tampered policy checker",
    );
    assert!(
        output_text(&policy_output)
            .contains("request_response_schema_compatibility_fixture_row_status_mismatch"),
        "tampered report must emit deterministic fixture-row status mismatch reason code"
    );
}

#[test]
fn integration_request_response_schema_compatibility_contract_lane_composes_policy_and_docs_parity()
{
    let tmp = TempDir::new("schema-compatibility-contract-integration");
    let lane_report = tmp.path().join("schema-compatibility-contract-lane.json");
    let policy_report = tmp.path().join("schema-compatibility-contract-policy.json");

    let output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg("scripts/runtime/validate_request_response_schema_compatibility_live_contract_lane.sh")
                .arg("--output-json")
                .arg(&lane_report)
                .arg("--policy-output-json")
                .arg(&policy_report);
            command
        },
        "request-response schema compatibility contract lane integration",
    );
    assert_success(
        &output,
        "request-response schema compatibility contract lane integration",
    );

    let text = output_text(&output);
    assert!(text.contains("status=pass"));
    assert!(text.contains("final_decision=GO"));
    assert!(text.contains("request_response_schema_compatibility_contract_status=verified"));
    assert!(text.contains("request_response_schema_compatibility_policy_status=verified"));
    assert!(text.contains("docs_contract_status=verified"));

    let lane_payload = load_text(&lane_report);
    assert_eq!(
        json_string_field(&lane_payload, "schema_version").as_deref(),
        Some("kamn.runtime.request-response-schema-compatibility-contract-lane-report.v1")
    );
    assert_eq!(
        json_string_field(
            &lane_payload,
            "request_response_schema_compatibility_contract_status"
        )
        .as_deref(),
        Some("verified")
    );

    let policy_payload = load_text(&policy_report);
    assert_eq!(
        json_string_field(&policy_payload, "schema_version").as_deref(),
        Some("kamn.runtime.request-response-schema-compatibility-policy-report.v1")
    );
    assert_eq!(
        json_string_field(&policy_payload, "final_decision").as_deref(),
        Some("GO")
    );
}

#[test]
fn performance_request_response_schema_compatibility_contract_lane_dry_run_stays_within_budget() {
    let tmp = TempDir::new("schema-compatibility-performance");
    let lane_report = tmp
        .path()
        .join("schema-compatibility-contract-lane-performance.json");
    let started = Instant::now();

    let output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg("scripts/runtime/validate_request_response_schema_compatibility_live_contract_lane.sh")
                .arg("--max-seconds")
                .arg("120")
                .arg("--output-json")
                .arg(&lane_report);
            command
        },
        "request-response schema compatibility contract lane performance",
    );
    assert_success(
        &output,
        "request-response schema compatibility contract lane performance",
    );

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
