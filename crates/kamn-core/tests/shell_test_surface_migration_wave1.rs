use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const FAST_WORKFLOW: &str = ".github/workflows/ci-fast-gate.yml";
const DEEP_WORKFLOW: &str = ".github/workflows/ci-deep-validate.yml";
const CI_TOOLS_SCRIPT: &str = "scripts/ci/test_ci_tools.sh";
const CI_STRATEGY_DOC: &str = "docs/ci/strategy.md";

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

fn extract_fast_mode_block(ci_tools_script: &str) -> String {
    let start_marker = "if [ \"${KAMN_CI_TOOLS_FAST_MODE:-false}\" = \"true\" ]; then";
    let end_marker = "  echo \"Fast-mode CI tool regression tests passed.\"";
    let start = ci_tools_script
        .find(start_marker)
        .expect("missing fast-mode block start marker")
        + start_marker.len();
    let end = ci_tools_script[start..]
        .find(end_marker)
        .map(|index| start + index)
        .expect("missing fast-mode block end marker");
    ci_tools_script[start..end].to_owned()
}

fn assert_contains_all(haystack: &str, needles: &[&str], context: &str) {
    for needle in needles {
        assert!(
            haystack.contains(needle),
            "{context} missing expected marker: {needle}"
        );
    }
}

fn extract_json_string_field(raw_json: &str, field: &str) -> Option<String> {
    let marker = format!("\"{field}\": \"");
    let start = raw_json.find(&marker)? + marker.len();
    let rest = &raw_json[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_owned())
}

#[test]
fn spec_c01_block_reconciliation_partition_rejoin_ci_exclusion_policy_markers() {
    let fast_workflow = read_text(FAST_WORKFLOW);
    let ci_tools = read_text(CI_TOOLS_SCRIPT);
    let ci_tools_fast_mode = extract_fast_mode_block(&ci_tools);
    let strategy_doc = read_text(CI_STRATEGY_DOC);

    assert!(
        !fast_workflow.contains(
            "bash scripts/runtime/validate_block_reconciliation_partition_rejoin_live.sh --mode run"
        ),
        "block reconciliation run-mode lane must remain excluded from ci-fast-gate"
    );
    assert!(
        !ci_tools_fast_mode.contains(
            "bash \"$ROOT_DIR/scripts/runtime/validate_block_reconciliation_partition_rejoin_live.sh\" --mode run"
        ),
        "block reconciliation run-mode lane must remain excluded from ci-tools fast mode"
    );

    assert_contains_all(
        &ci_tools,
        &[
            "bash \"$ROOT_DIR/scripts/runtime/test_validate_block_reconciliation_partition_rejoin_live.sh\"",
            "bash \"$ROOT_DIR/scripts/runtime/test_check_block_reconciliation_partition_rejoin_live_policy.sh\"",
            "bash \"$ROOT_DIR/scripts/runtime/test_validate_block_reconciliation_partition_rejoin_live_contract_lane.sh\"",
        ],
        "ci-tools block reconciliation command surface",
    );

    assert!(
        strategy_doc.contains(
            "block reconciliation partition/rejoin run-mode commands remain excluded from ci-fast-gate and ci-tools fast mode."
        ),
        "ci strategy doc missing block reconciliation exclusion marker"
    );
}

#[test]
fn spec_c02_legacy_ingress_parser_drift_checker_contract() {
    let checker = repo_path("scripts/ci/check_legacy_ingress_parser_drift.sh");
    assert!(
        checker.is_file(),
        "legacy ingress checker script must exist"
    );

    let tmp = TempDir::new("legacy-ingress-parser");
    let source_root = tmp.path().join("src");
    fs::create_dir_all(&source_root).expect("failed to create source root");

    fs::write(
        source_root.join("service_api_endpoint.rs"),
        r#"use std::net::{TcpListener, TcpStream};

fn read_http_request() {}
fn parse_http_request_line() {}
pub(crate) fn serve_service_api_endpoint() {}
"#,
    )
    .expect("failed to write service_api_endpoint.rs");
    fs::write(source_root.join("main.rs"), "fn main() {}\n").expect("failed to write main.rs");

    let baseline_file = tmp.path().join("baseline.json");
    fs::write(
        &baseline_file,
        r#"{
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
"#,
    )
    .expect("failed to write baseline file");

    let pass_output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg(&checker)
                .arg("--source-root")
                .arg(&source_root)
                .arg("--baseline-file")
                .arg(&baseline_file);
            command
        },
        "legacy ingress parser checker pass case",
    );
    assert_success(&pass_output, "legacy ingress parser pass case");
    let pass_text = output_text(&pass_output);
    assert_contains_all(
        &pass_text,
        &["status=pass", "policy_decision=GO", "reason_codes=none"],
        "legacy ingress parser pass markers",
    );

    fs::write(
        source_root.join("service_api_endpoint.rs"),
        r#"use std::net::{TcpListener, TcpStream};

fn read_http_request() {}
fn parse_http_request_line() {}
fn parse_http_request_line() {}
pub(crate) fn serve_service_api_endpoint() {}
"#,
    )
    .expect("failed to mutate service_api_endpoint.rs for count drift");

    let count_fail_output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg(&checker)
                .arg("--source-root")
                .arg(&source_root)
                .arg("--baseline-file")
                .arg(&baseline_file);
            command
        },
        "legacy ingress parser count-drift case",
    );
    assert_failure(&count_fail_output, "legacy ingress parser count-drift case");
    assert!(
        output_text(&count_fail_output)
            .contains("reason_codes=legacy_ingress_parser_marker_count_increased"),
        "count-drift case must emit deterministic reason code"
    );

    fs::write(
        source_root.join("service_api_endpoint.rs"),
        r#"use std::net::{TcpListener, TcpStream};

fn read_http_request() {}
fn parse_http_request_line() {}
pub(crate) fn serve_service_api_endpoint() {}
"#,
    )
    .expect("failed to reset service_api_endpoint.rs");
    fs::write(
        source_root.join("other.rs"),
        "fn parse_http_request_line() {}\n",
    )
    .expect("failed to write non-allowed parser marker file");

    let new_file_fail_output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg(&checker)
                .arg("--source-root")
                .arg(&source_root)
                .arg("--baseline-file")
                .arg(&baseline_file);
            command
        },
        "legacy ingress parser new-file case",
    );
    assert_failure(&new_file_fail_output, "legacy ingress parser new-file case");
    assert!(
        output_text(&new_file_fail_output).contains("legacy_ingress_parser_marker_new_file"),
        "new-file case must emit deterministic reason code"
    );

    let missing_baseline = tmp.path().join("missing-baseline.json");
    let missing_baseline_output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg(&checker)
                .arg("--source-root")
                .arg(&source_root)
                .arg("--baseline-file")
                .arg(&missing_baseline);
            command
        },
        "legacy ingress parser missing-baseline case",
    );
    assert_failure(
        &missing_baseline_output,
        "legacy ingress parser missing-baseline case",
    );
    assert!(
        output_text(&missing_baseline_output)
            .contains("reason_codes=legacy_ingress_parser_baseline_missing"),
        "missing-baseline case must emit deterministic reason code"
    );

    fs::write(
        &baseline_file,
        "{\n  \"schema_version\": \"bad-schema\"\n}\n",
    )
    .expect("failed to write invalid baseline schema");

    let invalid_baseline_output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg(&checker)
                .arg("--source-root")
                .arg(&source_root)
                .arg("--baseline-file")
                .arg(&baseline_file);
            command
        },
        "legacy ingress parser invalid-baseline case",
    );
    assert_failure(
        &invalid_baseline_output,
        "legacy ingress parser invalid-baseline case",
    );
    assert!(
        output_text(&invalid_baseline_output)
            .contains("reason_codes=legacy_ingress_parser_baseline_invalid"),
        "invalid-baseline case must emit deterministic reason code"
    );
}

