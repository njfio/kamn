#![allow(clippy::duplicate_mod)]

use std::collections::BTreeMap;
use std::path::Path;

use kamn_e2e_harness::{
    execute_agent_transaction_demo_with_config, parse_agent_transaction_demo_config,
};

#[path = "direct_settlement_fixture.rs"]
mod direct_settlement_fixture;
#[path = "fake_local_runtime.rs"]
mod fake_local_runtime;
#[path = "mvp_demo_command.rs"]
mod mvp_demo_command;
#[path = "agent_transaction_success_pi.rs"]
mod success_pi;

pub(crate) fn execute(root: &Path, actor_sources: &[String; 3]) -> Result<String, String> {
    let live = mvp_demo_command::live_task_evidence::write_v2(&root.join("live"));
    let _path_guard = direct_settlement_fixture::install(root);
    let mut config = configured(root, actor_sources, &live);
    let stub = mvp_demo_command::devnet_required_demo_config(&root.join("stub"));
    config.localhost_signed_demo_command = stub.localhost_signed_demo_command;
    config.service_api_vertical_slice_command = stub.service_api_vertical_slice_command;
    config.service_api_websocket_command = stub.service_api_websocket_command;
    execute_agent_transaction_demo_with_config(&config)
}

fn configured(
    root: &Path,
    actor_sources: &[String; 3],
    live: &kamn_e2e_harness::LiveTaskEvidencePaths,
) -> kamn_e2e_harness::AgentTransactionDemoConfig {
    write_inputs(root);
    success_pi::write(
        root,
        actor_sources,
        live,
        direct_settlement_fixture::state_source(root).as_path(),
    );
    parse_agent_transaction_demo_config(&fixture_env(root)).expect("configuration")
}

fn write_inputs(root: &Path) {
    std::fs::create_dir_all(root.join("extension")).expect("fixture root");
    for (name, byte) in [
        ("agent-a.key", "11"),
        ("agent-b.key", "22"),
        ("agent-c.key", "33"),
    ] {
        std::fs::write(root.join(name), byte.repeat(32)).expect("agent key");
    }
    let payer = (0_u8..64).collect::<Vec<_>>();
    std::fs::write(
        root.join("payer.json"),
        serde_json::to_vec(&payer).expect("payer"),
    )
    .expect("payer file");
    std::fs::write(root.join("extension/index.ts"), "export default {}").expect("extension");
}

fn fixture_env(root: &Path) -> BTreeMap<String, String> {
    let mut env = base_env();
    for (name, file) in input_paths() {
        env.insert(name.to_owned(), root.join(file).display().to_string());
    }
    fake_local_runtime::configure(root, &mut env);
    env
}

fn input_paths() -> [(&'static str, &'static str); 8] {
    [
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
    ]
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
