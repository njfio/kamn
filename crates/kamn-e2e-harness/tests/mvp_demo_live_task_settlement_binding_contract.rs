use kamn_e2e_harness::{
    execute_mvp_demo_contract, execute_verify_mvp_demo_contract, VerifyMvpDemoCommandConfig,
};
use std::path::{Path, PathBuf};

#[path = "support/artifact_digest.rs"]
#[allow(dead_code)]
mod artifact_digest;
#[path = "support/mvp_demo_command.rs"]
mod mvp_demo_command;

#[test]
fn spec_c00_bound_demo_rejects_altered_live_task_source() {
    let root = temp_root("altered-source");
    let config = mvp_demo_command::devnet_required_demo_config(&root);
    let handoff = &config
        .live_task_evidence
        .as_ref()
        .expect("binding configured")
        .handoff;
    let altered = std::fs::read_to_string(handoff)
        .expect("handoff should exist")
        .replace("task-local-bound-7086", "task-local-forged-7086");
    std::fs::write(handoff, altered).expect("handoff tamper should be written");

    let err = execute_mvp_demo_contract(&config).expect_err("altered task source must fail");
    assert!(err.contains("artifact digest mismatch"), "{err}");
}

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
        assert!(
            report.contains(marker),
            "missing bound report marker: {marker}"
        );
    }
    assert!(!report.contains("mvp-three-agent-"));
    verify_latest(&root).expect("bound report and artifacts should verify");
}

#[test]
fn spec_c02_unbound_devnet_settlement_does_not_claim_three_agent_proof() {
    let root = temp_root("unbound-settlement");
    let config = mvp_demo_command::devnet_required_without_task_binding(&root);
    let report =
        execute_mvp_demo_contract(&config).expect("standalone devnet settlement should pass");

    assert!(report.contains(r#""id":"devnet_settlement_asset_movement""#));
    assert!(!report.contains(r#""id":"three_agent_escrow_verification""#));
    assert!(!report.contains(r#""live_task_settlement_binding":""#));
    let markdown = std::fs::read_to_string(root.join("latest/proof/report.md"))
        .expect("standalone settlement markdown should exist");
    assert!(!markdown.contains("Three-Agent View Boundary"));
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

#[test]
fn spec_c04_live_service_claim_requires_funding_request_artifact() {
    let root = temp_root("missing-funding-request");
    execute_mvp_demo_contract(&mvp_demo_command::devnet_required_demo_config(&root))
        .expect("bound devnet demo should pass before claim tamper");
    relabel_report_as_live_service(&root, None);

    let err = verify_latest(&root).expect_err("live service claim without request must fail");
    assert!(err.contains("devnet escrow funding request"), "{err}");
}

#[test]
fn spec_c05_live_service_claim_requires_request_derived_escrow_id() {
    let root = temp_root("mismatched-request-escrow");
    execute_mvp_demo_contract(&mvp_demo_command::devnet_required_demo_config(&root))
        .expect("bound devnet demo should pass before claim tamper");
    let request = only_run_dir(&root).join("proof/devnet-escrow-funding-request.json");
    std::fs::write(
        &request,
        r#"{"schema_version":"kamn.mvp.devnet-settlement.v2","run_id":"forged","task_id":"task-local-bound-7086","task_binding_digest":"sha256:forged"}"#,
    )
    .expect("funding request should be written");
    relabel_report_as_live_service(&root, Some(&request));

    let err = verify_latest(&root).expect_err("request-derived escrow mismatch must fail");
    assert!(err.contains("devnet escrow funding request"), "{err}");
}

#[test]
fn spec_c06_bound_demo_accepts_transaction_aware_handoff_v2() {
    let root = temp_root("handoff-v2");
    let mut config = mvp_demo_command::devnet_required_demo_config(&root);
    config.live_task_evidence = Some(mvp_demo_command::live_task_evidence::write_v2(
        &root.join("handoff-v2"),
    ));

    execute_mvp_demo_contract(&config).expect("v2 handoff should bind to the demo");
}

fn relabel_report_as_live_service(root: &Path, request: Option<&Path>) {
    let report_path = root.join("latest/proof/report.json");
    let mut report = std::fs::read_to_string(&report_path)
        .expect("latest report should exist")
        .replace(
            r#""execution_surface":"command-override""#,
            r#""execution_surface":"live-service-api""#,
        );
    if let Some(path) = request {
        report = report.replace(
            r#""live_task_settlement_binding":"#,
            format!(
                r#""devnet_escrow_funding_request":"{}","live_task_settlement_binding":"#,
                path.display()
            )
            .as_str(),
        );
    }
    std::fs::write(report_path, report).expect("latest report tamper should be written");
    relabel_settlement_artifacts(root);
}

fn relabel_settlement_artifacts(root: &Path) {
    let proof = only_run_dir(root).join("proof");
    let evidence_path = proof.join("settlement-evidence.json");
    let evidence = std::fs::read_to_string(&evidence_path)
        .expect("settlement evidence")
        .replace("command-override", "live-service-api");
    std::fs::write(
        evidence_path,
        artifact_digest::with_digest(evidence, "evidence_digest"),
    )
    .expect("relabeled settlement evidence");
    let log_path = proof.join("devnet-settlement-output.txt");
    let log = std::fs::read_to_string(&log_path)
        .expect("settlement log")
        .replace("command-override", "live-service-api");
    std::fs::write(log_path, log).expect("relabeled settlement log");
}

fn verify_latest(root: &Path) -> Result<String, String> {
    execute_verify_mvp_demo_contract(&VerifyMvpDemoCommandConfig {
        report: root.join("latest/proof/report.json").display().to_string(),
        agent_harness_evidence_path: None,
        pi_transaction_actor_paths: None,
    })
}

fn only_run_dir(root: &Path) -> PathBuf {
    std::fs::read_dir(root)
        .expect("output root should exist")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| {
            path.is_dir() && path.file_name().and_then(|name| name.to_str()) != Some("latest")
        })
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