#[test]
fn spec_c03_performance_threshold_checker_contract() {
    let checker = repo_path("scripts/ci/check_performance_thresholds.sh");
    let profile_file = repo_path(".ci/performance-targets.env");
    assert!(
        checker.is_file() && profile_file.is_file(),
        "performance threshold checker and profile fixture must exist"
    );

    let tmp = TempDir::new("performance-thresholds");
    let pass_report = tmp.path().join("pass.json");
    let fail_report = tmp.path().join("fail.json");
    let invalid_report = tmp.path().join("invalid.json");

    fs::write(
        &pass_report,
        r#"{
  "latency_p50_ms": 88,
  "latency_p99_ms": 300,
  "throughput_tps": 12500,
  "availability_pct": 99.95,
  "baseline_provenance_artifact_version": "kamn.ci.performance-baseline.v1",
  "baseline_provenance_source_commit": "abc123def456",
  "baseline_provenance_source_run_id": "run-1001",
  "baseline_provenance_generated_at_utc": "2026-02-20T12:00:00Z",
  "baseline_provenance_generator": "scripts/ci/generate_performance_smoke_report.sh",
  "drift_threshold_seed_id": "smoke-seed-v1",
  "drift_threshold_seed_max_latency_p50_ms": 95,
  "drift_threshold_seed_max_latency_p99_ms": 350,
  "drift_threshold_seed_min_throughput_tps": 12000,
  "drift_threshold_seed_min_availability_pct": 99.9
}
"#,
    )
    .expect("failed to write pass report");
    fs::write(
        &fail_report,
        r#"{
  "latency_p50_ms": 101,
  "latency_p99_ms": 540,
  "throughput_tps": 9800,
  "availability_pct": 99.7,
  "baseline_provenance_artifact_version": "kamn.ci.performance-baseline.v1",
  "baseline_provenance_source_commit": "abc123def456",
  "baseline_provenance_source_run_id": "run-1001",
  "baseline_provenance_generated_at_utc": "2026-02-20T12:00:00Z",
  "baseline_provenance_generator": "scripts/ci/generate_performance_smoke_report.sh",
  "drift_threshold_seed_id": "smoke-seed-v1",
  "drift_threshold_seed_max_latency_p50_ms": 95,
  "drift_threshold_seed_max_latency_p99_ms": 350,
  "drift_threshold_seed_min_throughput_tps": 12000,
  "drift_threshold_seed_min_availability_pct": 99.9
}
"#,
    )
    .expect("failed to write fail report");
    fs::write(
        &invalid_report,
        r#"{
  "latency_p50_ms": 90,
  "throughput_tps": 12000
}
"#,
    )
    .expect("failed to write invalid report");

    let pass_output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg(&checker)
                .arg("--report-json")
                .arg(&pass_report)
                .arg("--profile-file")
                .arg(&profile_file)
                .arg("--lane")
                .arg("smoke");
            command
        },
        "performance threshold pass case",
    );
    assert_success(&pass_output, "performance threshold pass case");
    assert!(
        output_text(&pass_output).contains("status=pass"),
        "pass case must emit status=pass"
    );

    let fail_output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg(&checker)
                .arg("--report-json")
                .arg(&fail_report)
                .arg("--profile-file")
                .arg(&profile_file)
                .arg("--lane")
                .arg("smoke");
            command
        },
        "performance threshold fail case",
    );
    assert_failure(&fail_output, "performance threshold fail case");
    let fail_text = output_text(&fail_output);
    assert_contains_all(
        &fail_text,
        &[
            "status=fail",
            "performance_ci_smoke_reason_codes_value=",
            "performance_ci_smoke_latency_p50_threshold_exceeded",
            "performance_ci_smoke_throughput_threshold_below_minimum",
        ],
        "performance threshold fail markers",
    );

    let invalid_output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg(&checker)
                .arg("--report-json")
                .arg(&invalid_report)
                .arg("--profile-file")
                .arg(&profile_file)
                .arg("--lane")
                .arg("smoke");
            command
        },
        "performance threshold invalid-schema case",
    );
    assert_failure(&invalid_output, "performance threshold invalid-schema case");
    assert!(
        output_text(&invalid_output).contains("performance_ci_smoke_report_contract_violation"),
        "invalid-schema case must emit deterministic report-contract violation reason"
    );
}

