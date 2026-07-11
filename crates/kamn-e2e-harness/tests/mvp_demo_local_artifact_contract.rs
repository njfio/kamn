use kamn_e2e_harness::{execute_verify_mvp_demo_contract, VerifyMvpDemoCommandConfig};
use std::path::{Path, PathBuf};

#[path = "support/mvp_local_artifacts.rs"]
mod mvp_local_artifacts;

#[test]
fn spec_c01_command_rejects_missing_local_artifact() {
    let root = temp_root("missing-artifact");
    let report = write_report(&root, local_only_report(&root));

    let err = execute_verify_mvp_demo_contract(&config(report.as_path()))
        .expect_err("missing local artifacts must fail verification");

    assert!(err.contains("localhost signed demo artifact"));
}

#[test]
fn spec_c02_command_rejects_tampered_service_api_log() {
    let root = temp_root("tampered-service-log");
    write_valid_local_artifacts(&root);
    mvp_local_artifacts::write_file(
        root.join("proof/service-api-vertical-slice-output.txt")
            .as_path(),
        "--- stdout ---\nmissing real service api test marker\n",
    );
    let report = write_report(&root, local_only_report(&root));

    let err = execute_verify_mvp_demo_contract(&config(report.as_path()))
        .expect_err("tampered service API proof log must fail verification");

    assert!(err.contains("service API vertical slice"));
}

#[test]
fn spec_c03_command_accepts_valid_local_only_artifacts() {
    let root = temp_root("valid-local");
    write_valid_local_artifacts(&root);
    let report = write_report(&root, local_only_report(&root));

    let output = execute_verify_mvp_demo_contract(&config(report.as_path()))
        .expect("valid local-only artifact bundle should verify");

    assert!(output.contains("\"status\":\"PASS\""));
}

fn temp_root(stem: &str) -> PathBuf {
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_millis();
    std::env::temp_dir().join(format!("kamn-7058-{stem}-{}-{millis}", std::process::id()))
}

fn config(report: &Path) -> VerifyMvpDemoCommandConfig {
    VerifyMvpDemoCommandConfig {
        report: report.display().to_string(),
        agent_harness_evidence_path: None,
        pi_transaction_actor_paths: None,
    }
}

fn write_report(root: &Path, report: String) -> PathBuf {
    let path = root.join("proof/report.json");
    mvp_local_artifacts::write_file(path.as_path(), report.as_str());
    path
}

fn write_valid_local_artifacts(root: &Path) {
    mvp_local_artifacts::write_valid_local_artifacts(root);
}

fn local_only_report(root: &Path) -> String {
    format!(
        r#"{{"schema_version":"kamn.mvp.demo.proof-report.v1","run_id":"demo-local-artifacts","status":"GO","devnet_mode":"optional","artifacts":{},"claim_matrix":[{},{}],"no_go":{{"active":false,"reason":""}}}}"#,
        mvp_local_artifacts::artifacts_json(root, None),
        local_claims(),
        roadmap_claim()
    )
}

fn local_claims() -> &'static str {
    r#"{"id":"local_runtime_startup","label":"real","required":true,"status":"PASS","summary":"local runtime"},{"id":"authenticated_agent_identities","label":"local-only","required":true,"status":"PASS","summary":"agent identities"},{"id":"signed_message_or_task_flow","label":"local-only","required":true,"status":"PASS","summary":"message flow"},{"id":"durable_state_written","label":"local-only","required":true,"status":"PASS","summary":"durable state"},{"id":"relay_projection_visible","label":"local-only","required":true,"status":"PASS","summary":"relay projection"},{"id":"websocket_event_visibility","label":"local-only","required":true,"status":"PASS","summary":"websocket events"},{"id":"audit_proof_export","label":"local-only","required":true,"status":"PASS","summary":"audit export"}"#
}

fn roadmap_claim() -> &'static str {
    r#"{"id":"production_readiness","label":"roadmap","required":false,"status":"NOT_CLAIMED","summary":"production readiness is not claimed"}"#
}
