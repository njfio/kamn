use std::fs;
use std::path::{Path, PathBuf};

const DOC_PATH: &str = "docs/validation/restart-persistence-slice.md";
const DOC_MARKERS: &[&str] = &[
    "# Restart Persistence Slice",
    "integration_service_api_endpoint_persists_message_state_across_restart",
    "integration_service_api_endpoint_persists_task_and_escrow_state_across_restart",
    "integration_service_api_endpoint_persists_channel_creation_state_across_restart",
    "integration_service_api_endpoint_persists_agent_profile_query_state_across_restart",
    "integration_service_api_endpoint_recipient_mailbox_and_delivery_status_contract",
    "regression_service_api_endpoint_non_recipient_query_keeps_relayed_status_across_restart",
    "What This Proves",
    "What This Does Not Prove",
    "restart persistence",
    "not crash recovery",
];

#[test]
fn restart_persistence_doc_exists_with_required_markers() {
    assert_contains_all(read_workspace_file(DOC_PATH).as_str(), DOC_MARKERS);
}

fn assert_contains_all(doc: &str, markers: &[&str]) {
    for marker in markers {
        assert!(
            doc.contains(marker),
            "restart persistence proof doc missing marker: {}",
            marker
        );
    }
}

fn read_workspace_file(relative_path: &str) -> String {
    let path = workspace_root().join(relative_path);
    assert!(path.exists(), "expected path to exist: {}", path.display());
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {}", path.display(), error))
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root should resolve")
}