#[test]
fn spec_c04_workflow_kolme_heavy_exclusion_checker_contract() {
    let checker = repo_path("scripts/ci/check_workflow_kolme_heavy_exclusion_policy.py");
    let selector = repo_path("scripts/ci/select_targets.sh");
    let ci_tools = repo_path(CI_TOOLS_SCRIPT);
    let safe_fixture = repo_path("fixtures/ci/workflow_kolme_heavy_policy_safe.yml");
    let missing_input_fixture =
        repo_path("fixtures/ci/workflow_kolme_heavy_policy_unsafe_missing_input.yml");
    let forced_true_fixture =
        repo_path("fixtures/ci/workflow_kolme_heavy_policy_unsafe_forced_true.yml");
    let missing_local_heavy_command_fixture =
        repo_path("fixtures/ci/workflow_kolme_heavy_policy_unsafe_missing_local_heavy_command.yml");
    assert!(
        checker.is_file()
            && selector.is_file()
            && ci_tools.is_file()
            && safe_fixture.is_file()
            && missing_input_fixture.is_file()
            && forced_true_fixture.is_file()
            && missing_local_heavy_command_fixture.is_file(),
        "workflow kolmen heavy exclusion checker fixtures must exist"
    );

    let tmp = TempDir::new("workflow-kolme-heavy");
    let safe_report = tmp.path().join("safe-report.json");
    let safe_output = run_command(
        {
            let mut command = Command::new("python3");
            command
                .arg(&checker)
                .arg("--workflow-file")
                .arg(&safe_fixture)
                .arg("--selector-file")
                .arg(&selector)
                .arg("--ci-tools-file")
                .arg(&ci_tools)
                .arg("--output-json")
                .arg(&safe_report);
            command
        },
        "workflow heavy checker safe fixture",
    );
    assert_success(&safe_output, "workflow heavy checker safe fixture");
    let safe_text = output_text(&safe_output);
    assert_contains_all(
        &safe_text,
        &["status=pass", "reason_codes=none"],
        "workflow heavy checker safe fixture markers",
    );
    let safe_report_text =
        fs::read_to_string(&safe_report).expect("failed to read safe workflow checker report");
    assert!(
        safe_report_text.contains("\"final_decision\": \"GO\""),
        "safe workflow checker report must contain GO decision"
    );

    let fast_report = tmp.path().join("fast-report.json");
    let fast_output = run_command(
        {
            let mut command = Command::new("python3");
            command
                .arg(&checker)
                .arg("--workflow-file")
                .arg(repo_path(FAST_WORKFLOW))
                .arg("--selector-file")
                .arg(&selector)
                .arg("--ci-tools-file")
                .arg(&ci_tools)
                .arg("--output-json")
                .arg(&fast_report);
            command
        },
        "workflow heavy checker ci-fast-gate workflow",
    );
    assert_success(&fast_output, "workflow heavy checker ci-fast-gate workflow");
    assert_contains_all(
        &output_text(&fast_output),
        &["status=pass", "reason_codes=none"],
        "workflow heavy checker ci-fast-gate markers",
    );

    let missing_input_output = run_command(
        {
            let mut command = Command::new("python3");
            command
                .arg(&checker)
                .arg("--workflow-file")
                .arg(&missing_input_fixture);
            command
        },
        "workflow heavy checker missing-input fixture",
    );
    assert_failure(
        &missing_input_output,
        "workflow heavy checker missing-input fixture",
    );
    assert!(
        output_text(&missing_input_output).contains("workflow_dispatch_input_missing"),
        "missing-input fixture must emit deterministic reason code"
    );

    let forced_true_output = run_command(
        {
            let mut command = Command::new("python3");
            command
                .arg(&checker)
                .arg("--workflow-file")
                .arg(&forced_true_fixture);
            command
        },
        "workflow heavy checker forced-true fixture",
    );
    assert_failure(
        &forced_true_output,
        "workflow heavy checker forced-true fixture",
    );
    assert!(
        output_text(&forced_true_output).contains("selector_opt_in_env_forced_true_literal"),
        "forced-true fixture must emit deterministic reason code"
    );

    let missing_local_heavy_output = run_command(
        {
            let mut command = Command::new("python3");
            command
                .arg(&checker)
                .arg("--workflow-file")
                .arg(&missing_local_heavy_command_fixture);
            command
        },
        "workflow heavy checker missing-local-heavy fixture",
    );
    assert_failure(
        &missing_local_heavy_output,
        "workflow heavy checker missing-local-heavy fixture",
    );
    assert!(
        output_text(&missing_local_heavy_output).contains("local_heavy_lane_commands_missing"),
        "missing-local-heavy fixture must emit deterministic reason code"
    );
}

#[test]
fn spec_c05_fallback_retirement_docs_parity_markers() {
    let docs = [
        read_text("README.md"),
        read_text("docs/ci/strategy.md"),
        read_text("docs/planning/kolme-devnet-ops.md"),
    ];
    let markers = [
        "fallback_signer_private_key_env=KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK",
        "fallback_signer_secret_remediation=unset KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK",
        "contracts.fallback_signer_secret_rejected_profile_class=production",
        "contracts.fallback_signer_secret_remediation=unset KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK",
        "contracts.fallback_signer_secret_checkpoint_reason_code=checkpoint_failed_fallback_private_key_contract",
        "fallback_signer_secret_present_violation",
        "fallback_signer_secret_checkpoint_reason_mismatch",
        "Regression: #2337",
    ];

    for marker in markers {
        for (doc_index, doc) in docs.iter().enumerate() {
            assert!(
                doc.contains(marker),
                "fallback marker missing from doc #{doc_index}: {marker}"
            );
        }
    }
}

