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

fn run_generate(
    output_file: &Path,
    artifact_id: &str,
    lifecycle_stage: &str,
    profile: &str,
    record_count: &str,
    ci_fast_gate: &str,
) -> Output {
    run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg("scripts/runtime/generate_lifecycle_artifact_integrity_evidence_bundle.sh")
                .arg("--output-file")
                .arg(output_file)
                .arg("--artifact-id")
                .arg(artifact_id)
                .arg("--lifecycle-stage")
                .arg(lifecycle_stage)
                .arg("--profile")
                .arg(profile)
                .arg("--record-count")
                .arg(record_count)
                .arg("--ci-fast-gate")
                .arg(ci_fast_gate);
            command
        },
        "lifecycle artifact integrity generator",
    )
}

fn run_check(bundle_file: &Path, expected_final_decision: &str) -> Output {
    run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg("scripts/runtime/check_lifecycle_artifact_integrity_evidence_bundle.sh")
                .arg("--bundle-file")
                .arg(bundle_file)
                .arg("--expected-final-decision")
                .arg(expected_final_decision);
            command
        },
        "lifecycle artifact integrity checker",
    )
}

#[test]
fn unit_lifecycle_artifact_generator_emits_deterministic_integrity_markers() {
    let tmp = TempDir::new("lifecycle-artifact-unit");
    let first_bundle = tmp.path().join("lifecycle-artifact-first.json");
    let second_bundle = tmp.path().join("lifecycle-artifact-second.json");

    let first = run_generate(
        &first_bundle,
        "artifact-baseline",
        "retention",
        "baseline",
        "42",
        "PASS",
    );
    assert_success(&first, "lifecycle artifact integrity generate first");

    let second = run_generate(
        &second_bundle,
        "artifact-baseline",
        "retention",
        "baseline",
        "42",
        "PASS",
    );
    assert_success(&second, "lifecycle artifact integrity generate second");

    let first_payload = load_text(&first_bundle);
    let second_payload = load_text(&second_bundle);
    assert_eq!(
        json_string_field(&first_payload, "schema_version").as_deref(),
        Some("kamn.runtime.lifecycle-artifact-integrity-evidence.v1")
    );
    assert_eq!(
        json_string_field(&first_payload, "artifact_schema_version").as_deref(),
        Some("kamn.runtime.lifecycle-artifact-integrity-schema.v1")
    );
    assert_eq!(
        json_string_field(&first_payload, "reason_taxonomy_version").as_deref(),
        Some("kamn.runtime.lifecycle-artifact-integrity-reason-taxonomy.v1")
    );
    assert_eq!(
        json_string_field(&first_payload, "reason_codes_csv").as_deref(),
        Some("lifecycle_artifact_required_field_missing,lifecycle_artifact_marker_mismatch,lifecycle_artifact_hash_mismatch,lifecycle_artifact_reason_taxonomy_mismatch,lifecycle_artifact_reason_codes_csv_mismatch,lifecycle_artifact_expected_decision_mismatch")
    );
    let first_payload_hash = json_string_field(&first_payload, "payload_hash_sha256")
        .expect("missing payload hash marker");
    let first_integrity_hash = json_string_field(&first_payload, "integrity_hash_sha256")
        .expect("missing integrity hash marker");
    let first_provenance_hash = json_string_field(&first_payload, "provenance_hash_sha256")
        .expect("missing provenance hash marker");
    assert!(first_payload_hash.starts_with("sha256:"));
    assert!(first_integrity_hash.starts_with("sha256:"));
    assert!(first_provenance_hash.starts_with("sha256:"));

    let second_payload_hash = json_string_field(&second_payload, "payload_hash_sha256")
        .expect("missing second payload hash marker");
    let second_integrity_hash = json_string_field(&second_payload, "integrity_hash_sha256")
        .expect("missing second integrity hash marker");
    let second_provenance_hash = json_string_field(&second_payload, "provenance_hash_sha256")
        .expect("missing second provenance hash marker");
    assert_eq!(first_payload_hash, second_payload_hash);
    assert_eq!(first_integrity_hash, second_integrity_hash);
    assert_eq!(first_provenance_hash, second_provenance_hash);
}

