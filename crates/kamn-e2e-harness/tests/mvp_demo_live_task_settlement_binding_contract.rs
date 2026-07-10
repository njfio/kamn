use kamn_e2e_harness::{
    execute_mvp_demo_contract, execute_verify_mvp_demo_contract, VerifyMvpDemoCommandConfig,
};
use std::path::{Path, PathBuf};

#[path = "support/mvp_demo_command.rs"]
mod mvp_demo_command;

#[test]
fn spec_c01_bound_devnet_report_uses_live_task_and_actual_escrow() {
    let root = temp_root("bound-report");
    let report = execute_mvp_demo_contract(&mvp_demo_command::devnet_required_demo_config(&root))
        .expect("bound devnet demo should pass");

    for marker in [
        r#""live_task_settlement_binding":""#,
        r#""live_task_settlement_binding_digest":"sha256:"#,
        r#""transaction_id":"task-local-bound-7086""#,
        r#""escrow_id":"escrow-local-bound-7086""#,
        r#""task_binding_digest":"sha256:"#,
    ] {
        assert!(report.contains(marker), "missing bound report marker: {marker}");
    }
    assert!(!report.contains("mvp-three-agent-"));
    verify_latest(&root).expect("bound report and artifacts should verify");
}

#[test]
fn spec_c02_unbound_devnet_settlement_does_not_claim_three_agent_proof() {
    let root = temp_root("unbound-settlement");
    let config = mvp_demo_command::devnet_required_without_task_binding(&root);
    let report = execute_mvp_demo_contract(&config).expect("standalone devnet settlement should pass");

    assert!(report.contains(r#""id":"devnet_settlement_asset_movement""#));
    assert!(!report.contains(r#""id":"three_agent_escrow_verification""#));
    assert!(!report.contains(r#""live_task_settlement_binding":""#));
}

#[test]
fn spec_c03_verifier_rejects_tampered_task_binding_artifact() {
    let root = temp_root("tampered-binding");
    execute_mvp_demo_contract(&mvp_demo_command::devnet_required_demo_config(&root))
        .expect("bound devnet demo should pass before tamper");
    let binding = only_run_dir(&root).join("proof/live-task-settlement-binding.json");
    let tampered = std::fs::read_to_string(&binding)
        .expect("binding should exist")
        .replace("task-local-bound-7086", "task-local-forged-7086");
    std::fs::write(binding, tampered).expect("binding tamper should be written");

    let err = verify_latest(&root).expect_err("tampered task binding must fail");
    assert!(err.contains("live task settlement binding"), "{err}");
}

fn verify_latest(root: &Path) -> Result<String, String> {
    execute_verify_mvp_demo_contract(&VerifyMvpDemoCommandConfig {
        report: root.join("latest/proof/report.json").display().to_string(),
        agent_harness_evidence_path: None,
    })
}

fn only_run_dir(root: &Path) -> PathBuf {
    std::fs::read_dir(root)
        .expect("output root should exist")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| path.is_dir() && path.file_name().and_then(|name| name.to_str()) != Some("latest"))
        .expect("one run directory should exist")
}

fn temp_root(stem: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "kamn-7086-{stem}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock should be valid")
            .as_nanos()
    ))
}
