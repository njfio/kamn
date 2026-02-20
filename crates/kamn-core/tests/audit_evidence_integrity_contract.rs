use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

const AUDIT_INTEGRITY_REASON_TAXONOMY_VERSION: &str =
    "kamn.release.gonogo-audit-integrity-convergence-reason-taxonomy.v1";
const AUDIT_INTEGRITY_REASON_CODES_CSV: &str = "gonogo_audit_integrity_file_missing,gonogo_audit_integrity_invalid_json,gonogo_audit_integrity_schema_mismatch,gonogo_audit_integrity_status_not_ok,gonogo_audit_integrity_final_decision_not_go,gonogo_audit_integrity_policy_status_not_verified,gonogo_audit_integrity_reason_taxonomy_version_mismatch,gonogo_audit_integrity_reason_codes_csv_mismatch,gonogo_audit_integrity_freshness_window_exceeded";

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

fn write_audit_integrity_report(path: &Path) {
    fs::write(
        path,
        r#"{
  "schema_version": "kamn.runtime.sqlite-crash-recovery-live-policy-report.v1",
  "status": "ok",
  "final_decision": "GO",
  "sqlite_crash_recovery_policy_status": "verified",
  "durability_governance_reason_taxonomy_version": "kamn.runtime.durability-governance-reason-taxonomy.v1",
  "durability_governance_reason_codes_csv": "crash_recovery_promotion_stalled,audit_trail_parity_mismatch,ci_local_promotion_budget_boundary_exceeded"
}
"#,
    )
    .expect("failed to write audit-integrity report fixture");
}

fn generate_bundle(bundle_file: &Path, audit_report_file: &Path, release_suffix: &str) -> Output {
    run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg("scripts/deploy/generate_gonogo_evidence_bundle.sh")
                .arg("--output-file")
                .arg(bundle_file)
                .arg("--release-candidate")
                .arg(format!("v1.0.0-rc-{release_suffix}"))
                .arg("--schema-target-version")
                .arg("1.0.0")
                .arg("--runtime-image-digest")
                .arg(format!("sha256:audit-integrity-{release_suffix}"))
                .arg("--ci-fast-gate")
                .arg("PASS")
                .arg("--ci-deep-lane")
                .arg("PASS")
                .arg("--rollback-precheck")
                .arg("PASS")
                .arg("--rollback-trigger-status")
                .arg("CLEAR")
                .arg("--required-approvals")
                .arg("2")
                .arg("--received-approvals")
                .arg("2")
                .arg("--audit-integrity-report-file")
                .arg(audit_report_file)
                .arg("--audit-integrity-max-age-seconds")
                .arg("1800");
            command
        },
        "generate go/no-go bundle with audit-integrity gate",
    )
}

fn run_policy_checker(bundle_file: &Path) -> Output {
    run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg("scripts/deploy/check_gonogo_evidence_policy.sh")
                .arg("--bundle-file")
                .arg(bundle_file);
            command
        },
        "check go/no-go bundle policy",
    )
}

#[test]
fn spec_c01_audit_integrity_generate_bundle_emits_deterministic_go_markers() {
    let tmp = TempDir::new("audit-integrity-c01");
    let bundle_file = tmp.path().join("gonogo-audit-integrity-go.json");
    let audit_report_file = tmp.path().join("audit-integrity-policy-report.json");
    write_audit_integrity_report(&audit_report_file);

    let output = generate_bundle(&bundle_file, &audit_report_file, "4059-c01");
    assert_success(
        &output,
        "generate go/no-go bundle with audit-integrity gate",
    );

    let output_text = output_text(&output);
    assert!(output_text.contains("status=generated"));
    assert!(output_text.contains("final_decision=GO"));
    assert!(output_text.contains("audit_integrity_gate_final_decision=GO"));
    assert!(output_text.contains(&format!(
        "audit_integrity_reason_taxonomy_version={AUDIT_INTEGRITY_REASON_TAXONOMY_VERSION}"
    )));
    assert!(output_text.contains("audit_integrity_reason_codes_csv=none"));

    let bundle_payload = fs::read_to_string(&bundle_file).expect("failed to read generated bundle");
    assert!(
        bundle_payload.contains("\"schema_version\": \"kamn.release.gonogo.v1\""),
        "generated bundle must include top-level go/no-go schema marker"
    );
    assert!(
        bundle_payload
            .contains("\"schema_version\": \"kamn.release.gonogo-audit-integrity-gate.v1\""),
        "generated bundle must include audit-integrity gate schema marker"
    );
    assert_eq!(
        json_string_field(&bundle_payload, "reason_taxonomy_version").as_deref(),
        Some(AUDIT_INTEGRITY_REASON_TAXONOMY_VERSION)
    );
    assert_eq!(
        json_string_field(&bundle_payload, "final_decision").as_deref(),
        Some("GO")
    );
}

