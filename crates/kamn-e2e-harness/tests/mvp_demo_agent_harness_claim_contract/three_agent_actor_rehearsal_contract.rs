use crate::support::*;
use kamn_e2e_harness::execute_verify_mvp_demo_contract;

#[test]
fn spec_c12_command_rejects_three_agent_harness_without_actor_rehearsal() {
    let root = temp_root("missing-actor-rehearsal");
    let artifact = write_artifact(&root, agent_artifact_with_three_agent_boundary(&root));
    let report = write_report(&root, report_with_three_agent_claim(&root, &artifact));

    let err = execute_verify_mvp_demo_contract(&config(report.as_path()))
        .expect_err("three-agent Pi evidence must include actor rehearsal");

    assert!(err.contains("three_agent_actor_rehearsal"));
}

#[test]
fn spec_c13_command_rejects_actor_rehearsal_with_agent_c_private_scope() {
    let root = temp_root("actor-c-private-scope");
    let artifact_json = agent_artifact_with_three_agent_actor_rehearsal(&root).replace(
        r#""view_scope":"restricted-public""#,
        r#""view_scope":"participant-private""#,
    );
    let artifact = write_artifact(&root, artifact_json);
    let report = write_report(&root, report_with_three_agent_claim(&root, &artifact));

    let err = execute_verify_mvp_demo_contract(&config(report.as_path()))
        .expect_err("Agent C actor rehearsal must stay restricted-public");

    assert!(err.contains("agent_c_verifier"));
}

#[test]
fn spec_c14_command_rejects_actor_rehearsal_view_digest_mismatch() {
    let root = temp_root("actor-digest-mismatch");
    let artifact_json = agent_artifact_with_three_agent_actor_rehearsal(&root).replace(
        format!(r#""agent_a_view_digest":"{}""#, view_digest_for("agent_a")).as_str(),
        r#""agent_a_view_digest":"mismatch""#,
    );
    let artifact = write_artifact(&root, artifact_json);
    let report = write_report(&root, report_with_three_agent_claim(&root, &artifact));

    let err = execute_verify_mvp_demo_contract(&config(report.as_path()))
        .expect_err("actor rehearsal view digests must match report claim");

    assert!(err.contains("agent_a_view_digest"));
}
