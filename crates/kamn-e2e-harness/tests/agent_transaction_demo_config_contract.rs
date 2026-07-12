use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;

use kamn_e2e_harness::parse_agent_transaction_demo_config;

#[test]
fn spec_c01_canonical_make_target_invokes_rust_supervisor() {
    let output = Command::new("make")
        .args(["-n", "demo-agent-transaction"])
        .current_dir(repo_root())
        .output()
        .expect("make dry run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("kamn-e2e-harness -- demo-agent-transaction"));
}

#[test]
fn spec_c02_preflight_parses_complete_pi_devnet_configuration() {
    let config = parse_agent_transaction_demo_config(&complete_env())
        .expect("complete canonical configuration");

    assert_eq!(config.agent_driver, "pi");
    assert_eq!(config.devnet_mode, "required");
    assert_eq!(config.solana_rpc_url, "https://api.devnet.solana.com");
    assert_eq!(config.solana_lamports, 1_000_000);
    assert_eq!(config.solana_commitment, "finalized");
    assert_eq!(config.agent_key_files.len(), 3);
}

#[test]
fn spec_c03_partial_devnet_configuration_fails_before_execution() {
    let mut env = complete_env();
    env.remove("KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_RECIPIENT_PUBKEY");

    let error = parse_agent_transaction_demo_config(&env)
        .expect_err("partial devnet configuration must fail");
    assert!(error.starts_with("AGENT_TRANSACTION_DEVNET_CONFIG_INVALID"));
}

fn complete_env() -> BTreeMap<String, String> {
    [
        ("KAMN_MVP_AGENT_DRIVER", "pi"),
        ("KAMN_MVP_DEVNET_MODE", "required"),
        ("KAMN_MVP_SOLANA_RPC_URL", "https://api.devnet.solana.com"),
        ("KAMN_MVP_LIVE_MCP_AGENT_A_KEY_FILE", "/tmp/agent-a.key"),
        ("KAMN_MVP_LIVE_MCP_AGENT_B_KEY_FILE", "/tmp/agent-b.key"),
        ("KAMN_MVP_LIVE_MCP_AGENT_C_KEY_FILE", "/tmp/agent-c.key"),
        (
            "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_KEYPAIR_FILE",
            "/tmp/devnet-payer.json",
        ),
        (
            "KAMN_SERVICE_API_LIVE_SOLANA_SETTLEMENT_RECIPIENT_PUBKEY",
            "recipient111111111111111111111111111111111111111",
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

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
