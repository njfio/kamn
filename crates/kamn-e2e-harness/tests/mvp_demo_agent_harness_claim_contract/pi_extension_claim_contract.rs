use std::path::Path;

const DELEGATED_MCP_CONFIG: &str = ".pi/extensions/kamn-mvp/mcp-session-config.ts";
const PI_EXTENSION_SOURCES: &[&str] = &[
    ".pi/extensions/kamn-mvp/index.ts",
    ".pi/extensions/kamn-mvp/evidence.ts",
    ".pi/extensions/kamn-mvp/actor-receipts.ts",
    ".pi/extensions/kamn-mvp/live-mcp-tools.ts",
    ".pi/extensions/kamn-mvp/live-transaction-tools.ts",
    ".pi/extensions/kamn-mvp/pi-transaction-tools.ts",
    ".pi/extensions/kamn-mvp/mcp-session.ts",
    DELEGATED_MCP_CONFIG,
    ".pi/extensions/kamn-mvp/live-task-workflow.ts",
    ".pi/extensions/kamn-mvp/live-task-coordination.ts",
    ".pi/extensions/kamn-mvp/live-task-coordination-tools.ts",
    ".pi/extensions/kamn-mvp/restricted-task-observation.ts",
];
const PI_EXTENSION_MARKERS: &[&str] = &[
    "kamn_verify_mvp_report",
    "agentHarnessEvidencePath",
    "--agent-harness-evidence",
    "kamn_inspect_mvp_report_boundaries",
    "kamn_write_agent_harness_evidence",
    "kamn_run_demo_mvp_with_agent_evidence",
    "three_agent_boundary",
    "three_agent_actor_rehearsal",
    "three_agent_actor_tool_receipts",
    "three_agent_actor_observation_receipts",
    "kamn_agent_a_register",
    "kamn_agent_a_invoke_transaction",
    "kamn_agent_b_register",
    "kamn_agent_b_accept_task",
    "kamn_agent_c_verify_three_agent_proof",
    "invoke_transaction",
    "accept_task",
    "kamn_live_agent_a_register",
    "kamn_live_agent_a_query_profile",
    "query_agent_profile",
    "session_shutdown",
    "KAMN_MVP_LIVE_MCP_BINARY",
    "KAMN_MVP_LIVE_MCP_AGENT_A_KEY_FILE",
    "kamn_live_agent_b_register",
    "kamn_live_agent_a_create_task",
    "kamn_live_agent_b_accept_task",
    "kamn_live_agent_a_query_task",
    "kamn_live_agent_b_query_task",
    "kamn_live_agent_a_fund_escrow",
    "kamn_live_agent_a_release_escrow",
    "kamn_live_agent_a_query_participant_projection",
    "kamn_live_agent_b_complete_task",
    "kamn_live_agent_b_query_participant_projection",
    "kamn_live_agent_c_register",
    "kamn_live_agent_c_receive_task_handoff",
    "kamn_live_agent_c_query_verifier_projection",
    "kamn_live_agent_a_write_transaction_evidence",
    "kamn_live_agent_b_write_transaction_evidence",
    "kamn_live_agent_c_write_transaction_evidence",
    "kamn_live_verify_pi_transaction_actors",
    "KAMN_MVP_LIVE_MCP_AGENT_B_NAME",
    "KAMN_MVP_LIVE_MCP_AGENT_B_KEY_FILE",
    "KAMN_MVP_LIVE_MCP_AGENT_C_NAME",
    "KAMN_MVP_LIVE_MCP_AGENT_C_KEY_FILE",
    "real local-only task lifecycle",
    "kamn_live_agent_a_publish_task_handoff",
    "kamn_live_agent_b_receive_task_handoff",
    "kamn_live_agent_a_wait_for_task_acceptance",
    "kamn_live_agent_b_write_task_receipt",
    "kamn_live_verify_independent_actor_receipts",
    "KAMN_MVP_LIVE_TASK_HANDOFF_FILE",
    "KAMN_MVP_LIVE_TASK_AGENT_A_RECEIPT_FILE",
    "KAMN_MVP_LIVE_TASK_AGENT_B_RECEIPT_FILE",
    "real local-only independent Pi actors",
    "kamn_live_agent_c_verify_restricted_task_observation",
    "KAMN_MVP_LIVE_TASK_AGENT_C_OBSERVATION_FILE",
    "real local-only independent Agent C artifact observation",
];

#[test]
fn spec_c08_project_local_pi_extension_registers_kamn_tools() {
    let source = pi_extension_source();

    for marker in PI_EXTENSION_MARKERS {
        assert!(source.contains(marker), "missing Pi tool marker: {marker}");
    }
}

#[test]
fn spec_c08_pi_extension_inventory_includes_delegated_mcp_config() {
    assert!(
        PI_EXTENSION_SOURCES.contains(&DELEGATED_MCP_CONFIG),
        "Pi extension source inventory must include delegated MCP configuration"
    );
}

fn pi_extension_source() -> String {
    PI_EXTENSION_SOURCES
        .iter()
        .map(|path| read_extension_source(path))
        .collect::<Vec<_>>()
        .join("\n")
}

fn read_extension_source(path: &str) -> String {
    std::fs::read_to_string(workspace_root().join(path))
        .unwrap_or_else(|_| panic!("KAMN Pi extension file should exist: {path}"))
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate should live under crates/")
        .parent()
        .expect("workspace root should contain crates/")
}
