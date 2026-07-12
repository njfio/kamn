use std::collections::BTreeMap;
use std::path::PathBuf;

use kamn_e2e_harness::{
    execute_agent_transaction_demo_with_config, parse_agent_transaction_demo_config,
};

#[path = "support/fake_local_runtime.rs"]
mod fake_local_runtime;
#[path = "support/fake_supervisor_pi.rs"]
mod fake_supervisor_pi;

#[test]
fn spec_c01_actor_failure_cleans_every_child_and_writes_no_go() {
    let _guard = test_lock();
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
    let _guard = test_lock();
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
    let _guard = test_lock();
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

#[test]
fn spec_c04_tool_execution_failure_stops_the_transaction_immediately() {
    let _guard = test_lock();
    let fixture = SupervisorFixture::new("tool-error");
    let config = parse_agent_transaction_demo_config(&fixture.env()).expect("configuration");

    let error = execute_agent_transaction_demo_with_config(&config)
        .expect_err("failed tool execution must fail the actor phase");
    assert!(error.starts_with("AGENT_TRANSACTION_CHILD_FAILED"));
    assert!(error.contains("Pi tool failed: kamn_live_agent_b_register"));
    assert!(!fixture.root.join("demo/latest/proof/report.json").exists());
}

struct SupervisorFixture {
    root: PathBuf,
}

impl SupervisorFixture {
    fn new(pi_mode: &str) -> Self {
        let root = unique_root();
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
            serde_json::to_string(&payer).expect("payer JSON"),
        )
        .expect("payer file");
        std::fs::write(root.join("extension/index.ts"), "export default {}").expect("extension");
        fake_supervisor_pi::write(&root, pi_mode);
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
            ("KAMN_MVP_AGENT_TRANSACTION_STAGING_ROOT", "staging"),
        ] {
            env.insert(name.to_owned(), self.root.join(path).display().to_string());
        }
        env.insert(
            "KAMN_MVP_AGENT_TRANSACTION_RPC_TIMEOUT_MS".to_owned(),
            "100".to_owned(),
        );
        fake_local_runtime::configure(&self.root, &mut env);
        env
    }
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
    static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "kamn-agent-supervisor-{}-{nanos}-{id}",
        std::process::id()
    ))
}

fn test_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
