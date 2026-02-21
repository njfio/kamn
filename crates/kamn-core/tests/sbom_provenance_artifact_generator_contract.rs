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

fn run_generator(
    profile: &str,
    mode: &str,
    ci_fast_gate: &str,
    output_file: &Path,
    with_local_opt_in: bool,
) -> Output {
    run_command(
        {
            let mut command = Command::new("python3");
            command
                .arg("scripts/runtime/sbom_provenance_artifact_generator_contract.py")
                .arg("--profile")
                .arg(profile)
                .arg("--mode")
                .arg(mode)
                .arg("--ci-fast-gate")
                .arg(ci_fast_gate)
                .arg("--max-seconds")
                .arg("120")
                .arg("--output-json")
                .arg(output_file);
            if with_local_opt_in {
                command.env("KAMN_SBOM_PROVENANCE_GENERATOR_OPT_IN", "1");
            }
            command
        },
        "sbom provenance artifact generator",
    )
}

#[test]
fn unit_sbom_provenance_generator_baseline_emits_deterministic_schema_markers() {
    let tmp = TempDir::new("sbom-provenance-generator-unit");
    let report_file = tmp.path().join("sbom-provenance-baseline.json");

    let output = run_generator("baseline", "dry-run", "PASS", &report_file, false);
    assert_success(&output, "sbom provenance baseline dry-run");

    let text = output_text(&output);
    assert!(text.contains("status=pass"));
    assert!(text.contains("final_decision=GO"));
    assert!(text.contains("profile=baseline"));
    assert!(text.contains("reason_code=none"));
    assert!(text.contains("sbom_schema_version=spdx-2.3"));
    assert!(text.contains("provenance_schema_version=slsa-v1"));

    let payload = load_text(&report_file);
    assert_eq!(
        json_string_field(&payload, "schema_version").as_deref(),
        Some("kamn.runtime.sbom-provenance-artifact-report.v1")
    );
    assert_eq!(
        json_string_field(&payload, "artifact_schema_version").as_deref(),
        Some("kamn.runtime.sbom-provenance-artifact-schema.v1")
    );
    assert_eq!(
        json_string_field(&payload, "fixture_schema_version").as_deref(),
        Some("kamn.ci.sbom-provenance-artifact-fixture-matrix.v1")
    );
    assert_eq!(
        json_string_field(&payload, "reason_taxonomy_version").as_deref(),
        Some("kamn.runtime.sbom-provenance-artifact-reason-taxonomy.v1")
    );
}

#[test]
fn functional_sbom_provenance_generator_injected_drift_profile_fails_closed() {
    let tmp = TempDir::new("sbom-provenance-generator-functional");
    let report_file = tmp.path().join("sbom-provenance-injected-drift.json");

    let output = run_generator("injected-drift", "dry-run", "PASS", &report_file, false);
    assert_failure(&output, "sbom provenance injected-drift dry-run");

    let text = output_text(&output);
    assert!(text.contains("status=fail"));
    assert!(text.contains("final_decision=NO-GO"));
    assert!(text.contains("profile=injected-drift"));
    assert!(text.contains("reason_code=sbom_provenance_profile_contract_violation"));
}

#[test]
fn integration_sbom_provenance_generator_run_mode_requires_explicit_local_opt_in() {
    let tmp = TempDir::new("sbom-provenance-generator-integration");
    let report_without_opt_in = tmp.path().join("sbom-provenance-run-no-opt-in.json");
    let report_with_opt_in = tmp.path().join("sbom-provenance-run-opt-in.json");

    let without_opt_in = run_generator("baseline", "run", "FAIL", &report_without_opt_in, false);
    assert_failure(&without_opt_in, "sbom provenance run mode without opt-in");
    assert!(
        output_text(&without_opt_in)
            .contains("run mode requires explicit local-only opt-in via KAMN_SBOM_PROVENANCE_GENERATOR_OPT_IN=1"),
        "run mode must fail closed without explicit local-only opt-in"
    );

    let with_opt_in = run_generator("baseline", "run", "FAIL", &report_with_opt_in, true);
    assert_success(&with_opt_in, "sbom provenance run mode with opt-in");
    let text = output_text(&with_opt_in);
    assert!(text.contains("lane_mode=run"));
    assert!(text.contains("run_mode_command_status=sbom_provenance_generator_executed"));
    assert!(text.contains("command_count=1"));
}

#[test]
fn regression_sbom_provenance_generator_rejects_invalid_profile() {
    let tmp = TempDir::new("sbom-provenance-generator-regression");
    let report_file = tmp.path().join("sbom-provenance-invalid-profile.json");

    let output = run_generator("invalid", "dry-run", "PASS", &report_file, false);
    assert_failure(&output, "sbom provenance invalid profile");
    assert!(
        output_text(&output).contains("profile must be baseline or injected-drift"),
        "invalid profile must fail closed with deterministic error marker"
    );
}

#[test]
fn performance_sbom_provenance_generator_stays_within_budget() {
    let tmp = TempDir::new("sbom-provenance-generator-performance");
    let report_file = tmp.path().join("sbom-provenance-performance.json");

    let started = Instant::now();
    let output = run_generator("baseline", "dry-run", "PASS", &report_file, false);
    assert_success(&output, "sbom provenance performance baseline");
    let elapsed_millis = started.elapsed().as_millis();
    assert!(
        elapsed_millis < 5000,
        "sbom provenance generator exceeded 5s budget: {elapsed_millis}ms"
    );
}
