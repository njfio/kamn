use super::super::super::support::*;

#[test]
fn spec_c19_async_runtime_live_validation_lane_parity() {
    let validation_script = repo_path("scripts/runtime/validate_async_runtime_live.sh");
    assert!(
        validation_script.is_file(),
        "async runtime live validation script must exist"
    );

    let tmp = TempDir::new("async-runtime-live");
    let report_file = tmp.path().join("async-runtime-live-report.json");
    let lane_output =
        run_validation_with_report(&validation_script, &report_file, &["--max-seconds", "600"]);
    assert_async_runtime_output(&lane_output);
    assert_async_runtime_report(&report_file);
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
    let dry_run_output = run_validation_with_report(
        &validation_script,
        &report_file,
        &["--mode", "dry-run", "--max-seconds", "120"],
    );
    assert_libp2p_dry_run_output(&dry_run_output);
    assert_libp2p_report(&report_file);
    assert_libp2p_run_mode_requires_opt_in(&validation_script);
}

fn run_validation_with_report(
    script: &Path,
    report_file: &Path,
    args: &[&str],
) -> std::process::Output {
    run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg(script)
                .args(args)
                .arg("--output-json")
                .arg(report_file);
            command
        },
        &format!("{} execution", script.display()),
    )
}

fn assert_async_runtime_output(output: &std::process::Output) {
    assert_success(output, "async runtime live validation lane");
    assert_contains_all(
        &output_text(output),
        &[
            "status=pass",
            "final_decision=GO",
            "runtime_entrypoint=tokio-main",
            "failure_case_status=verified",
        ],
        "async runtime live validation lane markers",
    );
}

fn assert_async_runtime_report(report_file: &Path) {
    let report_text = fs::read_to_string(report_file)
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

fn assert_libp2p_dry_run_output(output: &std::process::Output) {
    assert_success(output, "libp2p process-isolated harness dry-run");
    assert_contains_all(
        &output_text(output),
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
}

fn assert_libp2p_report(report_file: &Path) {
    let report_text = fs::read_to_string(report_file)
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
}

fn assert_libp2p_run_mode_requires_opt_in(validation_script: &Path) {
    let output = run_command(
        {
            let mut command = Command::new("bash");
            command
                .arg(validation_script)
                .arg("--mode")
                .arg("run")
                .arg("--max-seconds")
                .arg("120");
            command
        },
        "libp2p process-isolated harness run mode without opt-in",
    );
    assert_failure(
        &output,
        "libp2p process-isolated harness run mode without opt-in",
    );
    assert!(
        output_text(&output).contains("KAMN_LIBP2P_PROCESS_ISOLATED_HARNESS_OPT_IN=1"),
        "run mode without opt-in must emit deterministic opt-in marker"
    );
}
