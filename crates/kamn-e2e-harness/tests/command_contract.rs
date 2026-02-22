use std::path::PathBuf;

use kamn_e2e_harness::{
    all_orchestration_phases, all_phase_result_statuses, execute_run_contract,
    execute_verify_contract, parse_command_args, parse_scenario_csv, HarnessCommand,
    RunCommandConfig, VerifyCommandConfig,
};

fn temp_path(name: &str) -> PathBuf {
    let pid = std::process::id();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock should be monotonic")
        .as_nanos();
    std::env::temp_dir().join(format!("kamn-e2e-harness-{pid}-{nanos}-{name}"))
}

#[test]
fn spec_c01_parser_accepts_run_with_required_flags() {
    let parsed = parse_command_args([
        "run",
        "--mode",
        "sdk-direct",
        "--evidence-dir",
        "/tmp/evidence",
        "--scenarios",
        "S-01,S-02",
    ])
    .expect("run command should parse");
    let expected = HarnessCommand::Run(RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned(), "S-02".to_owned()],
    });
    assert_eq!(parsed, expected);
}

#[test]
fn spec_c02_parser_accepts_verify_with_required_flags() {
    let parsed = parse_command_args([
        "verify",
        "--evidence-dir",
        "/tmp/evidence",
        "--kolme-chain-dump",
        "/tmp/evidence/kolme_chain_dump.json",
        "--output",
        "/tmp/verification-report.json",
    ])
    .expect("verify command should parse");
    let expected = HarnessCommand::Verify(VerifyCommandConfig {
        evidence_dir: "/tmp/evidence".to_owned(),
        kolme_chain_dump: "/tmp/evidence/kolme_chain_dump.json".to_owned(),
        output: "/tmp/verification-report.json".to_owned(),
    });
    assert_eq!(parsed, expected);
}

#[test]
fn spec_c03_parser_rejects_missing_required_flag_values() {
    let err = parse_command_args(["run", "--mode", "sdk-direct", "--evidence-dir"])
        .expect_err("missing evidence-dir value should fail");
    assert!(err.contains("missing value for --evidence-dir"));
}

#[test]
fn spec_c04_scenario_csv_parser_preserves_deterministic_order() {
    let selected = parse_scenario_csv("S-02,S-01,S-15").expect("selection should parse");
    assert_eq!(selected, vec!["S-02", "S-01", "S-15"]);
}

#[test]
fn spec_c05_scenario_csv_parser_rejects_unknown_id() {
    let err = parse_scenario_csv("S-01,S-99").expect_err("unknown scenario should fail");
    assert!(err.contains("unknown scenario id: S-99"));
}

#[test]
fn spec_c06_run_command_output_contains_selected_mode_and_count_markers() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned(), "S-02".to_owned(), "S-15".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains("\"command\":\"run\""));
    assert!(output.contains("\"mode\":\"sdk-direct\""));
    assert!(output.contains("\"evidence_dir\":\"/tmp/evidence\""));
    assert!(output.contains("\"scenario_count\":3"));
}

#[test]
fn spec_c07_verify_command_writes_deterministic_report_output() {
    let evidence_dir = temp_path("evidence");
    let output_path = temp_path("report.json");
    let chain_dump_path = temp_path("kolme_chain_dump.json");
    std::fs::create_dir_all(&evidence_dir).expect("evidence dir should be created");
    std::fs::write(
        evidence_dir.join("manifest.json"),
        r#"{"schema_version":"kamn.e2e.evidence-manifest.v3","run_id":"e2e-run","started_at":"2026-02-21T14:30:52Z","completed_at":"2026-02-21T14:35:12Z","duration_seconds":260,"execution_mode":"sdk-direct","infrastructure":{"kolme_version":"0.x.y","kamn_version":"0.1.0","kamn_commit":"49efe252","kamn_agent_lib_version":"0.1.0","agent_runtime":"sdk-direct","node_count":3,"agent_count":3,"storage_backend":"sqlite+postgres"},"scenarios":[],"summary":{"total_scenarios":15,"passed":13,"failed":1,"skipped":1,"kolme_blocks_produced":47,"messages_exchanged":128,"proofs_anchored":47,"proofs_verified":47}}"#,
    )
    .expect("manifest should be written");
    std::fs::write(&chain_dump_path, "{}").expect("chain dump should be written");

    let config = VerifyCommandConfig {
        evidence_dir: evidence_dir.display().to_string(),
        kolme_chain_dump: chain_dump_path.display().to_string(),
        output: output_path.display().to_string(),
    };
    let first = execute_verify_contract(&config).expect("verify should succeed");
    let second = execute_verify_contract(&config).expect("verify should be deterministic");
    assert_eq!(first, second);

    let written =
        std::fs::read_to_string(&output_path).expect("verification report should be written");
    assert_eq!(written, first);
    assert!(written.contains("\"schema_check\""));
    assert!(written.contains("\"proof_check\""));
    assert!(written.contains("\"chain_check\""));
    assert!(written.contains("\"content_check\""));

    let _ = std::fs::remove_file(output_path);
    let _ = std::fs::remove_file(chain_dump_path);
    let _ = std::fs::remove_file(evidence_dir.join("manifest.json"));
    let _ = std::fs::remove_dir(evidence_dir);
}

#[test]
fn spec_c08_phase_inventory_contains_prd_canonical_order() {
    let phases = all_orchestration_phases();
    let labels: Vec<&str> = phases.iter().map(|phase| phase.as_str()).collect();
    assert_eq!(
        labels,
        vec![
            "INFRA_UP",
            "AGENT_DEPLOY",
            "SCENARIO_RUN",
            "EVIDENCE",
            "TEARDOWN"
        ]
    );
}

#[test]
fn spec_c09_run_output_contains_phase_progression_markers() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned(), "S-02".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains("\"phase_count\":5"));
    assert!(output.contains("\"phases\":[\"INFRA_UP\""));
    assert!(output.contains("\"AGENT_DEPLOY\""));
    assert!(output.contains("\"SCENARIO_RUN\""));
    assert!(output.contains("\"EVIDENCE\""));
    assert!(output.contains("\"TEARDOWN\"]"));
}

#[test]
fn spec_c10_phase_result_status_inventory_is_canonical() {
    let statuses = all_phase_result_statuses();
    let labels: Vec<&str> = statuses.iter().map(|status| status.as_str()).collect();
    assert_eq!(labels, vec!["PASS", "FAIL", "SKIP"]);
}

#[test]
fn spec_c11_run_output_contains_phase_results_required_fields() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned(), "S-02".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains("\"phase_results\":[{"));
    assert!(output.contains("\"phase\":\"INFRA_UP\""));
    assert!(output.contains("\"status\":\"PASS\""));
    assert!(output.contains("\"started_at\""));
    assert!(output.contains("\"completed_at\""));
    assert!(output.contains("\"details\""));
}

#[test]
fn spec_c12_run_output_contains_infra_and_agent_deploy_placeholders() {
    let config = RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    };
    let output = execute_run_contract(&config).expect("run output should render");
    assert!(output.contains("\"phase\":\"INFRA_UP\""));
    assert!(output.contains("\"phase\":\"AGENT_DEPLOY\""));
    assert!(output.contains("deterministic placeholder"));
}
