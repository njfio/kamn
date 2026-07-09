use crate::support::*;
use kamn_e2e_harness::execute_verify_mvp_demo_contract;

#[test]
fn spec_c15_command_rejects_three_agent_harness_without_actor_tool_receipts() {
    let root = temp_root("missing-actor-tool-receipts");
    let artifact = write_artifact(
        &root,
        agent_artifact_with_three_agent_actor_rehearsal(&root),
    );
    let report = write_report(&root, report_with_three_agent_claim(&root, &artifact));

    let err = execute_verify_mvp_demo_contract(&config(report.as_path()))
        .expect_err("three-agent Pi evidence must include actor tool receipts");

    assert!(err.contains("three_agent_actor_tool_receipts"));
}

#[test]
fn spec_c16_command_rejects_actor_tool_receipt_view_digest_mismatch() {
    let root = temp_root("receipt-digest-mismatch");
    let artifact_json = agent_artifact_with_canonical_observation_receipts(&root).replace(
        format!(r#""view_digest":"{}""#, view_digest_for("agent_a")).as_str(),
        r#""view_digest":"mismatch""#,
    );
    let artifact = write_artifact(&root, artifact_json);
    let report = write_report(&root, report_with_three_agent_claim(&root, &artifact));

    let err = execute_verify_mvp_demo_contract(&config(report.as_path()))
        .expect_err("actor tool receipt view digests must match report claim");

    assert!(err.contains("three_agent_actor_tool_receipts"));
}

#[test]
fn spec_c17_command_rejects_agent_c_private_actor_tool_receipt_digest() {
    let root = temp_root("receipt-agent-c-private");
    let artifact_json = agent_artifact_with_canonical_observation_receipts(&root).replace(
        r#""sequence":5,"tool":"kamn_agent_c_verify_three_agent_proof""#,
        r#""sequence":5,"tool":"kamn_agent_c_verify_three_agent_proof","participant_private_view_digest":"leak""#,
    );
    let artifact = write_artifact(&root, artifact_json);
    let report = write_report(&root, report_with_three_agent_claim(&root, &artifact));

    let err = execute_verify_mvp_demo_contract(&config(report.as_path()))
        .expect_err("Agent C actor tool receipt must not expose private digest");

    assert!(err.contains("agent_c_verifier"));
}

#[test]
fn spec_c18_command_accepts_actor_tool_receipts_bound_to_report_views() {
    let root = temp_root("valid-actor-tool-receipts");
    let artifact = write_artifact(
        &root,
        agent_artifact_with_canonical_observation_receipts(&root),
    );
    let report = write_report(&root, report_with_three_agent_claim(&root, &artifact));

    execute_verify_mvp_demo_contract(&config(report.as_path()))
        .expect("actor tool receipts bound to report views should verify");
}
