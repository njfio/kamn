use std::path::{Path, PathBuf};

const RUNBOOK: &str = "docs/validation/pi-mcp-service-receipt-authority-slice.md";
const INDEX: &str = "docs/validation/current-proven-runtime-slices.md";
const RUNBOOK_MARKERS: &[&str] = &[
    "# Pi/MCP Service-Receipt Authority Slice",
    "make demo-agent-transaction",
    "kamn.mcp.authority-receipt.v1",
    "kamn.service.receipt-chain.v1",
    "task:create -> task:accept -> escrow:fund -> task:complete",
    "escrow:release-authorize -> settlement:confirmed",
    "live-service-persisted-receipt",
    "standalone verifier",
    "not production readiness",
    "not broad bridge finality",
    "not generalized external settlement",
];
const INDEX_MARKERS: &[&str] = &[
    "Pi/MCP service-receipt authority slice: `docs/validation/pi-mcp-service-receipt-authority-slice.md`",
    "proves one bounded three-role Pi/MCP transaction",
    "durable service receipts",
    "`make demo-agent-transaction`",
];

#[test]
fn pi_mcp_service_receipt_authority_runbook_is_bounded() {
    assert_contains_all(
        &read_workspace_file(RUNBOOK),
        RUNBOOK_MARKERS,
        "Pi/MCP authority runbook",
    );
}

#[test]
fn runtime_proof_index_includes_pi_mcp_authority_slice() {
    assert_contains_all(
        &read_workspace_file(INDEX),
        INDEX_MARKERS,
        "runtime proof index",
    );
}

fn assert_contains_all(content: &str, markers: &[&str], label: &str) {
    for marker in markers {
        assert!(content.contains(marker), "{label} missing marker: {marker}");
    }
}

fn read_workspace_file(relative_path: &str) -> String {
    let path = workspace_root().join(relative_path);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root should resolve")
}
