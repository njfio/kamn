use std::path::PathBuf;

const RUNBOOK: &str = "docs/validation/mvp-evaluator-demo.md";

#[test]
fn spec_c01_mvp_runbook_documents_pi_tool_harness_boundaries() {
    let runbook = read_runbook();

    for needle in [
        "## Optional Pi Agent Harness",
        ".pi/extensions/kamn-mvp/index.ts",
        "kamn_write_agent_harness_evidence",
        "kamn_run_demo_mvp_with_agent_evidence",
        "pi-extension-tools",
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
