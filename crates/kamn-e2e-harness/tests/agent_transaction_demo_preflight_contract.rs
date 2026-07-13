use std::collections::BTreeMap;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use kamn_e2e_harness::{parse_agent_transaction_demo_config, validate_agent_transaction_preflight};

#[test]
fn spec_c01_preflight_proves_files_and_pi_oauth_before_execution() {
    let fixture = PreflightFixture::new("pass");
    let config = parse_agent_transaction_demo_config(&fixture.env()).expect("configuration");

    validate_agent_transaction_preflight(&config).expect("complete preflight should pass");
}

#[test]
fn spec_c02_missing_agent_key_fails_closed() {
    let fixture = PreflightFixture::new("pass");
    std::fs::remove_file(fixture.root.join("agent-b.key")).expect("remove key");
    let config = parse_agent_transaction_demo_config(&fixture.env()).expect("configuration");

    let error = validate_agent_transaction_preflight(&config).expect_err("missing key must fail");
    assert!(error.starts_with("AGENT_TRANSACTION_AGENT_CONFIG_INVALID"));
}

#[test]
fn spec_c03_failed_pi_oauth_probe_fails_closed() {
    let fixture = PreflightFixture::new("fail");
    let config = parse_agent_transaction_demo_config(&fixture.env()).expect("configuration");

    let error = validate_agent_transaction_preflight(&config).expect_err("Pi auth must fail");
    assert!(error.starts_with("AGENT_TRANSACTION_PI_PREFLIGHT_FAILED"));
}

struct PreflightFixture {
    root: PathBuf,
}

impl PreflightFixture {
    fn new(pi_mode: &str) -> Self {
        let root = unique_root();
        std::fs::create_dir_all(root.join(".pi/extensions/kamn-mvp")).expect("fixture root");
        for name in ["agent-a.key", "agent-b.key", "agent-c.key"] {
            std::fs::write(root.join(name), format!("private-{name}")).expect("agent key");
        }
        let keypair = (0_u8..64).collect::<Vec<_>>();
        std::fs::write(
            root.join("payer.json"),
            serde_json::to_string(&keypair).expect("keypair JSON"),
        )
        .expect("payer keypair");
        std::fs::write(
            root.join(".pi/extensions/kamn-mvp/index.ts"),
            "export default {}",
        )
        .expect("extension");
        write_fake_pi(root.join("pi"), pi_mode);
        write_fake_pi(root.join("kamn-node"), "pass");
        std::fs::write(root.join("kamn-mcp-server"), "stub").expect("MCP binary");
        Self { root }
    }

    fn env(&self) -> BTreeMap<String, String> {
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
            ("KAMN_MVP_PI_EXTENSION", ".pi/extensions/kamn-mvp/index.ts"),
            ("KAMN_MVP_LOCAL_NODE_BINARY", "kamn-node"),
            ("KAMN_MVP_LIVE_MCP_BINARY", "kamn-mcp-server"),
        ] {
            env.insert(name.to_owned(), self.root.join(file).display().to_string());
        }
        env
    }
}

fn write_fake_pi(path: PathBuf, mode: &str) {
    let body = if mode == "pass" {
        "#!/bin/sh\necho KAMN_PI_PREFLIGHT_OK\n"
    } else {
        "#!/bin/sh\necho auth-failed >&2\nexit 1\n"
    };
    std::fs::write(&path, body).expect("fake Pi");
    let mut permissions = std::fs::metadata(&path).expect("metadata").permissions();
    permissions.set_mode(0o700);
    std::fs::set_permissions(path, permissions).expect("permissions");
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

fn unique_root() -> PathBuf {
    static FIXTURE_SEQUENCE: AtomicU64 = AtomicU64::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "kamn-agent-preflight-{}-{nanos}-{}",
        std::process::id(),
        FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ))
}
