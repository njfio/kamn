use super::super::support::*;

const VALID_SOURCE: &str = r#"use std::net::{TcpListener, TcpStream};

fn read_http_request() {}
fn parse_http_request_line() {}
pub(crate) fn serve_service_api_endpoint() {}
"#;

const COUNT_DRIFT_SOURCE: &str = r#"use std::net::{TcpListener, TcpStream};

fn read_http_request() {}
fn parse_http_request_line() {}
fn parse_http_request_line() {}
pub(crate) fn serve_service_api_endpoint() {}
"#;

const BASELINE_JSON: &str = r#"{
  "schema_version": "kamn.ci.legacy-ingress-parser-baseline.v1",
  "exclude_path_fragments": [],
  "markers": [
    {
      "id": "sync_http_request_reader",
      "pattern": "fn read_http_request(",
      "max_occurrences": 1,
      "allowed_files": ["service_api_endpoint.rs"]
    },
    {
      "id": "sync_http_request_line_parser",
      "pattern": "fn parse_http_request_line(",
      "max_occurrences": 1,
      "allowed_files": ["service_api_endpoint.rs"]
    },
    {
      "id": "sync_service_endpoint_server",
      "pattern": "pub(crate) fn serve_service_api_endpoint(",
      "max_occurrences": 1,
      "allowed_files": ["service_api_endpoint.rs"]
    }
  ]
}
"#;

#[test]
fn spec_c02_legacy_ingress_parser_drift_checker_contract() {
    let checker = repo_path("scripts/ci/check_legacy_ingress_parser_drift.sh");
    assert!(
        checker.is_file(),
        "legacy ingress checker script must exist"
    );

    let tmp = TempDir::new("legacy-ingress-parser");
    let source_root = prepare_source_root(&tmp);
    let baseline_file = write_baseline_file(&tmp);

    assert_pass_case(&checker, &source_root, &baseline_file);
    assert_count_drift_case(&checker, &source_root, &baseline_file);
    assert_new_file_case(&checker, &source_root, &baseline_file);
    assert_missing_baseline_case(&checker, &source_root, tmp.path());
    assert_invalid_baseline_case(&checker, &source_root, &baseline_file);
}

fn prepare_source_root(tmp: &TempDir) -> PathBuf {
    let source_root = tmp.path().join("src");
    fs::create_dir_all(&source_root).expect("failed to create source root");
    write_service_api_endpoint(&source_root, VALID_SOURCE);
    fs::write(source_root.join("main.rs"), "fn main() {}\n").expect("failed to write main.rs");
    source_root
}

fn write_baseline_file(tmp: &TempDir) -> PathBuf {
    let baseline_file = tmp.path().join("baseline.json");
    fs::write(&baseline_file, BASELINE_JSON).expect("failed to write baseline file");
    baseline_file
}

fn write_service_api_endpoint(source_root: &Path, source: &str) {
    fs::write(source_root.join("service_api_endpoint.rs"), source)
        .expect("failed to write service_api_endpoint.rs");
}

fn run_checker(
    checker: &Path,
    source_root: &Path,
    baseline_file: &Path,
    label: &str,
) -> CommandOutput {
    run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg(checker)
                .arg("--source-root")
                .arg(source_root)
                .arg("--baseline-file")
                .arg(baseline_file);
            command
        },
        label,
    )
}

fn assert_reason_code(output: &CommandOutput, reason_code: &str, label: &str) {
    assert!(
        output_text(output).contains(reason_code),
        "{label} must emit deterministic reason code"
    );
}

fn assert_pass_case(checker: &Path, source_root: &Path, baseline_file: &Path) {
    let output = run_checker(
        checker,
        source_root,
        baseline_file,
        "legacy ingress parser checker pass case",
    );
    assert_success(&output, "legacy ingress parser pass case");
    let output_text = output_text(&output);
    assert_contains_all(
        &output_text,
        &["status=pass", "policy_decision=GO", "reason_codes=none"],
        "legacy ingress parser pass markers",
    );
}

fn assert_count_drift_case(checker: &Path, source_root: &Path, baseline_file: &Path) {
    write_service_api_endpoint(source_root, COUNT_DRIFT_SOURCE);
    let output = run_checker(
        checker,
        source_root,
        baseline_file,
        "legacy ingress parser count-drift case",
    );
    assert_failure(&output, "legacy ingress parser count-drift case");
    assert_reason_code(
        &output,
        "reason_codes=legacy_ingress_parser_marker_count_increased",
        "count-drift case",
    );
}

fn assert_new_file_case(checker: &Path, source_root: &Path, baseline_file: &Path) {
    write_service_api_endpoint(source_root, VALID_SOURCE);
    fs::write(
        source_root.join("other.rs"),
        "fn parse_http_request_line() {}\n",
    )
    .expect("failed to write non-allowed parser marker file");
    let output = run_checker(
        checker,
        source_root,
        baseline_file,
        "legacy ingress parser new-file case",
    );
    assert_failure(&output, "legacy ingress parser new-file case");
    assert_reason_code(
        &output,
        "legacy_ingress_parser_marker_new_file",
        "new-file case",
    );
}

fn assert_missing_baseline_case(checker: &Path, source_root: &Path, tmp_root: &Path) {
    let missing_baseline = tmp_root.join("missing-baseline.json");
    let output = run_checker(
        checker,
        source_root,
        &missing_baseline,
        "legacy ingress parser missing-baseline case",
    );
    assert_failure(&output, "legacy ingress parser missing-baseline case");
    assert_reason_code(
        &output,
        "reason_codes=legacy_ingress_parser_baseline_missing",
        "missing-baseline case",
    );
}

fn assert_invalid_baseline_case(checker: &Path, source_root: &Path, baseline_file: &Path) {
    fs::write(
        baseline_file,
        "{\n  \"schema_version\": \"bad-schema\"\n}\n",
    )
    .expect("failed to write invalid baseline schema");
    let output = run_checker(
        checker,
        source_root,
        baseline_file,
        "legacy ingress parser invalid-baseline case",
    );
    assert_failure(&output, "legacy ingress parser invalid-baseline case");
    assert_reason_code(
        &output,
        "reason_codes=legacy_ingress_parser_baseline_invalid",
        "invalid-baseline case",
    );
}
