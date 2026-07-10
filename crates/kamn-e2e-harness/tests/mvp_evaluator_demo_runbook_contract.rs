use std::path::PathBuf;

const RUNBOOK: &str = "docs/validation/mvp-evaluator-demo.md";

#[test]
fn spec_c01_mvp_runbook_documents_pi_tool_harness_boundaries() {
    let runbook = read_runbook();

    for needle in [
        "## Optional Pi Agent Harness",
        ".pi/extensions/kamn-mvp/index.ts",
        "kamn_write_agent_harness_evidence",
        "--agent-harness-evidence /tmp/kamn-pi-mcp-agent-harness-evidence.json",
        "one canonical report remains unchanged",
        "kamn_run_demo_mvp_with_agent_evidence",
        "kamn_agent_a_register",
        "kamn_agent_a_invoke_transaction",
        "kamn_agent_b_register",
        "kamn_agent_b_accept_task",
        "kamn_agent_c_verify_three_agent_proof",
        "three_agent_actor_tool_receipts",
        "three_agent_actor_observation_receipts",
        "agent_a_observation_receipt_digest",
        "agent_c_verifier_observation_receipt_digest",
        "pi-extension-tools",
        "three-agent-transcript.json",
        "raw participant-private payloads stay redacted",
        "does not prove generic Pi MCP protocol support",
        "kamn_live_agent_a_register",
        "kamn_live_agent_a_query_profile",
        "KAMN_MVP_LIVE_MCP_BINARY",
        "KAMN_MVP_LIVE_MCP_ENDPOINT",
        "KAMN_MVP_LIVE_MCP_AGENT_A_NAME",
        "KAMN_MVP_LIVE_MCP_AGENT_A_KEY_FILE",
        "persistent live MCP process",
        "local-only identity durability",
        "does not prove task, escrow, settlement, or asset movement",
        "openssl rand -hex 32 > /tmp/kamn-pi-agent-a.key",
        "--runtime-mode api",
        "--api-bind 127.0.0.1:18278",
        "request nonce `1` and status `201`",
        "request nonce `2` and status `200`",
        "kamn_live_agent_b_register",
        "kamn_live_agent_a_create_task",
        "kamn_live_agent_b_accept_task",
        "kamn_live_agent_a_query_task",
        "kamn_live_agent_b_query_task",
        "KAMN_MVP_LIVE_MCP_AGENT_B_NAME",
        "KAMN_MVP_LIVE_MCP_AGENT_B_KEY_FILE",
        "real local-only task lifecycle",
        "does not prove escrow, settlement, asset movement, third-party verification, or restart durability",
        "Both registrations and task creation return `201`",
        "acceptance and both task queries return `200`",
        "kamn_live_agent_a_publish_task_handoff",
        "kamn_live_agent_b_receive_task_handoff",
        "kamn_live_agent_a_wait_for_task_acceptance",
        "kamn_live_agent_b_write_task_receipt",
        "kamn_live_verify_independent_actor_receipts",
        "KAMN_MVP_LIVE_TASK_HANDOFF_FILE",
        "KAMN_MVP_LIVE_TASK_AGENT_A_RECEIPT_FILE",
        "KAMN_MVP_LIVE_TASK_AGENT_B_RECEIPT_FILE",
        "real local-only independent Pi actors",
        "separate Pi processes",
    ] {
        assert!(runbook.contains(needle), "{RUNBOOK} missing: {needle}");
    }
}

fn read_runbook() -> String {
    std::fs::read_to_string(repo_root().join(RUNBOOK))
        .unwrap_or_else(|err| panic!("{RUNBOOK} should be readable: {err}"))
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
