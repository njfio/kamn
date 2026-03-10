use std::fs;
use std::path::PathBuf;

const MCP_AGENT_ROOT_SOURCE: &str = include_str!("../src/drivers/mcp_agent.rs");
const MCP_AGENT_DRIVER_CORE_FILE: &str = "src/drivers/mcp_agent/driver_core.rs";
const MCP_AGENT_TRANCHE_ONE_FILE: &str = "src/drivers/mcp_agent/live_probe_tranche_one.rs";
const MCP_AGENT_TRANCHE_TWO_FILE: &str = "src/drivers/mcp_agent/live_probe_tranche_two.rs";
const MCP_AGENT_TRANCHE_THREE_FILE: &str = "src/drivers/mcp_agent/live_probe_tranche_three.rs";
const MCP_AGENT_TOOL_CALL_SUPPORT_FILE: &str = "src/drivers/mcp_agent/tool_call_support.rs";
const MCP_AGENT_PROTOCOL_SUPPORT_FILE: &str = "src/drivers/mcp_agent/probe_protocol_support.rs";
const ROOT_MAX_LINES: usize = 200;
const EXTRACTED_MAX_LINES: usize = 200;

#[test]
fn regression_mcp_agent_root_declares_extracted_modules() {
    for marker in [
        "mod driver_core;",
        "mod live_probe_tranche_one;",
        "mod live_probe_tranche_two;",
        "mod live_probe_tranche_three;",
        "mod tool_call_support;",
        "mod probe_protocol_support;",
    ] {
        assert!(
            MCP_AGENT_ROOT_SOURCE.contains(marker),
            "mcp_agent.rs must declare extracted root module marker: {marker}"
        );
    }
}

#[test]
fn regression_mcp_agent_root_removes_residual_driver_probe_and_support_definitions() {
    for marker in [
        "pub struct McpAgentDriver {",
        "fn run_live_s01_mcp_probe()",
        "fn run_live_s06_mcp_proof_verification_probe()",
        "fn run_live_s11_mcp_signer_rotation_probe()",
        "fn run_live_s15_mcp_performance_smoke_probe()",
        "fn validate_s14_mcp_verify_proof_response(",
        "fn run_live_s02_mcp_tool_call(",
        "fn run_live_s15_mcp_tool_call(",
        "fn build_framed_jsonrpc_request(",
        "fn parse_framed_jsonrpc_payloads(",
        "fn escape_json_scalar(",
    ] {
        assert!(
            !MCP_AGENT_ROOT_SOURCE.contains(marker),
            "mcp_agent.rs must not keep residual root helper marker: {marker}"
        );
    }
}

#[test]
fn regression_mcp_agent_root_extracted_module_files_exist() {
    for relative_path in [
        MCP_AGENT_DRIVER_CORE_FILE,
        MCP_AGENT_TRANCHE_ONE_FILE,
        MCP_AGENT_TRANCHE_TWO_FILE,
        MCP_AGENT_TRANCHE_THREE_FILE,
        MCP_AGENT_TOOL_CALL_SUPPORT_FILE,
        MCP_AGENT_PROTOCOL_SUPPORT_FILE,
    ] {
        let full_path = manifest_dir().join(relative_path);
        assert!(
            full_path.exists(),
            "expected extracted mcp_agent root module missing: {}",
            full_path.display()
        );
    }
}

#[test]
fn regression_mcp_agent_root_respects_full_file_budget() {
    let line_count = MCP_AGENT_ROOT_SOURCE.lines().count();
    assert!(
        line_count <= ROOT_MAX_LINES,
        "mcp_agent.rs should stay within the root file budget: {line_count} > {ROOT_MAX_LINES}"
    );
}

#[test]
fn regression_mcp_agent_root_extracted_files_stay_within_line_budget() {
    let offenders = [
        MCP_AGENT_DRIVER_CORE_FILE,
        MCP_AGENT_TRANCHE_ONE_FILE,
        MCP_AGENT_TRANCHE_TWO_FILE,
        MCP_AGENT_TRANCHE_THREE_FILE,
        MCP_AGENT_TOOL_CALL_SUPPORT_FILE,
        MCP_AGENT_PROTOCOL_SUPPORT_FILE,
    ]
    .into_iter()
    .filter_map(|relative_path| {
        let full_path = manifest_dir().join(relative_path);
        let line_count = fs::read_to_string(&full_path).ok()?.lines().count();
        (line_count > EXTRACTED_MAX_LINES)
            .then(|| format!("{} ({line_count})", full_path.display()))
    })
    .collect::<Vec<String>>();

    assert!(
        offenders.is_empty(),
        "extracted mcp_agent root files exceed {EXTRACTED_MAX_LINES} LOC: {}",
        offenders.join(", ")
    );
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
