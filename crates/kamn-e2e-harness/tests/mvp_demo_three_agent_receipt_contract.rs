use kamn_e2e_harness::execute_mvp_demo_contract;

#[path = "support/generated_receipt_fixture.rs"]
mod generated_receipt_fixture;

#[path = "support/mvp_demo_command.rs"]
mod mvp_demo_command;

#[test]
fn spec_c01_command_rejects_missing_actor_observation_receipts() {
    let fixture = generated_receipt_fixture::Fixture::new("missing-receipts");
    fixture.remove_receipt_references();

    let err = fixture
        .verify()
        .expect_err("three-agent proof must include actor observation receipts");

    assert_eq!(err, "RECEIPT_CHAIN_INVALID");
}

#[test]
fn spec_c02_command_rejects_stale_agent_a_observation_receipt_digest() {
    let fixture = generated_receipt_fixture::Fixture::new("stale-agent-a-receipt");
    fixture.tamper_agent_a_receipt();

    let err = fixture
        .verify()
        .expect_err("stale Agent A receipt digest must fail");

    assert_eq!(err, "RECEIPT_CHAIN_INVALID");
}

#[test]
fn spec_c03_command_rejects_agent_a_receipt_view_digest_mismatch() {
    let fixture = generated_receipt_fixture::Fixture::new("agent-a-view-digest-mismatch");
    fixture.replace_receipt_field("agent_a", "view_digest", "mismatch");

    let err = fixture
        .verify()
        .expect_err("Agent A receipt must bind to Agent A view digest");

    assert_eq!(err, "PROJECTION_SCOPE_INVALID");
}

#[test]
fn spec_c04_command_rejects_agent_c_receipt_private_digest() {
    let fixture = generated_receipt_fixture::Fixture::new("agent-c-private-receipt");
    fixture.replace_receipt_field(
        "agent_c_verifier",
        "participant_private_view_digest",
        "leak",
    );

    let err = fixture
        .verify()
        .expect_err("Agent C receipt must not expose participant private digest");

    assert_eq!(err, "PROJECTION_SCOPE_INVALID");
}

#[test]
fn spec_c05_demo_mvp_devnet_required_writes_observation_receipt_digests() {
    let root = generated_receipt_fixture::temp_root("generated-receipts");
    let config = mvp_demo_command::devnet_required_demo_config(&root);
    let report = execute_mvp_demo_contract(&config)
        .expect("devnet-required demo should generate receipt artifacts");

    for marker in observation_receipt_markers() {
        assert!(report.contains(marker), "missing report marker: {marker}");
    }
    let _ = std::fs::remove_dir_all(&root);
}

fn observation_receipt_markers() -> [&'static str; 3] {
    [
        r#""agent_a_observation_receipt_digest":"sha256:"#,
        r#""agent_b_observation_receipt_digest":"sha256:"#,
        r#""agent_c_verifier_observation_receipt_digest":"sha256:"#,
    ]
}