#[test]
fn spec_c06_generate_test_harness_loc_report_contract() {
    let generator = repo_path("scripts/ci/generate_test_harness_loc_report.sh");
    assert!(
        generator.is_file(),
        "test-harness LOC report generator script must exist"
    );

    let tmp = TempDir::new("test-harness-loc-report");
    let scripts_root = tmp.path().join("scripts");
    fs::create_dir_all(scripts_root.join("ci")).expect("failed to create ci script fixture root");
    fs::create_dir_all(scripts_root.join("sdk")).expect("failed to create sdk script fixture root");

    fs::write(
        scripts_root.join("ci/test_alpha.sh"),
        "#!/usr/bin/env bash\necho \"alpha\"\n",
    )
    .expect("failed to write ci/test_alpha.sh");
    fs::write(
        scripts_root.join("sdk/test_beta.sh"),
        "#!/usr/bin/env bash\necho \"beta\"\n",
    )
    .expect("failed to write sdk/test_beta.sh");
    fs::write(
        scripts_root.join("sdk/run_non_harness.sh"),
        "#!/usr/bin/env bash\necho \"ignore\"\n",
    )
    .expect("failed to write sdk/run_non_harness.sh");

    let report_json = tmp.path().join("test-harness-loc-report.json");
    let output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg(&generator)
                .arg("--scripts-root")
                .arg(&scripts_root)
                .arg("--output-json")
                .arg(&report_json);
            command
        },
        "test-harness LOC report generation",
    );
    assert_success(&output, "test-harness LOC report generation");
    let output_text = output_text(&output);
    assert_contains_all(
        &output_text,
        &[
            "status=ok",
            "harness_script_count=2",
            "harness_shell_line_total=4",
            "report_file=",
        ],
        "test-harness LOC report output markers",
    );

    let report_text =
        fs::read_to_string(&report_json).expect("failed to read generated LOC report JSON");
    assert_contains_all(
        &report_text,
        &[
            "\"schema_version\": \"kamn.ci.test-harness-loc-report.v1\"",
            "\"harness_script_count\": 2",
            "\"harness_shell_line_total\": 4",
            "\"domains\"",
            "\"harness_scripts\"",
        ],
        "test-harness LOC report JSON markers",
    );
}

#[test]
fn spec_c07_local_metrics_scrape_ci_exclusion_policy_markers() {
    let fast_workflow = read_text(FAST_WORKFLOW);
    let ci_tools = read_text(CI_TOOLS_SCRIPT);
    let ci_tools_fast_mode = extract_fast_mode_block(&ci_tools);
    let strategy_doc = read_text(CI_STRATEGY_DOC);

    assert!(
        !fast_workflow
            .contains("bash scripts/runtime/validate_local_metrics_scrape_live.sh --mode run"),
        "local metrics scrape run-mode lane must remain excluded from ci-fast-gate"
    );
    assert!(
        !ci_tools_fast_mode
            .contains("bash \"$ROOT_DIR/scripts/runtime/test_validate_local_metrics_scrape_live_contract_lane.sh\""),
        "local metrics scrape contract lane must remain excluded from ci-tools fast mode"
    );
    assert_contains_all(
        &ci_tools,
        &[
            "bash \"$ROOT_DIR/scripts/runtime/test_check_local_metrics_scrape_live_policy.sh\"",
            "bash \"$ROOT_DIR/scripts/runtime/test_validate_local_metrics_scrape_live_contract_lane.sh\"",
        ],
        "ci-tools local metrics scrape command surface",
    );
    assert!(
        strategy_doc.contains(
            "local metrics scrape run-mode commands remain excluded from ci-fast-gate and ci-tools fast mode."
        ),
        "ci strategy doc missing local metrics scrape exclusion marker"
    );
}

#[test]
fn spec_c08_local_retry_diagnostics_ci_exclusion_policy_markers() {
    let fast_workflow = read_text(FAST_WORKFLOW);
    let ci_tools = read_text(CI_TOOLS_SCRIPT);
    let ci_tools_fast_mode = extract_fast_mode_block(&ci_tools);
    let strategy_doc = read_text(CI_STRATEGY_DOC);

    assert!(
        !fast_workflow
            .contains("bash scripts/runtime/validate_local_retry_diagnostics_live.sh --mode run"),
        "local retry diagnostics run-mode lane must remain excluded from ci-fast-gate"
    );
    assert!(
        !ci_tools_fast_mode.contains(
            "bash \"$ROOT_DIR/scripts/runtime/validate_local_retry_diagnostics_live.sh\" --mode run"
        ),
        "local retry diagnostics run-mode lane must remain excluded from ci-tools fast mode"
    );
    assert_contains_all(
        &ci_tools,
        &[
            "bash \"$ROOT_DIR/scripts/runtime/test_validate_local_retry_diagnostics_live.sh\"",
            "bash \"$ROOT_DIR/scripts/runtime/test_check_local_retry_diagnostics_live_policy.sh\"",
            "bash \"$ROOT_DIR/scripts/runtime/test_validate_local_retry_diagnostics_live_contract_lane.sh\"",
        ],
        "ci-tools local retry diagnostics command surface",
    );
    assert!(
        strategy_doc.contains(
            "local retry/diagnostics run-mode commands remain excluded from ci-fast-gate and ci-tools fast mode."
        ),
        "ci strategy doc missing local retry diagnostics exclusion marker"
    );
}

