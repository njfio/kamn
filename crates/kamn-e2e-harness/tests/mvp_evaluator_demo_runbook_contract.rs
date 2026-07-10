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