#[test]
fn functional_lifecycle_artifact_checker_rejects_tampered_payload() {
    let tmp = TempDir::new("lifecycle-artifact-functional");
    let bundle_file = tmp.path().join("lifecycle-artifact-functional.json");

    let generated = run_generate(
        &bundle_file,
        "artifact-functional",
        "retention",
        "baseline",
        "42",
        "PASS",
    );
    assert_success(&generated, "lifecycle artifact generate functional");

    let tampered = load_text(&bundle_file).replace("\"record_count\": 42", "\"record_count\": 43");
    fs::write(&bundle_file, tampered).expect("failed to write tampered lifecycle artifact bundle");

    let checked = run_check(&bundle_file, "GO");
    assert_failure(&checked, "lifecycle artifact checker tampered payload");
    assert!(
        output_text(&checked).contains("lifecycle_artifact_hash_mismatch"),
        "tampered payload must fail closed with deterministic hash mismatch marker"
    );
}

#[test]
fn integration_lifecycle_artifact_generator_and_checker_roundtrip() {
    let tmp = TempDir::new("lifecycle-artifact-integration");
    let bundle_file = tmp.path().join("lifecycle-artifact-integration.json");

    let generated = run_generate(
        &bundle_file,
        "artifact-integration",
        "deletion",
        "baseline",
        "9",
        "PASS",
    );
    assert_success(&generated, "lifecycle artifact generate integration");

    let checked = run_check(&bundle_file, "GO");
    assert_success(&checked, "lifecycle artifact checker integration");
    let output = output_text(&checked);
    assert!(output.contains("status=ok"));
    assert!(output.contains("final_decision=GO"));
}

#[test]
fn regression_lifecycle_artifact_checker_rejects_reason_taxonomy_drift() {
    let tmp = TempDir::new("lifecycle-artifact-regression");
    let bundle_file = tmp.path().join("lifecycle-artifact-regression.json");

    let generated = run_generate(
        &bundle_file,
        "artifact-regression",
        "retention",
        "baseline",
        "17",
        "PASS",
    );
    assert_success(&generated, "lifecycle artifact generate regression");

    let drifted = load_text(&bundle_file).replace(
        "kamn.runtime.lifecycle-artifact-integrity-reason-taxonomy.v1",
        "kamn.runtime.lifecycle-artifact-integrity-reason-taxonomy.v0",
    );
    fs::write(&bundle_file, drifted)
        .expect("failed to write reason taxonomy drift lifecycle artifact bundle");

    let checked = run_check(&bundle_file, "GO");
    assert_failure(&checked, "lifecycle artifact checker reason taxonomy drift");
    assert!(
        output_text(&checked).contains("lifecycle_artifact_reason_taxonomy_mismatch"),
        "reason taxonomy drift must fail closed with deterministic marker"
    );
}

#[test]
fn performance_lifecycle_artifact_generator_checker_stays_within_budget() {
    let tmp = TempDir::new("lifecycle-artifact-performance");
    let bundle_file = tmp.path().join("lifecycle-artifact-performance.json");

    let started = Instant::now();
    let generated = run_generate(
        &bundle_file,
        "artifact-performance",
        "retention",
        "baseline",
        "21",
        "PASS",
    );
    assert_success(&generated, "lifecycle artifact generate performance");

    let checked = run_check(&bundle_file, "GO");
    assert_success(&checked, "lifecycle artifact checker performance");

    let elapsed_millis = started.elapsed().as_millis();
    assert!(
        elapsed_millis < 5000,
        "lifecycle artifact generate+check exceeded 5s budget: {elapsed_millis}ms"
    );
}
