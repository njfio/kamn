use kamn_e2e_harness::{execute_verify_mvp_demo_contract, VerifyMvpDemoCommandConfig};
use std::path::{Path, PathBuf};

#[path = "support/agent_transaction_demo_fixture.rs"]
mod agent_transaction_demo_fixture;
#[path = "support/pi_transaction_actor_fixture.rs"]
mod pi_transaction_actor_fixture;
use pi_transaction_actor_fixture::{sha, ActorFixture, Overrides};

#[test]
fn spec_c27_bundled_identity_drift_keeps_authority_category() {
    assert_bundled_error(
        "identity",
        Overrides {
            agent_c_did: "kamn:did:a",
            ..Overrides::default()
        },
        "PI_SERVICE_AUTHORITY_MISMATCH",
    );
}

#[test]
fn spec_c28_bundled_authorization_drift_keeps_authority_category() {
    assert_bundled_error(
        "authorization",
        Overrides {
            agent_a_handoff_authorized: true,
            ..Overrides::default()
        },
        "PI_SERVICE_AUTHORITY_MISMATCH",
    );
}

#[test]
fn spec_c29_bundled_projection_leak_keeps_authority_category() {
    assert_bundled_error(
        "projection",
        Overrides {
            agent_c_private: Some(sha('f')),
            ..Overrides::default()
        },
        "PI_SERVICE_AUTHORITY_MISMATCH",
    );
}

#[test]
fn spec_c30_bundled_shared_fact_drift_keeps_authority_category() {
    assert_bundled_error(
        "agreement",
        Overrides {
            agent_b_escrow: "escrow-foreign",
            ..Overrides::default()
        },
        "PI_SERVICE_AUTHORITY_MISMATCH",
    );
}

#[test]
fn spec_c32_bundled_runtime_chain_rejects_schema_downgrade() {
    let fixture = Fixture::new("schema-downgrade");
    fixture.downgrade_transcript_schema();

    let error = fixture
        .verify()
        .expect_err("runtime transcript downgrade must fail");
    assert_eq!(error, "RECEIPT_CHAIN_INVALID");
}

fn assert_bundled_error(stem: &str, overrides: Overrides, expected: &str) {
    let fixture = Fixture::new(stem);
    fixture.replace_bundled(overrides);
    let error = fixture.verify().expect_err("bundled actor drift must fail");
    assert_eq!(error, expected);
}

struct Fixture {
    root: PathBuf,
    actors: ActorFixture,
}

impl Fixture {
    fn new(stem: &str) -> Self {
        let root = temp_root(stem);
        let actors = ActorFixture::new();
        actors.write_bound_v2_all();
        agent_transaction_demo_fixture::execute(&root, &actors.paths())
            .expect("valid runtime bundle");
        Self { root, actors }
    }

    fn replace_bundled(&self, overrides: Overrides) {
        self.actors.write_bound_v2(overrides);
        let proof = only_run_dir(&self.root.join("demo")).join("proof");
        for (source, name) in self.actors.paths().iter().zip([
            "runtime-agent-a-evidence.json",
            "runtime-agent-b-evidence.json",
            "runtime-agent-c-evidence.json",
        ]) {
            std::fs::copy(source, proof.join(name)).expect("replace bundled actor");
        }
    }

    fn verify(&self) -> Result<String, String> {
        execute_verify_mvp_demo_contract(&VerifyMvpDemoCommandConfig {
            report: self
                .root
                .join("demo/latest/proof/report.json")
                .display()
                .to_string(),
            agent_harness_evidence_path: None,
            pi_transaction_actor_paths: None,
        })
    }

    fn downgrade_transcript_schema(&self) {
        let path = only_run_dir(&self.root.join("demo")).join("proof/three-agent-transcript.json");
        let raw = std::fs::read_to_string(&path).expect("runtime transcript");
        std::fs::write(
            path,
            raw.replace(
                "kamn.service.receipt-chain.v1",
                "kamn.mvp.three-agent-transcript.v1",
            ),
        )
        .expect("downgraded transcript");
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn only_run_dir(root: &Path) -> PathBuf {
    std::fs::read_dir(root)
        .expect("fixture root")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| path.is_dir() && path.file_name().is_some_and(|name| name != "latest"))
        .expect("one run directory")
}

fn temp_root(stem: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "kamn-7103-bundled-{stem}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ))
}
