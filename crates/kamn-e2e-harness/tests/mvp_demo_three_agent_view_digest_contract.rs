#[path = "support/generated_receipt_fixture.rs"]
mod generated_receipt_fixture;

#[path = "support/generated_view_fixture.rs"]
mod generated_view_fixture;

#[path = "support/mvp_demo_command.rs"]
mod mvp_demo_command;

#[test]
fn spec_c01_command_rejects_stale_agent_a_view_digest_after_content_tamper() {
    assert_stale_view_digest_rejected("agent-a-stale-view-digest", "agent_a");
}

#[test]
fn spec_c02_command_rejects_stale_agent_b_view_digest_after_content_tamper() {
    assert_stale_view_digest_rejected("agent-b-stale-view-digest", "agent_b");
}

#[test]
fn spec_c03_command_rejects_stale_agent_c_view_digest_after_content_tamper() {
    assert_stale_view_digest_rejected("agent-c-stale-view-digest", "agent_c_verifier");
}

fn assert_stale_view_digest_rejected(stem: &str, agent: &str) {
    let fixture = generated_view_fixture::Fixture::new(stem);
    fixture.tamper_view(agent);
    let err = fixture.verify().expect_err("stale view digest must fail");
    assert_eq!(err, "PROJECTION_SCOPE_INVALID");
}
