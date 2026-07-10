use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use kamn_e2e_harness::{
    execute_mvp_demo_contract, execute_verify_mvp_demo_contract, parse_command_args,
    HarnessCommand, MvpDemoCommandConfig, VerifyMvpDemoCommandConfig,
};

#[path = "support/mvp_demo_command.rs"]
mod mvp_demo_command;

#[test]
fn spec_c01_parser_accepts_demo_mvp_with_output_root() {
    let parsed = parse_command_args(["demo-mvp", "--output-root", "/tmp/kamn-demo"])
        .expect("demo-mvp command should parse");
    let expected = HarnessCommand::DemoMvp(MvpDemoCommandConfig {
        output_root: "/tmp/kamn-demo".to_owned(),
        devnet_mode: "optional".to_owned(),
        solana_rpc_url: None,
        devnet_settlement_command: None,
        localhost_signed_demo_command: None,
        service_api_vertical_slice_command: None,
        service_api_websocket_command: None,
        agent_harness_evidence_path: None,
    });
    assert_eq!(parsed, expected);
}

#[test]
fn spec_c02_parser_accepts_verify_mvp_demo_with_report() {
    let parsed = parse_command_args(["verify-mvp-demo", "--report", "/tmp/report.json"])
        .expect("verify-mvp-demo command should parse");
    let expected = HarnessCommand::VerifyMvpDemo(VerifyMvpDemoCommandConfig {
        report: "/tmp/report.json".to_owned(),
    });
    assert_eq!(parsed, expected);
}

#[test]
fn spec_c07_parser_accepts_verify_mvp_demo_with_agent_harness_evidence() {
    let parsed = parse_command_args([
        "verify-mvp-demo",
        "--agent-harness-evidence",
        "/tmp/pi-evidence.json",
        "--report",
        "/tmp/report.json",
    ])
    .expect("verify-mvp-demo should accept direct Pi evidence");

    let debug = format!("{parsed:?}");
    assert!(debug.contains("/tmp/report.json"));
    assert!(debug.contains("/tmp/pi-evidence.json"));
}

#[test]
fn spec_c03_makefile_wires_demo_mvp_to_harness_command() {
    let output = make_dry_run("demo-mvp");
    assert!(
        output.contains("cargo run -p kamn-e2e-harness -- demo-mvp"),
        "make demo-mvp should call the Rust-owned harness command. output:\n{output}"
    );
}

#[test]
fn spec_c04_demo_mvp_creates_run_directory_and_latest_report_paths() {
    let temp = temp_dir("mvp-demo-command");
    let config = mvp_demo_command::local_demo_config(&temp);
    execute_mvp_demo_contract(&config).expect("demo-mvp should generate local proof artifacts");
    assert!(temp.join("latest/proof/report.json").is_file());
    assert!(temp.join("latest/proof/report.md").is_file());
    assert!(
        run_directories(&temp).len() == 1,
        "demo should create exactly one concrete run directory"
    );
    let _ = std::fs::remove_dir_all(&temp);
}

#[test]
fn spec_c05_verify_mvp_demo_command_accepts_generated_report() {
    let temp = temp_dir("mvp-demo-verify");
    let demo_config = mvp_demo_command::local_demo_config(&temp);
    execute_mvp_demo_contract(&demo_config).expect("demo should generate report");
    let verify_config = VerifyMvpDemoCommandConfig {
        report: temp.join("latest/proof/report.json").display().to_string(),
    };
    let output = execute_verify_mvp_demo_contract(&verify_config)
        .expect("verify-mvp-demo should accept generated report");
    assert!(output.contains("\"status\":\"PASS\""));
    let _ = std::fs::remove_dir_all(&temp);
}

#[test]
fn spec_c06_demo_mvp_devnet_required_records_settlement_evidence() {
    let temp = temp_dir("mvp-demo-devnet-settlement");
    let config = mvp_demo_command::devnet_required_demo_config(&temp);
    let report = execute_mvp_demo_contract(&config)
        .expect("devnet-required demo should accept real settlement evidence");

    assert!(report.contains(r#""status":"GO""#));
    assert!(report.contains(r#""id":"devnet_settlement_asset_movement""#));
    assert!(report.contains(r#""label":"devnet-backed""#));
    for marker in devnet_settlement_markers() {
        assert!(report.contains(marker), "missing report marker: {marker}");
    }
    for marker in three_agent_digest_markers() {
        assert!(report.contains(marker), "missing report marker: {marker}");
    }
    let _ = std::fs::remove_dir_all(&temp);
}

fn devnet_settlement_markers() -> [&'static str; 6] {
    [
        r#""settlement_tx_signature":"devnet-signature-111""#,
        r#""persisted_settlement_tx_signature":"devnet-signature-111""#,
        r#""payer_balance_before":2500000000"#,
        r#""payer_balance_after":2498995000"#,
        r#""recipient_balance_before":2500000000"#,
        r#""recipient_balance_after":2501000000"#,
    ]
}

fn three_agent_digest_markers() -> [&'static str; 4] {
    [
        r#""three_agent_transcript_digest":"sha256:"#,
        r#""agent_a_view_digest":"sha256:"#,
        r#""agent_b_view_digest":"sha256:"#,
        r#""agent_c_verifier_view_digest":"sha256:"#,
    ]
}

fn make_dry_run(target: &str) -> String {
    let output = Command::new("make")
        .arg("-n")
        .arg(target)
        .current_dir(repo_root())
        .output()
        .expect("make dry run should execute");
    assert!(
        output.status.success(),
        "make -n {target} failed:\n{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn run_directories(root: &Path) -> Vec<PathBuf> {
    std::fs::read_dir(root)
        .expect("demo output root should be readable")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .filter(|path| path.file_name().and_then(|name| name.to_str()) != Some("latest"))
        .collect()
}

fn temp_dir(prefix: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("kamn-{prefix}-{}-{suffix}", std::process::id()))
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