#[test]
fn spec_c09_kolme_test_harness_loc_soft_budget_manifest_contract_lane() {
    let legacy_lane_script =
        repo_path("scripts/ci/run_kolme_test_harness_loc_soft_budget_contract_lane.sh");
    let manifest_runner = repo_path("scripts/framework/run_manifest_lane.sh");
    let shared_impl =
        repo_path("scripts/ci/kolme_test_harness_loc_soft_budget_contract_lane_impl.sh");
    let manifest_file = repo_path(
        "scripts/framework/manifests/ci_kolme_test_harness_loc_soft_budget_contract_lane.json",
    );
    let strategy_doc = read_text("docs/ci/strategy.md");
    let cost_doc = read_text("docs/ci/ci-cost-and-lane-framework.md");

    assert!(
        !legacy_lane_script.exists(),
        "superseded Kolme soft-budget wrapper must be deleted"
    );
    assert!(
        manifest_runner.is_file() && shared_impl.is_file() && manifest_file.is_file(),
        "manifest runner, shared impl, and manifest must exist"
    );

    let tmp = TempDir::new("kolme-soft-budget-contract");
    let report_file = tmp
        .path()
        .join("kolme-test-harness-soft-budget-contract-report.json");

    let lane_output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg(&manifest_runner)
                .arg("--manifest")
                .arg(&manifest_file)
                .arg("--phase")
                .arg("contract")
                .arg("--")
                .arg("--output-json")
                .arg(&report_file)
                .arg("--max-runtime-seconds")
                .arg("120");
            command
        },
        "Kolme soft-budget manifest contract lane",
    );
    assert_success(&lane_output, "Kolme soft-budget manifest contract lane");
    let lane_text = output_text(&lane_output);
    assert_contains_all(
        &lane_text,
        &[
            "kolme_test_harness_loc_soft_budget_contract_status=pass",
            "kolme_test_harness_loc_soft_budget_contract_go_decision=GO",
            "kolme_test_harness_loc_soft_budget_contract_warn_decision=WARN",
            "kolme_test_harness_loc_soft_budget_contract_fail_decision=NO-GO",
        ],
        "Kolme soft-budget manifest contract lane markers",
    );

    let report_text = fs::read_to_string(&report_file)
        .expect("failed to read Kolme soft-budget contract report JSON");
    assert_contains_all(
        &report_text,
        &[
            "\"schema_version\": \"kamn.ci.kolme-test-harness-loc-soft-budget-contract-report.v1\"",
            "\"combined_reason_code_contract\": \"pass\"",
            "\"command_surface_fail_reason_contract\": \"pass\"",
        ],
        "Kolme soft-budget contract report markers",
    );

    let manifest_runner_command = "run_manifest_lane.sh --manifest scripts/framework/manifests/ci_kolme_test_harness_loc_soft_budget_contract_lane.json --phase contract --output-json /tmp/kolme-test-harness-loc-soft-budget-contract-report.json";
    assert!(
        strategy_doc.contains(manifest_runner_command),
        "ci strategy doc missing Kolme soft-budget manifest-runner command marker"
    );
    assert!(
        cost_doc.contains(manifest_runner_command),
        "ci cost doc missing Kolme soft-budget manifest-runner command marker"
    );

    let manifest_text =
        fs::read_to_string(&manifest_file).expect("failed to read Kolme soft-budget manifest file");
    assert!(
        manifest_text.contains("kolme_test_harness_loc_soft_budget_contract_lane_impl.sh"),
        "Kolme soft-budget manifest must dispatch shared implementation script"
    );
}

#[test]
fn spec_c10_run_with_retry_contract() {
    let script = repo_path("scripts/ci/run_with_retry.sh");
    assert!(script.is_file(), "run_with_retry helper must exist");

    let tmp = TempDir::new("run-with-retry");
    let out_success = tmp.path().join("out_success.txt");
    let out_retry = tmp.path().join("out_retry.txt");
    let counter_file = tmp.path().join("counter");
    let flaky_once = tmp.path().join("flaky_once.sh");

    let immediate_output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg(&script)
                .arg("--label")
                .arg("immediate")
                .arg("--max-attempts")
                .arg("2")
                .arg("--")
                .arg("bash")
                .arg("-lc")
                .arg("exit 0");
            command.env("GITHUB_OUTPUT", &out_success);
            command
        },
        "run_with_retry immediate-success case",
    );
    assert_success(&immediate_output, "run_with_retry immediate-success case");

    let success_output_markers =
        fs::read_to_string(&out_success).expect("failed to read run_with_retry success output");
    assert_contains_all(
        &success_output_markers,
        &[
            "retry_attempts<<EOF\n1\nEOF",
            "retry_used<<EOF\nfalse\nEOF",
            "retry_final_status<<EOF\npassed\nEOF",
        ],
        "run_with_retry immediate-success output markers",
    );

    fs::write(
        &flaky_once,
        "#!/usr/bin/env bash\nset -euo pipefail\nf=\"$1\"\nif [ ! -f \"$f\" ]; then\n  echo 1 > \"$f\"\n  exit 1\nfi\nexit 0\n",
    )
    .expect("failed to write flaky_once script");

    let flaky_output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg(&script)
                .arg("--label")
                .arg("flaky-once")
                .arg("--max-attempts")
                .arg("2")
                .arg("--")
                .arg("bash")
                .arg(&flaky_once)
                .arg(&counter_file);
            command.env("GITHUB_OUTPUT", &out_retry);
            command
        },
        "run_with_retry flaky-once case",
    );
    assert_success(&flaky_output, "run_with_retry flaky-once case");

    let retry_output_markers =
        fs::read_to_string(&out_retry).expect("failed to read run_with_retry retry output");
    assert_contains_all(
        &retry_output_markers,
        &[
            "retry_attempts<<EOF\n2\nEOF",
            "retry_used<<EOF\ntrue\nEOF",
            "retry_final_status<<EOF\npassed\nEOF",
        ],
        "run_with_retry flaky-once output markers",
    );

    let always_fail_output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg(&script)
                .arg("--label")
                .arg("always-fail")
                .arg("--max-attempts")
                .arg("2")
                .arg("--")
                .arg("bash")
                .arg("-lc")
                .arg("exit 1");
            command
        },
        "run_with_retry always-fail case",
    );
    assert_failure(&always_fail_output, "run_with_retry always-fail case");
}

#[test]
fn spec_c11_service_api_reason_code_compatibility_ci_exclusion_policy_markers() {
    let fast_workflow = read_text(FAST_WORKFLOW);
    let ci_tools = read_text(CI_TOOLS_SCRIPT);
    let ci_tools_fast_mode = extract_fast_mode_block(&ci_tools);
    let strategy_doc = read_text(CI_STRATEGY_DOC);

    assert!(
        !fast_workflow.contains(
            "bash scripts/runtime/validate_service_api_reason_code_compatibility_live.sh"
        ),
        "service API reason-code compatibility lane must remain excluded from ci-fast-gate"
    );
    assert!(
        !ci_tools_fast_mode.contains(
            "bash \"$ROOT_DIR/scripts/runtime/test_validate_service_api_reason_code_compatibility_live_contract_lane.sh\""
        ),
        "service API reason-code compatibility lane must remain excluded from ci-tools fast mode"
    );
    assert_contains_all(
        &ci_tools,
        &[
            "bash \"$ROOT_DIR/scripts/runtime/test_check_service_api_reason_code_compatibility_live_policy.sh\"",
            "bash \"$ROOT_DIR/scripts/runtime/test_validate_service_api_reason_code_compatibility_live_contract_lane.sh\"",
        ],
        "ci-tools service API reason-code compatibility command surface",
    );
    assert!(
        strategy_doc.contains(
            "service api reason-code compatibility contract-lane commands remain excluded from ci-fast-gate and ci-tools fast mode."
        ),
        "ci strategy doc missing service API reason-code compatibility exclusion marker"
    );
}

