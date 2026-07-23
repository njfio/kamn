use kamn_e2e_harness::{execute_verify_mvp_demo_contract, VerifyMvpDemoCommandConfig};
use std::path::PathBuf;

#[path = "support/agent_transaction_demo_fixture.rs"]
mod agent_transaction_demo_fixture;
#[path = "support/pi_transaction_actor_fixture.rs"]
mod pi_transaction_actor_fixture;
use pi_transaction_actor_fixture::{sha, ActorFixture, Overrides};

#[test]
fn spec_c07_missing_actor_receipt_fails_with_service_authority_code() {
    let fixture = Fixture::new("missing-receipt");
    fixture.rewrite(Overrides {
        agent_a_include_release: false,
        ..Overrides::default()
    });

    fixture.assert_error("PI_SERVICE_AUTHORITY_MISMATCH");
}

#[test]
fn spec_c08_reordered_operations_fail_with_service_authority_code() {
    let fixture = Fixture::new("operation-order");
    fixture.actors.reorder_agent_a_mutations();

    fixture.assert_error("PI_SERVICE_AUTHORITY_MISMATCH");
}

#[test]
fn spec_c09_verifier_private_projection_fails_with_service_authority_code() {
    let fixture = Fixture::new("projection-scope");
    fixture.rewrite(Overrides {
        agent_c_private: Some(sha('f')),
        ..Overrides::default()
    });

    fixture.assert_error("PI_SERVICE_AUTHORITY_MISMATCH");
}

#[test]
fn spec_c10_duplicate_agent_identity_fails_with_service_authority_code() {
    let fixture = Fixture::new("identity-drift");
    fixture.rewrite(Overrides {
        agent_c_did: "kamn:did:a",
        ..Overrides::default()
    });

    fixture.assert_error("PI_SERVICE_AUTHORITY_MISMATCH");
}

#[test]
fn spec_c24_duplicate_agent_process_fails_with_transport_provenance_code() {
    let fixture = Fixture::new("process-reuse");
    fixture.rewrite(Overrides {
        agent_c_pid: 101,
        ..Overrides::default()
    });

    fixture.assert_error("PI_TRANSPORT_PROVENANCE_INVALID");
}

#[test]
fn spec_c16_report_indexes_all_runtime_actor_sources() {
    let fixture = Fixture::new("actor-source-index");
    let report = std::fs::read_to_string(fixture.report_path()).expect("report");

    for field in [
        "runtime_agent_a_evidence",
        "runtime_agent_b_evidence",
        "runtime_agent_c_evidence",
    ] {
        assert!(report.contains(format!(r#""{field}":"#).as_str()));
    }
}

#[test]
fn spec_c17_standalone_verifier_rejects_missing_runtime_actor_source() {
    let fixture = Fixture::new("missing-actor-source");
    let path = fixture.run_proof().join("runtime-agent-a-evidence.json");
    std::fs::remove_file(path).expect("remove actor source");

    fixture.assert_standalone_error("PROOF_ARTIFACT_MISSING");
}

#[test]
fn spec_c18_standalone_verifier_rejects_tampered_runtime_actor_source() {
    let fixture = Fixture::new("tampered-actor-source");
    let path = fixture.run_proof().join("runtime-agent-a-evidence.json");
    let raw = std::fs::read_to_string(&path).expect("actor source");
    std::fs::write(path, raw.replace("kamn:did:a", "kamn:did:forged"))
        .expect("tampered actor source");

    fixture.assert_standalone_error("PI_SERVICE_AUTHORITY_MISMATCH");
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
            .expect("valid actor proof bundle");
        Self { root, actors }
    }

    fn rewrite(&self, overrides: Overrides) {
        self.actors.write_bound_v2(overrides);
    }

    fn assert_error(&self, expected: &str) {
        let error = execute_verify_mvp_demo_contract(&VerifyMvpDemoCommandConfig {
            report: self.report_path().display().to_string(),
            agent_harness_evidence_path: None,
            pi_transaction_actor_paths: Some(self.actors.paths()),
        })
        .expect_err("tampered actor evidence must fail");
        assert_eq!(error, expected);
    }

    fn assert_standalone_error(&self, expected: &str) {
        let error = execute_verify_mvp_demo_contract(&VerifyMvpDemoCommandConfig {
            report: self.report_path().display().to_string(),
            agent_harness_evidence_path: None,
            pi_transaction_actor_paths: None,
        })
        .expect_err("invalid bundled actor evidence must fail");
        assert_eq!(error, expected);
    }

    fn report_path(&self) -> PathBuf {
        self.root.join("demo/latest/proof/report.json")
    }

    fn run_proof(&self) -> PathBuf {
        only_run_dir(&self.root.join("demo")).join("proof")
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

fn only_run_dir(root: &std::path::Path) -> PathBuf {
    std::fs::read_dir(root)
        .expect("fixture root")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| path.is_dir() && path.file_name().is_some_and(|name| name != "latest"))
        .expect("one run directory")
}
