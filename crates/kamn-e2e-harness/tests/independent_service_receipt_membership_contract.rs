use kamn_e2e_harness::{execute_verify_mvp_demo_contract, VerifyMvpDemoCommandConfig};

#[path = "support/agent_transaction_demo_fixture.rs"]
mod agent_transaction_demo_fixture;
#[path = "support/artifact_digest.rs"]
#[allow(dead_code)]
mod artifact_digest;
#[path = "support/pi_transaction_actor_fixture.rs"]
mod pi_transaction_actor_fixture;
use pi_transaction_actor_fixture::ActorFixture;

#[test]
fn fabricated_portable_receipt_cannot_reuse_durable_chain_commitment() {
    let root = unique_root();
    let actors = ActorFixture::new();
    actors.write_bound_v2_all();
    agent_transaction_demo_fixture::execute(&root, &actors.paths()).expect("valid proof bundle");
    forge_actor_and_transcript(root.as_path());

    let error = execute_verify_mvp_demo_contract(&VerifyMvpDemoCommandConfig {
        report: root
            .join("demo/latest/proof/report.json")
            .display()
            .to_string(),
        agent_harness_evidence_path: None,
        pi_transaction_actor_paths: None,
    })
    .expect_err("fabricated receipt membership must fail");
    assert_eq!(error, "RECEIPT_CHAIN_INVALID");
}

fn forge_actor_and_transcript(root: &std::path::Path) {
    let proof = only_run_dir(&root.join("demo")).join("proof");
    refresh_artifact(
        &proof.join("runtime-agent-a-evidence.json"),
        "artifact_digest",
    );
    let transcript = proof.join("three-agent-transcript.json");
    let old = read_digest(&transcript, "chain_digest");
    refresh_artifact(&transcript, "chain_digest");
    let new = read_digest(&transcript, "chain_digest");
    let report = root.join("demo/latest/proof/report.json");
    let raw = std::fs::read_to_string(&report).expect("report");
    std::fs::write(report, raw.replace(old.as_str(), new.as_str())).expect("forged report claim");
}

fn refresh_artifact(path: &std::path::Path, digest_field: &str) {
    let raw = std::fs::read_to_string(path).expect("proof artifact");
    let forged = raw.replace("service-receipt-01", "service-receipt-forged");
    assert_ne!(forged, raw, "receipt mutation must change the artifact");
    std::fs::write(path, artifact_digest::with_digest(forged, digest_field))
        .expect("forged proof artifact");
}

fn read_digest(path: &std::path::Path, field: &str) -> String {
    let raw = std::fs::read_to_string(path).expect("proof artifact");
    artifact_digest::digest_field(raw.as_str(), field)
}

fn only_run_dir(root: &std::path::Path) -> std::path::PathBuf {
    std::fs::read_dir(root)
        .expect("fixture root")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| path.is_dir() && path.file_name().is_some_and(|name| name != "latest"))
        .expect("one run directory")
}

fn unique_root() -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "kamn-receipt-member-{}-{nanos}",
        std::process::id()
    ))
}
