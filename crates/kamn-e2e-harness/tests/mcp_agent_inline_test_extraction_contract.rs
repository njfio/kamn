use std::path::Path;

const MCP_AGENT_ROOT_SOURCE: &str = include_str!("../src/drivers/mcp_agent.rs");
const MCP_AGENT_TEST_MODULE_FILE: &str = "src/drivers/mcp_agent_tests.rs";
const ROOT_STAGED_MAX_LINES: usize = 2_600;

#[test]
fn regression_mcp_agent_root_removes_inline_cfg_test_module() {
    assert!(
        !MCP_AGENT_ROOT_SOURCE.contains("#[cfg(test)]\nmod tests {")
            && !MCP_AGENT_ROOT_SOURCE.contains("#[cfg(test)]\r\nmod tests {"),
        "mcp_agent.rs must not keep the inline cfg(test) module"
    );
}

#[test]
fn regression_mcp_agent_root_declares_extracted_test_module() {
    assert!(
        MCP_AGENT_ROOT_SOURCE.contains("#[cfg(test)] mod mcp_agent_tests;")
            || MCP_AGENT_ROOT_SOURCE.contains("#[cfg(test)]\nmod mcp_agent_tests;"),
        "mcp_agent.rs must declare the extracted mcp_agent_tests submodule"
    );
}

#[test]
fn regression_mcp_agent_extracted_test_module_file_exists() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let full_path = manifest_dir.join(MCP_AGENT_TEST_MODULE_FILE);
    assert!(
        full_path.exists(),
        "expected extracted MCP-agent test module file missing: {}",
        full_path.display()
    );
}

#[test]
fn regression_mcp_agent_root_respects_staged_line_budget() {
    let line_count = MCP_AGENT_ROOT_SOURCE.lines().count();
    assert!(
        line_count <= ROOT_STAGED_MAX_LINES,
        "mcp_agent.rs should stay within the staged line budget: {line_count} > {ROOT_STAGED_MAX_LINES}"
    );
}