#[test]
fn spec_c12_service_api_serde_payload_parity_ci_exclusion_policy_markers() {
    let fast_workflow = read_text(FAST_WORKFLOW);
    let ci_tools = read_text(CI_TOOLS_SCRIPT);
    let ci_tools_fast_mode = extract_fast_mode_block(&ci_tools);
    let strategy_doc = read_text(CI_STRATEGY_DOC);

    assert!(
        !fast_workflow
            .contains("bash scripts/runtime/validate_service_api_serde_payload_parity_live.sh"),
        "service API serde payload parity lane must remain excluded from ci-fast-gate"
    );
    assert!(
        !ci_tools_fast_mode.contains(
            "bash \"$ROOT_DIR/scripts/runtime/test_validate_service_api_serde_payload_parity_live_contract_lane.sh\""
        ),
        "service API serde payload parity lane must remain excluded from ci-tools fast mode"
    );
    assert_contains_all(
        &ci_tools,
        &[
            "bash \"$ROOT_DIR/scripts/runtime/test_check_service_api_serde_payload_parity_live_policy.sh\"",
            "bash \"$ROOT_DIR/scripts/runtime/test_validate_service_api_serde_payload_parity_live_contract_lane.sh\"",
        ],
        "ci-tools service API serde payload parity command surface",
    );
    assert!(
        strategy_doc.contains(
            "service api serde payload parity contract-lane commands remain excluded from ci-fast-gate and ci-tools fast mode."
        ),
        "ci strategy doc missing service API serde payload parity exclusion marker"
    );
}

#[test]
fn spec_c13_service_api_validation_negative_matrix_ci_exclusion_policy_markers() {
    let fast_workflow = read_text(FAST_WORKFLOW);
    let ci_tools = read_text(CI_TOOLS_SCRIPT);
    let ci_tools_fast_mode = extract_fast_mode_block(&ci_tools);
    let strategy_doc = read_text(CI_STRATEGY_DOC);

    assert!(
        !fast_workflow.contains(
            "bash scripts/runtime/validate_service_api_validation_negative_matrix_live.sh --mode run"
        ),
        "service API validation negative-matrix lane must remain excluded from ci-fast-gate"
    );
    assert!(
        !ci_tools_fast_mode.contains(
            "bash \"$ROOT_DIR/scripts/runtime/test_validate_service_api_validation_negative_matrix_live_contract_lane.sh\""
        ),
        "service API validation negative-matrix lane must remain excluded from ci-tools fast mode"
    );
    assert_contains_all(
        &ci_tools,
        &[
            "bash \"$ROOT_DIR/scripts/runtime/test_check_service_api_validation_negative_matrix_live_policy.sh\"",
            "bash \"$ROOT_DIR/scripts/runtime/test_validate_service_api_validation_negative_matrix_live_contract_lane.sh\"",
        ],
        "ci-tools service API validation negative-matrix command surface",
    );
    assert!(
        strategy_doc.contains(
            "service api validation negative-matrix contract-lane commands remain excluded from ci-fast-gate and ci-tools fast mode."
        ),
        "ci strategy doc missing service API validation negative-matrix exclusion marker"
    );
}

#[test]
fn spec_c14_workflow_cache_policy_markers() {
    let fast_workflow = read_text(FAST_WORKFLOW);
    let deep_workflow = read_text(DEEP_WORKFLOW);
    for workflow in [&fast_workflow, &deep_workflow] {
        assert!(
            workflow.contains("shared-key: kamn-rust-ci-v1"),
            "workflow missing rust-cache shared key marker"
        );
        assert!(
            workflow.contains("save-if: ${{ github.ref == 'refs/heads/main' }}"),
            "workflow missing rust-cache save-if guard marker"
        );
    }
    assert!(
        deep_workflow.contains("run_invariant_harness.sh --mode deep --parallelism 2"),
        "ci-deep-validate workflow missing bounded invariant harness parallelism marker"
    );
    assert!(
        fast_workflow.contains("if: steps.scope.outputs.run_ci_tool_checks == 'true'"),
        "ci-fast-gate workflow missing run_ci_tool_checks gate marker"
    );
    assert!(
        fast_workflow.contains("KAMN_CI_TOOLS_FAST_MODE: 'true'"),
        "ci-fast-gate workflow missing fast-mode env marker for CI tools"
    );
}

#[test]
fn spec_c15_workflow_performance_policy_markers() {
    let fast_workflow = read_text(FAST_WORKFLOW);
    let deep_workflow = read_text(DEEP_WORKFLOW);
    assert_contains_all(
        &fast_workflow,
        &[
            "Generate performance smoke report",
            "generate_performance_smoke_report.sh --lane smoke --output-json performance-smoke-report.json",
            "Check performance thresholds (smoke)",
            "check_performance_thresholds.sh --lane smoke --report-json performance-smoke-report.json --profile-file .ci/performance-targets.env",
            "Generate fast-gate budget delta report",
            "generate_fast_gate_budget_delta_report.sh --current-json ci-budget-fast-gate.json --baseline-file .ci/fast-gate-budget-delta.env --output-json ci-budget-fast-gate-delta.json",
            "Check fast-gate budget delta thresholds",
            "check_fast_gate_budget_delta_threshold.sh --report-json ci-budget-fast-gate-delta.json --threshold-file .ci/fast-gate-budget-delta.env --waiver-file .ci/fast-gate-budget-delta-waiver.json",
            "Upload fast-gate budget delta telemetry",
            "ci-budget-fast-gate-delta-${{ github.run_id }}-${{ github.run_attempt }}",
        ],
        "ci-fast-gate workflow performance policy markers",
    );
    assert_contains_all(
        &deep_workflow,
        &[
            "Generate performance smoke report",
            "generate_performance_smoke_report.sh --lane deep --output-json performance-deep-report.json",
            "Check performance thresholds (deep)",
            "check_performance_thresholds.sh --lane deep --report-json performance-deep-report.json --profile-file .ci/performance-targets.env",
        ],
        "ci-deep-validate workflow performance policy markers",
    );
}

