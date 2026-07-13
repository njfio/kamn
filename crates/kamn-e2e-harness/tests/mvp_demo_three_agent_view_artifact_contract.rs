#[path = "support/generated_receipt_fixture.rs"]
mod generated_receipt_fixture;

#[path = "support/generated_view_fixture.rs"]
mod generated_view_fixture;

#[path = "support/mvp_demo_command.rs"]
mod mvp_demo_command;

#[test]
fn spec_c01_command_rejects_missing_three_agent_view_artifacts() {
    let fixture = generated_view_fixture::Fixture::new("missing-view");
    fixture.remove_view("agent_a");

    let err = fixture
        .verify()
        .expect_err("missing per-agent view artifacts must fail");

    assert_eq!(err, "PROOF_ARTIFACT_MISSING");
}

#[test]
fn spec_c02_command_rejects_agent_c_private_overdisclosure() {
    let fixture = generated_view_fixture::Fixture::new("agent-c-private");
    fixture.replace_view_field(
        "agent_c_verifier",
        "participant_private_view_digest",
        "leaked",
    );

    let err = fixture
        .verify()
        .expect_err("Agent C private over-disclosure must fail");

    assert_eq!(err, "PROJECTION_SCOPE_INVALID");
}

#[test]
fn spec_c03_command_rejects_mismatched_view_settlement_signature() {
    let fixture = generated_view_fixture::Fixture::new("mismatched-view-signature");
    fixture.replace_view_field("agent_c_verifier", "settlement_tx_signature", "mismatch");

    let err = fixture
        .verify()
        .expect_err("view settlement mismatch must fail");

    assert_eq!(err, "PROJECTION_SCOPE_INVALID");
}

#[test]
fn spec_c04_command_rejects_agent_a_view_identity_mismatch() {
    let fixture = generated_view_fixture::Fixture::new("agent-a-identity-mismatch");
    fixture.replace_view_field("agent_a", "agent", "agent_b");

    let err = fixture
        .verify()
        .expect_err("Agent A view artifact identity must match agent_a");

    assert_eq!(err, "PROJECTION_SCOPE_INVALID");
}

#[test]
fn spec_c05_command_rejects_agent_b_view_identity_mismatch() {
    let fixture = generated_view_fixture::Fixture::new("agent-b-identity-mismatch");
    fixture.replace_view_field("agent_b", "agent", "agent_a");

    let err = fixture
        .verify()
        .expect_err("Agent B view artifact identity must match agent_b");

    assert_eq!(err, "PROJECTION_SCOPE_INVALID");
}

#[test]
fn spec_c06_command_rejects_agent_c_short_identity() {
    let fixture = generated_view_fixture::Fixture::new("agent-c-short-identity");
    fixture.replace_view_field("agent_c_verifier", "agent", "agent_c");

    let err = fixture
        .verify()
        .expect_err("Agent C verifier view artifact identity must be agent_c_verifier");

    assert_eq!(err, "PROJECTION_SCOPE_INVALID");
}
