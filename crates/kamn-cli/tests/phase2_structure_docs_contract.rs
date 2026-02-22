use std::path::{Path, PathBuf};

const DOC: &str =
    include_str!("../../../docs/research/e2e-live-testing-prd-phase2-gap-analysis.md");

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .to_path_buf()
}

#[test]
fn spec_c01_phase2_required_paths_exist() {
    let root = repo_root();
    let required_paths = [
        "crates/kamn-mcp-server/Cargo.toml",
        "crates/kamn-mcp-server/src/main.rs",
        "crates/kamn-mcp-server/src/tools.rs",
        "crates/kamn-mcp-server/src/config.rs",
        "crates/kamn-cli/Cargo.toml",
        "crates/kamn-cli/src/main.rs",
        "crates/kamn-cli/src/commands/register.rs",
        "crates/kamn-cli/src/commands/send_message.rs",
        "crates/kamn-cli/src/commands/create_channel.rs",
        "crates/kamn-cli/src/commands/list_messages.rs",
        "crates/kamn-cli/src/commands/query_message.rs",
        "crates/kamn-cli/src/commands/create_task.rs",
        "crates/kamn-cli/src/commands/accept_task.rs",
        "crates/kamn-cli/src/commands/complete_task.rs",
        "crates/kamn-cli/src/commands/fund_escrow.rs",
        "crates/kamn-cli/src/commands/release_escrow.rs",
        "crates/kamn-cli/src/commands/verify_proof.rs",
        "crates/kamn-cli/src/commands/health.rs",
    ];

    for path in required_paths {
        assert!(root.join(path).is_file(), "required path missing: {path}");
    }
}

#[test]
fn spec_c08_phase2_docs_markers_present() {
    assert!(DOC.contains("phase2_required_paths_total=21"));
    assert!(DOC.contains("phase2_required_paths_present_before=0"));
    assert!(DOC.contains("phase2_required_paths_missing_before=21"));
    assert!(DOC.contains("phase2_required_paths_present_after=21"));
    assert!(DOC.contains("phase2_required_paths_missing_after=0"));
    assert!(DOC.contains("phase2_mcp_tool_inventory_count=12"));
    assert!(DOC.contains("phase2_cli_subcommand_inventory_count=12"));
    assert!(DOC.contains("phase2_status_after=implemented"));
}