#[test]
fn spec_c16_input_mutation_coverage_guided_contract_lane_wrapper_parity() {
    let contract_lane =
        repo_path("scripts/runtime/run_input_mutation_coverage_guided_contract_lane.sh");
    let shared_contract =
        repo_path("scripts/runtime/input_mutation_coverage_guided_contract_lane_contract.sh");
    let manifest_file = repo_path(
        "scripts/framework/manifests/runtime_input_mutation_coverage_guided_contract_lane.json",
    );

    assert!(
        contract_lane.is_file() && shared_contract.is_file() && manifest_file.is_file(),
        "coverage-guided input mutation lane assets must exist"
    );

    let shared_contract_text = fs::read_to_string(&shared_contract)
        .expect("failed to read shared coverage-guided contract");
    assert_contains_all(
        &shared_contract_text,
        &[
            "unit_input_mutation_coverage_guided_envelope_seed_corpus_covers_boundary_classes",
            "unit_input_mutation_coverage_guided_did_seed_corpus_covers_boundary_classes",
            "minimal_failing_seed_prefix",
            "KAMN_RUNTIME_INPUT_MUTATION_COVERAGE_GUIDED_MAX_SECONDS",
        ],
        "coverage-guided shared contract markers",
    );

    let tmp = TempDir::new("coverage-guided-contract-lane");
    let report_file = tmp
        .path()
        .join("input-mutation-coverage-guided-contract-report.json");
    let lane_output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg(&contract_lane)
                .arg("--output-json")
                .arg(&report_file);
            command
        },
        "coverage-guided contract lane default target",
    );
    assert_success(&lane_output, "coverage-guided contract lane default target");
    let lane_text = output_text(&lane_output);
    assert_contains_all(
        &lane_text,
        &[
            "runtime input mutation coverage-guided contract lane tests passed.",
            "runtime_input_mutation_coverage_guided_contract_report=",
        ],
        "coverage-guided contract lane success markers",
    );

    let report_text = fs::read_to_string(&report_file)
        .expect("failed to read coverage-guided contract lane report JSON");
    assert_contains_all(
        &report_text,
        &[
            "\"schema_version\":\"kamn.runtime.input-mutation-coverage-guided-contract-report.v1\"",
            "\"status\":\"pass\"",
            "\"target\":\"all\"",
            "\"replay_schema_version\":\"kamn.runtime.input-mutation-coverage-guided-replay-metadata.v1\"",
            "\"replay_artifact_key\":\"input_mutation_coverage_guided_replay:v1\"",
            "\"minimizer\":\"minimal_failing_seed_prefix\"",
        ],
        "coverage-guided contract lane report markers",
    );

    let envelope_report = tmp
        .path()
        .join("input-mutation-coverage-guided-envelope-report.json");
    let envelope_output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg(&contract_lane)
                .arg("--target")
                .arg("envelope")
                .arg("--output-json")
                .arg(&envelope_report);
            command
        },
        "coverage-guided contract lane envelope target",
    );
    assert_success(
        &envelope_output,
        "coverage-guided contract lane envelope target",
    );

    let did_report = tmp
        .path()
        .join("input-mutation-coverage-guided-did-report.json");
    let did_output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg(&contract_lane)
                .arg("--target")
                .arg("did")
                .arg("--output-json")
                .arg(&did_report);
            command
        },
        "coverage-guided contract lane did target",
    );
    assert_success(&did_output, "coverage-guided contract lane did target");
}

#[test]
fn spec_c17_input_mutation_coverage_guided_deep_lane_wrapper_parity() {
    let fast_lane =
        repo_path("scripts/runtime/run_input_mutation_coverage_guided_contract_lane.sh");
    let deep_lane = repo_path("scripts/runtime/run_input_mutation_coverage_guided_deep_lane.sh");
    let deep_impl =
        repo_path("scripts/runtime/run_input_mutation_coverage_guided_deep_lane_impl.sh");
    let deep_manifest = repo_path(
        "scripts/framework/manifests/runtime_input_mutation_coverage_guided_deep_lane.json",
    );
    let dispatcher = repo_path("scripts/framework/run_non_kolme_contract_lane_dispatch.sh");

    assert!(
        fast_lane.is_file()
            && deep_lane.is_file()
            && deep_impl.is_file()
            && deep_manifest.is_file()
            && dispatcher.is_file(),
        "coverage-guided deep lane assets must exist"
    );

    let deep_impl_text =
        fs::read_to_string(&deep_impl).expect("failed to read coverage-guided deep lane impl");
    assert_contains_all(
        &deep_impl_text,
        &[
            "run_input_mutation_coverage_guided_contract_lane.sh",
            "performance_input_mutation_coverage_guided_deep_lane_stress -- --ignored",
            "KAMN_RUNTIME_INPUT_MUTATION_COVERAGE_GUIDED_DEEP_MAX_SECONDS",
        ],
        "coverage-guided deep lane implementation markers",
    );

    let resolved_manifest_output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg(&dispatcher)
                .arg("--lane-wrapper")
                .arg("run_input_mutation_coverage_guided_deep_lane.sh")
                .arg("--resolve-manifest-path");
            command
        },
        "coverage-guided deep lane manifest resolution",
    );
    assert_success(
        &resolved_manifest_output,
        "coverage-guided deep lane manifest resolution",
    );
    let resolved_manifest_path = output_text(&resolved_manifest_output).trim().to_owned();
    let expected_manifest_path = deep_manifest
        .canonicalize()
        .expect("failed to canonicalize expected deep manifest path");
    assert_eq!(
        resolved_manifest_path,
        expected_manifest_path.to_string_lossy(),
        "coverage-guided deep lane wrapper must resolve runtime deep manifest via dispatcher"
    );

    let deep_manifest_text =
        fs::read_to_string(&deep_manifest).expect("failed to read coverage-guided deep manifest");
    assert!(
        deep_manifest_text.contains("run_input_mutation_coverage_guided_deep_lane_impl.sh"),
        "coverage-guided deep manifest must dispatch deep-lane implementation"
    );

    let lane_output = run_command(
        {
            let mut command = Command::new("bash");
            command.arg(&deep_lane);
            command
        },
        "coverage-guided deep lane execution",
    );
    assert_success(&lane_output, "coverage-guided deep lane execution");
    assert!(
        output_text(&lane_output)
            .contains("runtime input mutation coverage-guided deep lane tests passed."),
        "coverage-guided deep lane must emit deterministic success marker"
    );
}

