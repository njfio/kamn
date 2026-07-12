use std::collections::BTreeMap;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use kamn_e2e_harness::{
    execute_agent_transaction_demo_with_config, parse_agent_transaction_demo_config,
};

#[test]
fn spec_c01_actor_failure_cleans_every_child_and_writes_no_go() {
    let fixture = SupervisorFixture::new();
    let config = parse_agent_transaction_demo_config(&fixture.env()).expect("configuration");

    let error = execute_agent_transaction_demo_with_config(&config)
        .expect_err("Agent B failure must fail the demo");
    assert!(error.starts_with("AGENT_TRANSACTION_CHILD_FAILED"));
    for role in ["kamn-mvp-agent-a", "kamn-mvp-agent-c"] {
        assert!(fixture.root.join(format!("{role}.cleaned")).is_file());
    }
    let no_go =
        std::fs::read_to_string(fixture.root.join("demo/latest/NO-GO.txt")).expect("NO-GO report");
    assert!(no_go.contains("decision=NO-GO"));
    assert!(no_go.contains("AGENT_TRANSACTION_CHILD_FAILED"));
    assert!(!fixture.root.join("demo/latest/proof/report.json").exists());
}

struct SupervisorFixture {
    root: PathBuf,
}

impl SupervisorFixture {
    fn new() -> Self {
        let root = unique_root();
        std::fs::create_dir_all(root.join("extension")).expect("fixture root");
        for name in ["agent-a.key", "agent-b.key", "agent-c.key"] {
            std::fs::write(root.join(name), name).expect("agent key");
        }
        let payer = (0_u8..64).collect::<Vec<_>>();
        std::fs::write(
            root.join("payer.json"),
            serde_json::to_string(&payer).expect("payer JSON"),
        )
        .expect("payer file");
        std::fs::write(root.join("extension/index.ts"), "export default {}").expect("extension");
        write_fake_pi(&root);
        Self { root }
    }

    fn env(&self) -> BTreeMap<String, String> {
        let mut env = base_env();
        for (name, path) in [
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
        ] {
            env.insert(name.to_owned(), self.root.join(path).display().to_string());
        }
        env
    }
}

fn write_fake_pi(root: &std::path::Path) {
    let script = format!(
        r#"#!/bin/sh
case " $* " in *" --print "*) echo KAMN_PI_PREFLIGHT_OK; exit 0;; esac
role=""
previous=""
for arg in "$@"; do
  if [ "$previous" = "--name" ]; then role="$arg"; fi
  previous="$arg"
done
trap 'echo cleaned > "{}/$role.cleaned"; exit 0' TERM INT
if [ "$role" = "kamn-mvp-agent-b" ]; then read line; exit 9; fi
while read line; do
  echo '{{"type":"response","command":"prompt","success":true}}'
  echo '{{"type":"agent_end","messages":[]}}'
done
echo cleaned > "{}/$role.cleaned"
"#,
        root.display(),
        root.display()
    );
    let path = root.join("pi");
    std::fs::write(&path, script).expect("fake Pi");
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
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "kamn-agent-supervisor-{}-{nanos}",
        std::process::id()
    ))
}
