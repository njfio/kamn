use std::path::{Path, PathBuf};

use kamn_e2e_harness::{execute_verify_mvp_demo_contract, VerifyMvpDemoCommandConfig};

#[path = "support/agent_transaction_demo_fixture.rs"]
mod agent_transaction_demo_fixture;
#[path = "support/pi_transaction_actor_fixture.rs"]
mod pi_transaction_actor_fixture;
use pi_transaction_actor_fixture::ActorFixture;

#[test]
fn spec_c01_success_writes_one_runtime_chain_report_after_children_exit() {
    let root = unique_root();
    let actors = ActorFixture::new();
    actors.write_bound_v2_all();
    let report = agent_transaction_demo_fixture::execute(&root, &actors.paths())
        .expect("canonical deterministic demo should pass");
    assert!(report.contains("kamn.service.receipt-chain.v1"));
    assert!(report.contains(r#""receipt_chain_commitment":"sha256:"#));
    assert!(!report.contains("agent_a_registered"));
    assert_eq!(run_directories(&root.join("demo")).len(), 1);
    assert!(!root.join("demo/latest/NO-GO.txt").exists());
    let bootstrap = std::fs::read_to_string(root.join("staging/service-api-state.json"))
        .expect("local authorization bootstrap");
    assert!(bootstrap.contains(r#""action":"task:create""#));
    assert!(bootstrap.contains(r#""action":"escrow:release""#));
    assert!(bootstrap.contains(r#""action":"escrow:release-authorize""#));
    verify_latest(&root).expect("canonical verifier should pass after child exit");
}

fn run_directories(root: &Path) -> Vec<PathBuf> {
    std::fs::read_dir(root)
        .expect("output root")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_dir() && path.file_name().and_then(|name| name.to_str()) != Some("latest")
        })
        .collect()
}

fn verify_latest(root: &Path) -> Result<String, String> {
    execute_verify_mvp_demo_contract(&VerifyMvpDemoCommandConfig {
        report: root
            .join("demo/latest/proof/report.json")
            .display()
            .to_string(),
        agent_harness_evidence_path: None,
        pi_transaction_actor_paths: Some([
            root.join("staging/agent-a.json").display().to_string(),
            root.join("staging/agent-b.json").display().to_string(),
            root.join("staging/agent-c.json").display().to_string(),
        ]),
    })
}

fn unique_root() -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!("kamn-agent-success-{}-{nanos}", std::process::id()))
}
