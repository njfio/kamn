use std::collections::BTreeMap;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use kamn_e2e_harness::{
    execute_agent_transaction_demo_with_config, execute_verify_mvp_demo_contract,
    parse_agent_transaction_demo_config, LiveTaskEvidencePaths, VerifyMvpDemoCommandConfig,
};

#[path = "support/fake_local_runtime.rs"]
mod fake_local_runtime;
#[path = "support/mvp_demo_command.rs"]
mod mvp_demo_command;
#[path = "support/pi_transaction_actor_fixture.rs"]
mod pi_transaction_actor_fixture;
use pi_transaction_actor_fixture::{ActorFixture, Overrides};

#[test]
fn spec_c01_success_writes_one_runtime_chain_report_after_children_exit() {
    let root = unique_root();
    let actors = ActorFixture::new();
    actors.write_all(Overrides::default());
    actors.rebind_shared_facts();
    let demo_stub = mvp_demo_command::devnet_required_demo_config(&root.join("stub"));
    let live = demo_stub
        .live_task_evidence
        .as_ref()
        .expect("live evidence");
    let mut config = configured_fixture(&root, &actors.paths(), live);
    config.devnet_settlement_command = demo_stub.devnet_settlement_command;
    config.localhost_signed_demo_command = demo_stub.localhost_signed_demo_command;
    config.service_api_vertical_slice_command = demo_stub.service_api_vertical_slice_command;
    config.service_api_websocket_command = demo_stub.service_api_websocket_command;

    let report = execute_agent_transaction_demo_with_config(&config)
        .expect("canonical deterministic demo should pass");
    assert!(report.contains("kamn.mvp.runtime-receipt-chain.v1"));
    assert!(!report.contains("agent_a_registered"));
    assert_eq!(run_directories(&root.join("demo")).len(), 1);
    assert!(!root.join("demo/latest/NO-GO.txt").exists());
    verify_latest(&root).expect("canonical verifier should pass after child exit");
}

fn configured_fixture(
    root: &Path,
    actor_sources: &[String; 3],
    live: &LiveTaskEvidencePaths,
) -> kamn_e2e_harness::AgentTransactionDemoConfig {
    std::fs::create_dir_all(root.join("extension")).expect("fixture root");
    for name in ["agent-a.key", "agent-b.key", "agent-c.key"] {
        std::fs::write(root.join(name), name).expect("agent key");
    }
    let payer = (0_u8..64).collect::<Vec<_>>();
    std::fs::write(
        root.join("payer.json"),
        serde_json::to_string(&payer).expect("payer"),
    )
    .expect("payer file");
    std::fs::write(root.join("extension/index.ts"), "export default {}").expect("extension");
    write_success_pi(root, actor_sources, live);
    parse_agent_transaction_demo_config(&fixture_env(root)).expect("configuration")
}

fn fixture_env(root: &Path) -> BTreeMap<String, String> {
    let mut env = base_env();
    for (name, file) in [
        ("KAMN_MVP_LIVE_MCP_AGENT_A_KEY_FILE", "agent-a.key"),
        ("KAMN_MVP_LIVE_MCP_AGENT_B_KEY_FILE", "agent-b.key"),
        ("KAMN_MVP_LIVE_MCP_AGENT_C_KEY_FILE", "agent-c.key"),
        (
            "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_KEYPAIR_FILE",
            "payer.json",
        ),
        ("KAMN_MVP_PI_BINARY", "pi"),
        ("KAMN_MVP_PI_EXTENSION", "extension/index.ts"),
        ("KAMN_MVP_AGENT_TRANSACTION_OUTPUT_ROOT", "demo"),
        ("KAMN_MVP_AGENT_TRANSACTION_STAGING_ROOT", "staging"),
    ] {
        env.insert(name.to_owned(), root.join(file).display().to_string());
    }
    fake_local_runtime::configure(root, &mut env);
    env
}

fn write_success_pi(root: &Path, actors: &[String; 3], live: &LiveTaskEvidencePaths) {
    let script = format!(
        r#"#!/bin/sh
case " $* " in *" --print "*) echo KAMN_PI_PREFLIGHT_OK; exit 0;; esac
role=""; previous=""
for arg in "$@"; do
  if [ "$previous" = "--name" ]; then role="$arg"; fi
  previous="$arg"
done
test -n "$KAMN_MVP_LIVE_MCP_BINARY" || exit 41
test -n "$KAMN_MVP_LIVE_MCP_ENDPOINT" || exit 42
test -n "$KAMN_MVP_LIVE_MCP_AGENT_A_NAME" || exit 43
test -n "$KAMN_MVP_LIVE_MCP_AGENT_B_NAME" || exit 44
test -n "$KAMN_MVP_LIVE_MCP_AGENT_C_NAME" || exit 45
while read line; do
  mkdir -p "$(dirname "$KAMN_MVP_PI_TRANSACTION_AGENT_A_FILE")"
  cp "{}" "$KAMN_MVP_PI_TRANSACTION_AGENT_A_FILE"
  cp "{}" "$KAMN_MVP_PI_TRANSACTION_AGENT_B_FILE"
  cp "{}" "$KAMN_MVP_PI_TRANSACTION_AGENT_C_FILE"
  cp "{}" "$KAMN_MVP_LIVE_TASK_HANDOFF_FILE"
  cp "{}" "$KAMN_MVP_LIVE_TASK_AGENT_A_RECEIPT_FILE"
  cp "{}" "$KAMN_MVP_LIVE_TASK_AGENT_B_RECEIPT_FILE"
  cp "{}" "$KAMN_MVP_LIVE_TASK_AGENT_C_OBSERVATION_FILE"
  echo '{{"type":"response","command":"prompt","success":true}}'
  if [ "$role" = "kamn-mvp-agent-b" ]; then
    echo '{{"type":"agent_end","messages":[{{"did":"kamn:did:agent:b"}}]}}'
  else
    echo '{{"type":"agent_end","messages":[]}}'
  fi
done
"#,
        actors[0],
        actors[1],
        actors[2],
        live.handoff,
        live.agent_a_receipt,
        live.agent_b_receipt,
        live.agent_c_observation,
    );
    write_executable(root.join("pi"), script.as_str());
}

fn base_env() -> BTreeMap<String, String> {
    [
        ("KAMN_MVP_AGENT_DRIVER", "pi"),
        ("KAMN_MVP_DEVNET_MODE", "required"),
        ("KAMN_MVP_SOLANA_RPC_URL", "https://api.devnet.solana.com"),
        (
            "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_RECIPIENT_PUBKEY",
            "FV5LvudLjZQGCrPwXUY2JaVr26sQE15K25BGvsKWvyFe",
        ),
        (
            "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_LAMPORTS",
            "1000000",
        ),
        (
            "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_COMMITMENT",
            "finalized",
        ),
    ]
    .into_iter()
    .map(|(key, value)| (key.to_owned(), value.to_owned()))
    .collect()
}

fn write_executable(path: PathBuf, body: &str) {
    std::fs::write(&path, body).expect("fake Pi");
    let mut permissions = std::fs::metadata(&path).expect("metadata").permissions();
    permissions.set_mode(0o700);
    std::fs::set_permissions(path, permissions).expect("permissions");
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
