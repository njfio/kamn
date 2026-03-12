use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn agent_upgrade_workflow_root_is_extracted() {
    let repo_root = workspace_root();
    let root = repo_root.join("crates/kamn-core/src/agent_upgrade_workflow.rs");
    let root_contents = fs::read_to_string(&root).expect("read root module");
    assert_file_max_lines(&root, 180);

    for marker in [
        "mod audit;",
        "mod engine;",
        "mod models;",
        "mod support;",
        "#[cfg(test)]\nmod tests;",
    ] {
        assert!(root_contents.contains(marker), "missing root marker: {marker}");
    }

    for rel in [
        "crates/kamn-core/src/agent_upgrade_workflow/audit.rs",
        "crates/kamn-core/src/agent_upgrade_workflow/engine.rs",
        "crates/kamn-core/src/agent_upgrade_workflow/models.rs",
        "crates/kamn-core/src/agent_upgrade_workflow/support.rs",
        "crates/kamn-core/src/agent_upgrade_workflow/tests.rs",
    ] {
        let path = repo_root.join(rel);
        assert!(path.is_file(), "missing extracted module: {}", path.display());
        assert_file_max_lines(&path, 200);
    }

    for marker in [
        "pub struct AgentUpgradeWorkflowConfig",
        "pub struct AgentUpgradeProposalDraft",
        "pub enum AgentUpgradeProposalState",
        "pub struct AgentDrivenUpgradeWorkflow",
        "pub fn submit_agent_proposal(",
        "pub fn approve_human_review(",
        "pub fn submit_for_governance(",
        "pub enum AgentUpgradeWorkflowError",
        "fn validate_did(",
        "mod tests {",
    ] {
        assert!(!root_contents.contains(marker), "root still contains moved marker: {marker}");
    }
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crates/kamn-core")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

fn assert_file_max_lines(path: &Path, max_lines: usize) {
    let content = fs::read_to_string(path).expect("read file");
    let lines = content.lines().count();
    assert!(
        lines <= max_lines,
        "extracted module exceeds {max_lines} lines: {}",
        path.display()
    );
}
