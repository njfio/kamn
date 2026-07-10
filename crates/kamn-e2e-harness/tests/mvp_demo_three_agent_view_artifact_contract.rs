use kamn_e2e_harness::{execute_verify_mvp_demo_contract, VerifyMvpDemoCommandConfig};
use std::path::Path;

#[path = "support/mvp_local_artifacts.rs"]
mod mvp_local_artifacts;

#[path = "support/three_agent_view_artifacts.rs"]
mod three_agent_view_artifacts;

#[test]
fn spec_c01_command_rejects_missing_three_agent_view_artifacts() {
    let root = three_agent_view_artifacts::temp_root("missing-views");
    mvp_local_artifacts::write_valid_local_artifacts(&root);
    three_agent_view_artifacts::write_transcript(
        &root,
        three_agent_view_artifacts::transcript(None),
    );
    let report = three_agent_view_artifacts::write_report(
        &root,
        three_agent_view_artifacts::report_json(&root, None),
    );

    let err = execute_verify_mvp_demo_contract(&config(report.as_path()))
        .expect_err("missing per-agent view artifacts must fail");

    assert!(err.contains("agent_a_view"));
}

#[test]
fn spec_c02_command_rejects_agent_c_private_overdisclosure() {
    let root = three_agent_view_artifacts::temp_root("agent-c-private");
    mvp_local_artifacts::write_valid_local_artifacts(&root);
    three_agent_view_artifacts::write_view_artifacts(
        &root,
        Some(three_agent_view_artifacts::agent_c_private_view()),
    );
    three_agent_view_artifacts::write_transcript(
        &root,
        three_agent_view_artifacts::transcript(Some(&root)),
    );
    let report = three_agent_view_artifacts::write_report(
        &root,
        three_agent_view_artifacts::report_json(&root, Some(&root)),
    );

    let err = execute_verify_mvp_demo_contract(&config(report.as_path()))
        .expect_err("Agent C private over-disclosure must fail");

    assert!(err.contains("agent_c_verifier_view"));
}

#[test]
fn spec_c03_command_rejects_mismatched_view_settlement_signature() {
    let root = three_agent_view_artifacts::temp_root("mismatched-view-signature");
    mvp_local_artifacts::write_valid_local_artifacts(&root);
    three_agent_view_artifacts::write_view_artifacts(
        &root,
        Some(three_agent_view_artifacts::agent_c_mismatched_signature_view()),
    );
    three_agent_view_artifacts::write_transcript(
        &root,
        three_agent_view_artifacts::transcript(Some(&root)),
    );
    let report = three_agent_view_artifacts::write_report(
        &root,
        three_agent_view_artifacts::report_json(&root, Some(&root)),
    );

    let err = execute_verify_mvp_demo_contract(&config(report.as_path()))
        .expect_err("view settlement mismatch must fail");

    assert!(err.contains("settlement_tx_signature"));
}

#[test]
fn spec_c04_command_rejects_agent_a_view_identity_mismatch() {
    let root = three_agent_view_artifacts::temp_root("agent-a-identity-mismatch");
    mvp_local_artifacts::write_valid_local_artifacts(&root);
    three_agent_view_artifacts::write_view_artifacts(&root, None);
    three_agent_view_artifacts::replace_agent_a_view(
        &root,
        three_agent_view_artifacts::agent_a_mismatched_identity_view(),
    );
    three_agent_view_artifacts::write_transcript(
        &root,
        three_agent_view_artifacts::transcript(Some(&root)),
    );
    let report = three_agent_view_artifacts::write_report(
        &root,
        three_agent_view_artifacts::report_json(&root, Some(&root)),
    );

    let err = execute_verify_mvp_demo_contract(&config(report.as_path()))
        .expect_err("Agent A view artifact identity must match agent_a");

    assert!(err.contains("agent_a_view"));
}

#[test]
fn spec_c05_command_rejects_agent_b_view_identity_mismatch() {
    let root = three_agent_view_artifacts::temp_root("agent-b-identity-mismatch");
    mvp_local_artifacts::write_valid_local_artifacts(&root);
    three_agent_view_artifacts::write_view_artifacts(&root, None);
    three_agent_view_artifacts::replace_agent_b_view(
        &root,
        three_agent_view_artifacts::agent_b_mismatched_identity_view(),
    );
    three_agent_view_artifacts::write_transcript(
        &root,
        three_agent_view_artifacts::transcript(Some(&root)),
    );
    let report = three_agent_view_artifacts::write_report(
        &root,
        three_agent_view_artifacts::report_json(&root, Some(&root)),
    );

    let err = execute_verify_mvp_demo_contract(&config(report.as_path()))
        .expect_err("Agent B view artifact identity must match agent_b");

    assert!(err.contains("agent_b_view"));
}

#[test]
fn spec_c06_command_rejects_agent_c_short_identity() {
    let root = three_agent_view_artifacts::temp_root("agent-c-short-identity");
    mvp_local_artifacts::write_valid_local_artifacts(&root);
    three_agent_view_artifacts::write_view_artifacts(
        &root,
        Some(three_agent_view_artifacts::agent_c_short_identity_view()),
    );
    three_agent_view_artifacts::write_transcript(
        &root,
        three_agent_view_artifacts::transcript(Some(&root)),
    );
    let report = three_agent_view_artifacts::write_report(
        &root,
        three_agent_view_artifacts::report_json(&root, Some(&root)),
    );

    let err = execute_verify_mvp_demo_contract(&config(report.as_path()))
        .expect_err("Agent C verifier view artifact identity must be agent_c_verifier");

    assert!(err.contains("agent_c_verifier_view"));
}

fn config(report: &Path) -> VerifyMvpDemoCommandConfig {
    VerifyMvpDemoCommandConfig {
        report: report.display().to_string(),
        agent_harness_evidence_path: None,
    }
}
