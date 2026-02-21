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

fn run_generator(profile: &str, output_file: &Path) -> Output {
    run_command(
        {
            let mut command = Command::new("python3");
            command
                .arg("scripts/deploy/sbom_provenance_artifact_generator_contract.py")
                .arg("--profile")
                .arg(profile)
                .arg("--mode")
                .arg("dry-run")
                .arg("--ci-fast-gate")
                .arg("PASS")
                .arg("--max-seconds")
                .arg("120")
                .arg("--output-json")
                .arg(output_file);
            command
        },
        "sbom provenance artifact generator",
    )
}

fn run_checker(
    artifact_file: &Path,
    ci_strategy_doc: &Path,
    ops_configuration_doc: &Path,
    report_file: &Path,
    max_seconds: &str,
) -> Output {
    run_command(
        {
            let mut command = Command::new("python3");
            command
                .arg("scripts/deploy/sbom_provenance_release_gonogo_checker_contract.py")
                .arg("--artifact-json")
                .arg(artifact_file)
                .arg("--ci-strategy-doc")
                .arg(ci_strategy_doc)
                .arg("--ops-configuration-doc")
                .arg(ops_configuration_doc)
                .arg("--max-seconds")
                .arg(max_seconds)
                .arg("--output-json")
                .arg(report_file);
            command
        },
        "sbom provenance release go-no-go checker",
    )
}

#[test]
fn unit_sbom_provenance_release_checker_accepts_baseline_artifact_marker_contract() {
    let tmp = TempDir::new("sbom-provenance-release-checker-unit");
    let artifact_file = tmp.path().join("sbom-provenance-baseline.json");
    let report_file = tmp
        .path()
        .join("sbom-provenance-release-gonogo-report.json");

    let generator_output = run_generator("baseline", &artifact_file);
    assert_success(&generator_output, "baseline artifact generation");

    let checker_output = run_checker(
        &artifact_file,
        &repo_root().join("docs/ci/strategy.md"),
        &repo_root().join("docs/ops/configuration.md"),
        &report_file,
        "120",
    );
    assert_success(&checker_output, "baseline release go-no-go check");

    let output = output_text(&checker_output);
    assert!(output.contains("status=pass"));
    assert!(output.contains("final_decision=GO"));
    assert!(output.contains("reason_code=none"));
    assert!(output.contains(
        "checker_schema_version=kamn.runtime.sbom-provenance-release-gonogo-checker-report.v1"
    ));
    assert!(output.contains(
        "checker_reason_taxonomy_version=kamn.runtime.sbom-provenance-release-gonogo-checker-reason-taxonomy.v1"
    ));

    let payload = load_text(&report_file);
    assert_eq!(
        json_string_field(&payload, "schema_version").as_deref(),
        Some("kamn.runtime.sbom-provenance-release-gonogo-checker-report.v1")
    );
    assert_eq!(
        json_string_field(&payload, "reason_taxonomy_version").as_deref(),
        Some("kamn.runtime.sbom-provenance-release-gonogo-checker-reason-taxonomy.v1")
    );
}

#[test]
fn functional_sbom_provenance_release_checker_fails_closed_on_missing_required_artifact_marker() {
    let tmp = TempDir::new("sbom-provenance-release-checker-functional");
    let artifact_file = tmp.path().join("sbom-provenance-baseline.json");
    let tampered_artifact_file = tmp.path().join("sbom-provenance-tampered.json");
    let report_file = tmp
        .path()
        .join("sbom-provenance-release-gonogo-report-functional.json");

    let generator_output = run_generator("baseline", &artifact_file);
    assert_success(&generator_output, "baseline artifact generation");

    let mut artifact_text = load_text(&artifact_file);
    artifact_text = artifact_text.replace(
        "\"release_manifest_required_artifact_id\": \"sbom_provenance\"",
        "\"release_manifest_required_artifact_id\": \"\"",
    );
    fs::write(&tampered_artifact_file, artifact_text)
        .expect("failed to write tampered artifact file");

    let checker_output = run_checker(
        &tampered_artifact_file,
        &repo_root().join("docs/ci/strategy.md"),
        &repo_root().join("docs/ops/configuration.md"),
        &report_file,
        "120",
    );
    assert_failure(&checker_output, "tampered artifact release go-no-go check");

    let output = output_text(&checker_output);
    assert!(output.contains("status=fail"));
    assert!(output.contains("final_decision=NO-GO"));
    assert!(output.contains("reason_code=sbom_provenance_artifact_marker_missing"));
}