#[test]
fn spec_c02_audit_integrity_policy_checker_accepts_converged_bundle() {
    let tmp = TempDir::new("audit-integrity-c02");
    let bundle_file = tmp.path().join("gonogo-audit-integrity-go.json");
    let audit_report_file = tmp.path().join("audit-integrity-policy-report.json");
    write_audit_integrity_report(&audit_report_file);

    let generate_output = generate_bundle(&bundle_file, &audit_report_file, "4059-c02");
    assert_success(
        &generate_output,
        "generate go/no-go bundle for converged audit-integrity checker path",
    );

    let checker_output = run_policy_checker(&bundle_file);
    assert_success(
        &checker_output,
        "policy checker for converged audit-integrity bundle",
    );
    let checker_text = output_text(&checker_output);
    assert!(checker_text.contains("status=ok"));
    assert!(checker_text.contains("final_decision=GO"));
    assert!(checker_text.contains("audit_integrity_gate_final_decision=GO"));
    assert!(checker_text.contains("required_approvals=2"));
    assert!(checker_text.contains("received_approvals=2"));
}

#[test]
fn regression_spec_c03_audit_integrity_policy_checker_rejects_tampered_gate_payload() {
    let tmp = TempDir::new("audit-integrity-c03");
    let bundle_file = tmp.path().join("gonogo-audit-integrity-go.json");
    let audit_report_file = tmp.path().join("audit-integrity-policy-report.json");
    let tampered_bundle_file = tmp.path().join("gonogo-audit-integrity-tampered.json");
    write_audit_integrity_report(&audit_report_file);

    let generate_output = generate_bundle(&bundle_file, &audit_report_file, "4059-c03");
    assert_success(
        &generate_output,
        "generate go/no-go bundle for tamper regression setup",
    );

    let payload = fs::read_to_string(&bundle_file).expect("failed to read generated bundle");
    let tampered_payload = payload.replacen(
        "\"audit_integrity_report_status\": \"ok\"",
        "\"audit_integrity_report_status\": \"fail\"",
        1,
    );
    assert_ne!(
        payload, tampered_payload,
        "tamper mutation should alter audit-integrity report status marker"
    );
    fs::write(&tampered_bundle_file, tampered_payload).expect("write tampered bundle");

    let checker_output = run_policy_checker(&tampered_bundle_file);
    assert_failure(
        &checker_output,
        "policy checker for tampered audit-integrity bundle",
    );
    assert!(
        output_text(&checker_output).contains("audit integrity gate convergence mismatch"),
        "tampered audit-integrity bundle must fail closed with deterministic convergence mismatch message"
    );
}

#[test]
fn performance_spec_c05_audit_integrity_generate_and_check_dry_run_stays_within_budget() {
    let tmp = TempDir::new("audit-integrity-c05");
    let bundle_file = tmp.path().join("gonogo-audit-integrity-go.json");
    let audit_report_file = tmp.path().join("audit-integrity-policy-report.json");
    write_audit_integrity_report(&audit_report_file);

    let start = Instant::now();

    let generate_output = generate_bundle(&bundle_file, &audit_report_file, "4059-c05");
    assert_success(
        &generate_output,
        "generate go/no-go bundle for performance contract",
    );

    let checker_output = run_policy_checker(&bundle_file);
    assert_success(&checker_output, "policy checker for performance contract");

    let elapsed = start.elapsed().as_secs_f64();
    assert!(
        elapsed <= 20.0,
        "audit-integrity dry-run generate+check should stay within 20s CI budget (observed {elapsed:.3}s)"
    );

    let output = format!(
        "{}{}",
        output_text(&generate_output),
        output_text(&checker_output)
    );
    assert!(output.contains(&format!(
        "audit_integrity_reason_taxonomy_version={AUDIT_INTEGRITY_REASON_TAXONOMY_VERSION}"
    )));
    assert!(
        output.contains(&format!(
            "audit_integrity_reason_codes_csv={AUDIT_INTEGRITY_REASON_CODES_CSV}"
        )) || output.contains("audit_integrity_reason_codes_csv=none")
    );
}
