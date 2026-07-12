use std::collections::BTreeMap;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use kamn_e2e_harness::{
    execute_agent_transaction_demo_with_config, parse_agent_transaction_demo_config,
};

#[test]
fn spec_c01_actor_failure_cleans_every_child_and_writes_no_go() {
    let fixture = SupervisorFixture::new("fail");
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
    assert!(fixture.root.join("runtime.started").is_file());
    assert!(fixture.root.join("runtime.stopped").is_file());
}

#[test]
fn spec_c02_unresponsive_actor_times_out_and_cleans_every_child() {
    let fixture = SupervisorFixture::new("hang");
    let config = parse_agent_transaction_demo_config(&fixture.env()).expect("configuration");
    let started = std::time::Instant::now();

    let error = execute_agent_transaction_demo_with_config(&config)
        .expect_err("unresponsive actor must fail");
    assert!(error.starts_with("AGENT_TRANSACTION_CHILD_FAILED"));
    assert!(started.elapsed() < std::time::Duration::from_secs(1));
    for role in ["kamn-mvp-agent-a", "kamn-mvp-agent-c"] {
        assert!(fixture.root.join(format!("{role}.cleaned")).is_file());
    }
    assert_no_processes(&fixture.root);
}

#[test]
fn spec_c03_successful_rpc_children_run_the_canonical_phase_order() {
    let fixture = SupervisorFixture::new("success");
    let config = parse_agent_transaction_demo_config(&fixture.env()).expect("configuration");

    let error = execute_agent_transaction_demo_with_config(&config)
        .expect_err("missing proof artifacts must fail after actor phases");
    assert!(
        error.starts_with("AGENT_TRANSACTION_PROOF_INVALID"),
        "unexpected error: {error}"
    );
    let phases = std::fs::read_to_string(fixture.root.join("prompts.log")).expect("phase log");
    assert_eq!(
        phases.lines().collect::<Vec<_>>(),
        [
            "kamn-mvp-agent-b",
            "kamn-mvp-agent-a",
            "kamn-mvp-agent-b",
            "kamn-mvp-agent-a",
            "kamn-mvp-agent-b",
            "kamn-mvp-agent-a",
            "kamn-mvp-agent-c",
        ]
    );
    assert_no_processes(&fixture.root);
}

struct SupervisorFixture {
    root: PathBuf,
    endpoint: String,
}

impl SupervisorFixture {
    fn new(pi_mode: &str) -> Self {
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
        write_fake_pi(&root, pi_mode);
        let endpoint = format!("http://127.0.0.1:{}", free_port());
        write_fake_runtime(&root);
        std::fs::write(root.join("kamn-mcp-server"), "stub").expect("MCP binary");
        Self { root, endpoint }
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
        env.insert(
            "KAMN_MVP_AGENT_TRANSACTION_RPC_TIMEOUT_MS".to_owned(),
            "100".to_owned(),
        );
        env.insert(
            "KAMN_MVP_LOCAL_NODE_BINARY".to_owned(),
            self.root.join("kamn-node").display().to_string(),
        );
        env.insert(
            "KAMN_MVP_LIVE_MCP_BINARY".to_owned(),
            self.root.join("kamn-mcp-server").display().to_string(),
        );
        env.insert("KAMN_MVP_LIVE_MCP_ENDPOINT".to_owned(), self.endpoint.clone());
        env
    }
}

fn write_fake_runtime(root: &std::path::Path) {
    let script = format!(
        r#"#!/bin/sh
port=""
previous=""
for arg in "$@"; do
  if [ "$previous" = "--api-bind" ]; then port="${{arg##*:}}"; fi
  previous="$arg"
done
exec python3 -c 'import signal,socket,sys,time
root=sys.argv[1]; port=int(sys.argv[2])
open(root+"/runtime.started","w").write(str(__import__("os").getpid()))
server=socket.socket(); server.setsockopt(socket.SOL_SOCKET,socket.SO_REUSEADDR,1); server.bind(("127.0.0.1",port)); server.listen()
def stop(*_):
 open(root+"/runtime.stopped","w").write("stopped"); server.close(); sys.exit(0)
signal.signal(signal.SIGTERM,stop)
while True: time.sleep(.05)' "{}" "$port"
"#,
        root.display()
    );
    write_executable(root.join("kamn-node"), script.as_str());
}

fn write_fake_pi(root: &std::path::Path, mode: &str) {
    let b_branch = match mode {
        "hang" => "if [ \"$role\" = \"kamn-mvp-agent-b\" ]; then read line; sleep 2; exit 9; fi",
        "fail" => "if [ \"$role\" = \"kamn-mvp-agent-b\" ]; then read line; exit 9; fi",
        _ => "",
    };
    let script = format!(
        r#"#!/bin/sh
case " $* " in *" --print "*) echo KAMN_PI_PREFLIGHT_OK; exit 0;; esac
role=""
previous=""
for arg in "$@"; do
  if [ "$previous" = "--name" ]; then role="$arg"; fi
  previous="$arg"
done
echo $$ > "{}/$role.pid"
trap 'echo cleaned > "{}/$role.cleaned"; exit 0' TERM INT
{b_branch}
while read line; do
  echo "$role" >> "{}/prompts.log"
  echo '{{"type":"response","command":"prompt","success":true}}'
  if [ "$role" = "kamn-mvp-agent-b" ]; then
    echo '{{"type":"agent_end","messages":[{{"did":"kamn:did:agent:b"}}]}}'
  else
    echo '{{"type":"agent_end","messages":[]}}'
  fi
done
echo cleaned > "{}/$role.cleaned"
"#,
        root.display(),
        root.display(),
        root.display(),
        root.display()
    );
    let path = root.join("pi");
    std::fs::write(&path, script).expect("fake Pi");
    let mut permissions = std::fs::metadata(&path).expect("metadata").permissions();
    permissions.set_mode(0o700);
    std::fs::set_permissions(path, permissions).expect("permissions");
}

fn assert_no_processes(root: &std::path::Path) {
    for role in ["kamn-mvp-agent-a", "kamn-mvp-agent-b", "kamn-mvp-agent-c"] {
        let pid = std::fs::read_to_string(root.join(format!("{role}.pid")))
            .expect("child pid")
            .trim()
            .to_owned();
        let status = std::process::Command::new("kill")
            .args(["-0", pid.as_str()])
            .status()
            .expect("kill probe");
        assert!(!status.success(), "child {pid} survived cleanup");
    }
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

fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .expect("free port")
        .local_addr()
        .expect("local address")
        .port()
}