#[test]
fn integration_sbom_provenance_release_checker_validates_generator_and_docs_parity() {
    let tmp = TempDir::new("sbom-provenance-release-checker-integration");
    let artifact_file = tmp.path().join("sbom-provenance-baseline.json");
    let report_file = tmp
        .path()
        .join("sbom-provenance-release-gonogo-report-integration.json");

    let generator_output = run_generator("baseline", &artifact_file);
    assert_success(&generator_output, "baseline artifact generation");

    let checker_output = run_checker(
        &artifact_file,
        &repo_root().join("docs/ci/strategy.md"),
        &repo_root().join("docs/ops/configuration.md"),
        &report_file,
        "120",
    );
    assert_success(&checker_output, "integration release go-no-go check");

    let output = output_text(&checker_output);
    assert!(output.contains("artifact_marker_contract_status=verified"));
    assert!(output.contains("docs_parity_status=verified"));
    assert!(output.contains("strategy_doc_marker_status=verified"));
    assert!(output.contains("ops_configuration_doc_marker_status=verified"));
}

#[test]
fn regression_sbom_provenance_release_checker_detects_docs_marker_drift() {
    let tmp = TempDir::new("sbom-provenance-release-checker-regression");
    let artifact_file = tmp.path().join("sbom-provenance-baseline.json");
    let report_file = tmp
        .path()
        .join("sbom-provenance-release-gonogo-report-regression.json");
    let ci_doc_copy = tmp.path().join("strategy-drifted.md");

    let generator_output = run_generator("baseline", &artifact_file);
    assert_success(&generator_output, "baseline artifact generation");

    let mut ci_doc_text = load_text(&repo_root().join("docs/ci/strategy.md"));
    ci_doc_text = ci_doc_text.replace(
        "sbom_provenance_release_gonogo_checker_schema_version=kamn.runtime.sbom-provenance-release-gonogo-checker-report.v1",
        "",
    );
    fs::write(&ci_doc_copy, ci_doc_text).expect("failed to write drifted ci strategy doc copy");

    let checker_output = run_checker(
        &artifact_file,
        &ci_doc_copy,
        &repo_root().join("docs/ops/configuration.md"),
        &report_file,
        "120",
    );
    assert_failure(&checker_output, "docs marker drift release go-no-go check");

    let output = output_text(&checker_output);
    assert!(output.contains("status=fail"));
    assert!(output.contains("final_decision=NO-GO"));
    assert!(output.contains("reason_code=sbom_provenance_docs_parity_marker_missing"));
}

#[test]
fn performance_sbom_provenance_release_checker_stays_within_budget() {
    let tmp = TempDir::new("sbom-provenance-release-checker-performance");
    let artifact_file = tmp.path().join("sbom-provenance-baseline.json");
    let report_file = tmp
        .path()
        .join("sbom-provenance-release-gonogo-report-performance.json");

    let generator_output = run_generator("baseline", &artifact_file);
    assert_success(&generator_output, "baseline artifact generation");

    let started = Instant::now();
    let checker_output = run_checker(
        &artifact_file,
        &repo_root().join("docs/ci/strategy.md"),
        &repo_root().join("docs/ops/configuration.md"),
        &report_file,
        "120",
    );
    assert_success(&checker_output, "performance release go-no-go check");

    let elapsed_millis = started.elapsed().as_millis();
    assert!(
        elapsed_millis < 5000,
        "sbom provenance release checker exceeded 5s budget: {elapsed_millis}ms"
    );
}
