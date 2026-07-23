use kamn_e2e_harness::execute_mvp_demo_contract;

#[path = "support/artifact_digest.rs"]
#[allow(dead_code)]
mod artifact_digest;
#[path = "support/mvp_demo_command.rs"]
mod mvp_demo_command;
#[path = "support/pi_transaction_actor_fixture.rs"]
mod pi_transaction_actor_fixture;

const DIRECT_SETTLEMENT_OVERRIDE_ERROR: &str = "PI_SERVICE_AUTHORITY_MISMATCH";

#[test]
fn spec_c07_required_verifier_rejects_command_override_settlement() {
    let root = temp_root();
    let actors = pi_transaction_actor_fixture::ActorFixture::new();
    actors.write_all(pi_transaction_actor_fixture::Overrides::default());
    actors.rebind_shared_facts();
    let mut config = mvp_demo_command::devnet_required_demo_config(&root);
    config.pi_transaction_actor_paths = Some(actors.paths());
    let error = execute_mvp_demo_contract(&config)
        .expect_err("override-only required-devnet proof must fail before publication");
    assert_eq!(error, DIRECT_SETTLEMENT_OVERRIDE_ERROR);
}

fn temp_root() -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "kamn-7125-command-override-rejected-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock should be valid")
            .as_nanos()
    ))
}
