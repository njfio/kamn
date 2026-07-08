use crate::support::*;
use kamn_e2e_harness::execute_verify_mvp_demo_contract;

#[test]
fn spec_c09_command_rejects_agent_artifact_without_three_agent_boundary() {
    let root = temp_root("missing-three-agent-boundary");
    let artifact = write_artifact(&root, agent_artifact_without_three_agent_boundary(&root));
    let report = write_report(&root, report_with_three_agent_claim(&root, &artifact));

    let err = execute_verify_mvp_demo_contract(&config(report.as_path()))
        .expect_err("agent harness evidence must include three-agent boundary");

    assert!(err.contains("three_agent_boundary"));
}

#[test]
fn spec_c10_command_rejects_agent_artifact_with_mismatched_three_agent_claim() {
    for (stem, from, to, expected) in mismatched_claim_cases() {
        let root = temp_root(format!("mismatched-three-agent-{stem}").as_str());
        let artifact_json = agent_artifact_with_three_agent_boundary(&root).replace(from, to);
        let artifact = write_artifact(&root, artifact_json);
        let report = write_report(&root, report_with_three_agent_claim(&root, &artifact));

        let err = execute_verify_mvp_demo_contract(&config(report.as_path()))
            .expect_err("agent harness evidence must match report claim");
        assert!(err.contains(expected));
    }
}

#[test]
fn spec_c11_command_rejects_agent_artifact_with_invalid_private_boundary() {
    for (stem, from, to, expected) in invalid_private_boundary_cases() {
        let root = temp_root(format!("invalid-three-agent-{stem}").as_str());
        let artifact_json = agent_artifact_with_three_agent_boundary(&root).replace(from, to);
        let artifact = write_artifact(&root, artifact_json);
        let report = write_report(&root, report_with_three_agent_claim(&root, &artifact));

        let err = execute_verify_mvp_demo_contract(&config(report.as_path()))
            .expect_err("agent harness evidence must preserve private boundary");
        assert!(err.contains(expected));
    }
}

fn mismatched_claim_cases() -> [(&'static str, &'static str, &'static str, &'static str); 2] {
    [
        (
            "status",
            r#""claim_status":"PASS""#,
            r#""claim_status":"NO-GO""#,
            "claim_status",
        ),
        (
            "label",
            r#""claim_label":"devnet-backed""#,
            r#""claim_label":"local-only""#,
            "claim_label",
        ),
    ]
}

fn invalid_private_boundary_cases() -> [(&'static str, &'static str, &'static str, &'static str); 4]
{
    [
        (
            "agent-a-zero",
            r#""agent_a_private_field_count":3"#,
            r#""agent_a_private_field_count":0"#,
            "agent_a_private_field_count",
        ),
        (
            "verifier-private",
            r#""verifier_private_field_count":0"#,
            r#""verifier_private_field_count":1"#,
            "verifier_private_field_count",
        ),
        (
            "digest-present",
            r#""verifier_private_view_digest_present":false"#,
            r#""verifier_private_view_digest_present":true"#,
            "verifier_private_view_digest_present",
        ),
        (
            "unredacted",
            r#""private_payload_redacted":true"#,
            r#""private_payload_redacted":false"#,
            "private_payload_redacted",
        ),
    ]
}
