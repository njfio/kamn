use kamn_e2e_harness::{
    execute_mvp_demo_contract, execute_verify_mvp_demo_contract, VerifyMvpDemoCommandConfig,
};
use std::path::Path;

#[allow(dead_code)]
#[path = "support/artifact_digest.rs"]
mod artifact_digest;

#[path = "support/mvp_demo_command.rs"]
mod mvp_demo_command;

#[path = "support/mvp_local_artifacts.rs"]
mod mvp_local_artifacts;

#[path = "support/three_agent_receipts.rs"]
mod three_agent_receipts;

#[allow(dead_code)]
#[path = "support/three_agent_view_artifacts.rs"]
mod three_agent_view_artifacts;

use three_agent_receipts::ReceiptOverrides;

#[test]
fn spec_c01_command_rejects_missing_actor_observation_receipts() {
    let root = three_agent_receipts::base_fixture("missing-receipts");
    let report = three_agent_receipts::write_report(
        &root,
        three_agent_view_artifacts::report_json(&root, Some(&root)),
    );

    let err = execute_verify_mvp_demo_contract(&config(report.as_path()))
        .expect_err("three-agent proof must include actor observation receipts");

    assert!(err.contains("agent_a_observation_receipt"), "{err}");
}

#[test]
fn spec_c02_command_rejects_stale_agent_a_observation_receipt_digest() {
    let root = three_agent_receipts::base_fixture("stale-agent-a-receipt");
    let paths = three_agent_receipts::write_receipts(&root, ReceiptOverrides::default());
    let report = three_agent_receipts::write_report(
        &root,
        three_agent_receipts::report_with_receipts(&root, &paths),
    );
    three_agent_receipts::tamper_json_file(paths.agent_a.as_path(), "agent-a-receipt-tamper");

    let err = execute_verify_mvp_demo_contract(&config(report.as_path()))
        .expect_err("stale Agent A receipt digest must fail");

    assert!(err.contains("agent_a_observation_receipt_digest"), "{err}");
}

#[test]
fn spec_c03_command_rejects_agent_a_receipt_view_digest_mismatch() {
    let root = three_agent_receipts::base_fixture("agent-a-view-digest-mismatch");
    let paths = three_agent_receipts::write_receipts(
        &root,
        ReceiptOverrides {
            agent_a_view_digest: Some("mismatch".to_owned()),
            ..ReceiptOverrides::default()
        },
    );
    let report = three_agent_receipts::write_report(
        &root,
        three_agent_receipts::report_with_receipts(&root, &paths),
    );

    let err = execute_verify_mvp_demo_contract(&config(report.as_path()))
        .expect_err("Agent A receipt must bind to Agent A view digest");

    assert!(err.contains("agent_a_observation_receipt"), "{err}");
}

#[test]
fn spec_c04_command_rejects_agent_c_receipt_private_digest() {
    let root = three_agent_receipts::base_fixture("agent-c-private-receipt");
    let paths = three_agent_receipts::write_receipts(
        &root,
        ReceiptOverrides {
            agent_c_private_digest: true,
            ..ReceiptOverrides::default()
        },
    );
    let report = three_agent_receipts::write_report(
        &root,
        three_agent_receipts::report_with_receipts(&root, &paths),
    );

    let err = execute_verify_mvp_demo_contract(&config(report.as_path()))
        .expect_err("Agent C receipt must not expose participant private digest");

    assert!(
        err.contains("agent_c_verifier_observation_receipt"),
        "{err}"
    );
}

#[test]
fn spec_c05_demo_mvp_devnet_required_writes_observation_receipt_digests() {
    let root = three_agent_receipts::temp_root("generated-receipts");
    let config = mvp_demo_command::devnet_required_demo_config(&root);
    let report = execute_mvp_demo_contract(&config)
        .expect("devnet-required demo should generate receipt artifacts");

    for marker in observation_receipt_markers() {
        assert!(report.contains(marker), "missing report marker: {marker}");
    }
    let _ = std::fs::remove_dir_all(&root);
}

fn config(report: &Path) -> VerifyMvpDemoCommandConfig {
    VerifyMvpDemoCommandConfig {
        report: report.display().to_string(),
    }
}

fn observation_receipt_markers() -> [&'static str; 3] {
    [
        r#""agent_a_observation_receipt_digest":"sha256:"#,
        r#""agent_b_observation_receipt_digest":"sha256:"#,
        r#""agent_c_verifier_observation_receipt_digest":"sha256:"#,
    ]
}