#[test]
fn spec_c18_live_network_smoke_contract_lane_wrapper_parity() {
    let contract_lane = repo_path("scripts/runtime/run_live_network_smoke_contract_lane.sh");
    let shared_contract = repo_path("scripts/runtime/live_network_smoke_contract_lane_contract.sh");
    let manifest_file =
        repo_path("scripts/framework/manifests/runtime_live_network_smoke_contract_lane.json");
    let dispatcher = repo_path("scripts/framework/run_non_kolme_contract_lane_dispatch.sh");
    let smoke_runner = repo_path("scripts/runtime/run_live_network_smoke_lane.sh");

    assert!(
        contract_lane.is_file()
            && shared_contract.is_file()
            && manifest_file.is_file()
            && dispatcher.is_file()
            && smoke_runner.is_file(),
        "live-network smoke lane assets must exist"
    );

    let shared_contract_text =
        fs::read_to_string(&shared_contract).expect("failed to read live-network shared contract");
    assert!(
        shared_contract_text.contains("run_live_network_smoke_lane.sh"),
        "live-network shared contract must execute smoke runner"
    );

    let lane_output = run_command(
        {
            let mut command = Command::new("bash");
            command.arg(&contract_lane);
            command
        },
        "live-network smoke contract lane execution",
    );
    assert_success(&lane_output, "live-network smoke contract lane execution");
    assert!(
        output_text(&lane_output).contains("live-network smoke contract lane tests passed."),
        "live-network smoke contract lane must emit deterministic success marker"
    );

    let resolved_manifest_output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg(&dispatcher)
                .arg("--lane-wrapper")
                .arg("run_live_network_smoke_contract_lane.sh")
                .arg("--resolve-manifest-path");
            command
        },
        "live-network smoke manifest resolution",
    );
    assert_success(
        &resolved_manifest_output,
        "live-network smoke manifest resolution",
    );
    let resolved_manifest_path = output_text(&resolved_manifest_output).trim().to_owned();
    let expected_manifest_path = manifest_file
        .canonicalize()
        .expect("failed to canonicalize expected live-network manifest path");
    assert_eq!(
        resolved_manifest_path,
        expected_manifest_path.to_string_lossy(),
        "live-network smoke wrapper must resolve runtime manifest via dispatcher"
    );
}

#[test]
fn spec_c19_async_runtime_live_validation_lane_parity() {
    let validation_script = repo_path("scripts/runtime/validate_async_runtime_live.sh");
    assert!(
        validation_script.is_file(),
        "async runtime live validation script must exist"
    );

    let tmp = TempDir::new("async-runtime-live");
    let report_file = tmp.path().join("async-runtime-live-report.json");
    let lane_output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg(&validation_script)
                .arg("--output-json")
                .arg(&report_file);
            command
        },
        "async runtime live validation lane",
    );
    assert_success(&lane_output, "async runtime live validation lane");
    let lane_text = output_text(&lane_output);
    assert_contains_all(
        &lane_text,
        &[
            "status=pass",
            "final_decision=GO",
            "runtime_entrypoint=tokio-main",
            "failure_case_status=verified",
        ],
        "async runtime live validation lane markers",
    );

    let report_text = fs::read_to_string(&report_file)
        .expect("failed to read async runtime live validation report JSON");
    assert_contains_all(
        &report_text,
        &[
            "\"schema_version\": \"kamn.runtime.async-runtime-live-validation.v1\"",
            "\"status\": \"pass\"",
            "\"final_decision\": \"GO\"",
            "\"runtime_entrypoint\": \"tokio-main\"",
            "\"failure_case_status\": \"verified\"",
        ],
        "async runtime live validation report markers",
    );
}

#[test]
fn spec_c20_libp2p_process_isolated_harness_validation_parity() {
    let validation_script =
        repo_path("scripts/runtime/validate_libp2p_process_isolated_harness.sh");
    assert!(
        validation_script.is_file(),
        "libp2p process-isolated harness validation script must exist"
    );

    let tmp = TempDir::new("libp2p-process-isolated");
    let report_file = tmp
        .path()
        .join("libp2p-process-isolated-harness-report.json");
    let dry_run_output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg(&validation_script)
                .arg("--mode")
                .arg("dry-run")
                .arg("--max-seconds")
                .arg("120")
                .arg("--output-json")
                .arg(&report_file);
            command
        },
        "libp2p process-isolated harness dry-run",
    );
    assert_success(&dry_run_output, "libp2p process-isolated harness dry-run");
    let dry_run_text = output_text(&dry_run_output);
    assert_contains_all(
        &dry_run_text,
        &[
            "status=pass",
            "final_decision=GO",
            "two_node_startup_status=verified",
            "three_node_startup_status=verified",
            "partition_rejoin_status=verified",
            "publish_drop_recovery_status=verified",
            "runtime_transport_mode=libp2p_process_isolated_convergence",
        ],
        "libp2p process-isolated harness dry-run markers",
    );

    let report_text = fs::read_to_string(&report_file)
        .expect("failed to read libp2p process-isolated harness report JSON");
    assert_contains_all(
        &report_text,
        &[
            "\"schema_version\": \"kamn.runtime.libp2p-process-isolated-harness-report.v1\"",
            "\"status\": \"pass\"",
            "\"final_decision\": \"GO\"",
            "\"runtime_transport_mode\": \"libp2p_process_isolated_convergence\"",
        ],
        "libp2p process-isolated harness report markers",
    );

    let evidence_file = extract_json_string_field(&report_text, "process_harness_evidence_file")
        .expect("missing process_harness_evidence_file marker in harness report");
    assert!(
        Path::new(&evidence_file).is_file(),
        "process_harness_evidence_file must exist on disk"
    );

    let run_without_opt_in_output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg(&validation_script)
                .arg("--mode")
                .arg("run")
                .arg("--max-seconds")
                .arg("120");
            command
        },
        "libp2p process-isolated harness run mode without opt-in",
    );
    assert_failure(
        &run_without_opt_in_output,
        "libp2p process-isolated harness run mode without opt-in",
    );
    assert!(
        output_text(&run_without_opt_in_output)
            .contains("KAMN_LIBP2P_PROCESS_ISOLATED_HARNESS_OPT_IN=1"),
        "run mode without opt-in must emit deterministic opt-in marker"
    );
}
