use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;

use kamn_e2e_harness::{
    build_pi_actor_command, parse_agent_transaction_demo_config, parse_command_args,
    AgentTransactionRole, HarnessCommand,
};

#[test]
fn spec_c00_parser_routes_canonical_agent_transaction_command() {
    let parsed = parse_command_args(["demo-agent-transaction"])
        .expect("canonical agent transaction command should parse");
    assert_eq!(parsed, HarnessCommand::DemoAgentTransaction);
}

#[test]
fn spec_c01_canonical_make_target_invokes_rust_supervisor() {
    let output = Command::new("make")
        .args(["-n", "demo-agent-transaction"])
        .current_dir(repo_root())
        .output()
        .expect("make dry run");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("cargo build -p kamn-node -p kamn-mcp-server"));
    assert!(stdout.contains("kamn-e2e-harness -- demo-agent-transaction"));
}

#[test]
fn spec_c05_make_help_separates_canonical_and_local_only_demo_lanes() {
    let output = Command::new("make")
        .arg("help")
        .current_dir(repo_root())
        .output()
        .expect("make help");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("canonical Pi/devnet transaction demo"));
    assert!(stdout.contains("local-only compatibility proof"));
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

#[test]
fn spec_c04_pi_rpc_commands_are_role_bounded_and_persistent() {
    let config = parse_agent_transaction_demo_config(&complete_env()).expect("configuration");
    let agent_a = build_pi_actor_command(&config, AgentTransactionRole::AgentA);
    let agent_b = build_pi_actor_command(&config, AgentTransactionRole::AgentB);
    let agent_c = build_pi_actor_command(&config, AgentTransactionRole::AgentC);

    for command in [&agent_a, &agent_b, &agent_c] {
        assert!(contains_pair(command, "--mode", "rpc"));
        assert!(contains_pair(command, "--provider", "openai-codex"));
        assert!(contains_pair(command, "--model", "gpt-5.5"));
        for flag in [
            "--no-session",
            "--approve",
            "--no-extensions",
            "--no-builtin-tools",
            "--no-skills",
            "--no-prompt-templates",
            "--no-context-files",
        ] {
            assert!(command.contains(&flag.to_owned()), "missing {flag}");
        }
    }
    assert!(tools(&agent_a).contains("kamn_live_agent_a_release_escrow"));
    assert!(!tools(&agent_a).contains("kamn_live_agent_b_complete_task"));
    assert!(tools(&agent_b).contains("kamn_live_agent_b_complete_task"));
    assert!(tools(&agent_b).contains("kamn_live_agent_b_query_task"));
    assert!(!tools(&agent_b).contains("kamn_live_agent_a_release_escrow"));
    assert!(tools(&agent_c).contains("kamn_live_agent_c_query_verifier_projection"));
    assert!(!tools(&agent_c).contains("kamn_live_agent_a_fund_escrow"));
}

fn contains_pair(command: &[String], flag: &str, value: &str) -> bool {
    command
        .windows(2)
        .any(|pair| pair == [flag.to_owned(), value.to_owned()])
}

fn tools(command: &[String]) -> &str {
    let index = command
        .iter()
        .position(|value| value == "--tools")
        .expect("tools flag");
    command[index + 1].as_str()
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
