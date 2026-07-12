use kamn_e2e_harness::{
    execute_mvp_demo_contract, execute_verify_mvp_demo_contract, VerifyMvpDemoCommandConfig,
};
use std::path::PathBuf;

#[path = "support/mvp_demo_command.rs"]
mod mvp_demo_command;
#[path = "support/pi_transaction_actor_fixture.rs"]
mod pi_transaction_actor_fixture;
use pi_transaction_actor_fixture::{sha, ActorFixture, Overrides};

#[test]
fn spec_c07_missing_actor_receipt_fails_with_public_chain_code() {
    let fixture = Fixture::new("missing-receipt");
    fixture.rewrite(Overrides {
        agent_a_include_release: false,
        ..Overrides::default()
    });

    fixture.assert_error("RECEIPT_CHAIN_INVALID");
}

#[test]
fn spec_c08_reordered_operations_fail_with_public_chain_code() {
    let fixture = Fixture::new("operation-order");
    fixture.actors.reorder_agent_a_mutations();

    fixture.assert_error("RECEIPT_CHAIN_INVALID");
}

#[test]
fn spec_c09_verifier_private_projection_fails_with_public_scope_code() {
    let fixture = Fixture::new("projection-scope");
    fixture.rewrite(Overrides {
        agent_c_private: Some(sha('f')),
        ..Overrides::default()
    });

    fixture.assert_error("PROJECTION_SCOPE_INVALID");
}

#[test]
fn spec_c10_duplicate_agent_identity_fails_with_public_identity_code() {
    let fixture = Fixture::new("identity-drift");
    fixture.rewrite(Overrides {
        agent_c_did: "kamn:did:a",
        ..Overrides::default()
    });

    fixture.assert_error("AGENT_IDENTITY_INVALID");
}

struct Fixture {
    root: PathBuf,
    actors: ActorFixture,
}

impl Fixture {
    fn new(stem: &str) -> Self {
        let root = temp_root(stem);
        let actors = ActorFixture::new();
        actors.write_all(Overrides::default());
        actors.rebind_shared_facts();
        let mut config = mvp_demo_command::devnet_required_demo_config(&root);
        config.pi_transaction_actor_paths = Some(actors.paths());
        execute_mvp_demo_contract(&config).expect("valid actor proof bundle");
        Self { root, actors }
    }

    fn rewrite(&self, overrides: Overrides) {
        self.actors.write_all(overrides);
        self.actors.rebind_shared_facts();
    }

    fn assert_error(&self, expected: &str) {
        let error = execute_verify_mvp_demo_contract(&VerifyMvpDemoCommandConfig {
            report: self
                .root
                .join("latest/proof/report.json")
                .display()
                .to_string(),
            agent_harness_evidence_path: None,
            pi_transaction_actor_paths: Some(self.actors.paths()),
        })
        .expect_err("tampered actor evidence must fail");
        assert_eq!(error, expected);
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn temp_root(stem: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "kamn-7103-actor-{stem}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ))
}
