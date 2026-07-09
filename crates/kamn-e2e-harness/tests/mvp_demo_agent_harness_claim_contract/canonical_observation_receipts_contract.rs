use crate::support::*;
use kamn_e2e_harness::execute_verify_mvp_demo_contract;

#[test]
fn spec_c19_command_rejects_harness_without_canonical_observation_receipts() {
    let root = temp_root("missing-canonical-observation-receipts");
    let artifact = write_artifact(&root, agent_artifact_with_three_agent_actor_receipts(&root));
    let report = write_report(&root, report_with_three_agent_claim(&root, &artifact));

    let err = execute_verify_mvp_demo_contract(&config(report.as_path()))
        .expect_err("Pi evidence must include canonical observation receipts");

    assert!(err.contains("three_agent_actor_observation_receipts"));
}

#[test]
fn spec_c20_command_rejects_harness_observation_receipt_digest_mismatch() {
    let root = temp_root("canonical-observation-receipt-digest-mismatch");
    let artifact_json = agent_artifact_with_canonical_observation_receipts(&root).replace(
        r#""agent":"agent_a","view_scope":"participant-private","artifact":"#,
        r#""agent":"agent_a","view_scope":"participant-private","digest":"sha256:mismatch","artifact":"#,
    );
    let artifact = write_artifact(&root, artifact_json);
    let report = write_report(&root, report_with_three_agent_claim(&root, &artifact));

    let err = execute_verify_mvp_demo_contract(&config(report.as_path()))
        .expect_err("canonical observation receipt digests must match the report claim");

    assert!(err.contains("agent_a_observation_receipt_digest"), "{err}");
}

#[test]
fn spec_c21_command_rejects_agent_c_observation_receipt_private_marker() {
    let root = temp_root("canonical-observation-receipt-agent-c-private");
    let artifact_json = agent_artifact_with_canonical_observation_receipts(&root).replace(
        r#""agent":"agent_c_verifier","view_scope":"restricted-public""#,
        r#""agent":"agent_c_verifier","view_scope":"restricted-public","participant_private_view_digest":"leak""#,
    );
    let artifact = write_artifact(&root, artifact_json);
    let report = write_report(&root, report_with_three_agent_claim(&root, &artifact));

    let err = execute_verify_mvp_demo_contract(&config(report.as_path()))
        .expect_err("Agent C canonical receipt evidence must not expose private markers");

    assert!(err.contains("agent_c_verifier"), "{err}");
}
