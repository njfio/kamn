use std::path::{Path, PathBuf};

const BRIDGE_ENVELOPES_SOURCE: &str = "src/bridge_adapter/models/envelopes.rs";
const UNQUALIFIED_BRIDGE_ADAPTER_LINK: &str = "[`BridgeAdapter`]";
const QUALIFIED_BRIDGE_ADAPTER_LINK: &str =
    "[`BridgeAdapter`](crate::bridge_adapter::BridgeAdapter)";

#[test]
fn bridge_envelope_docs_link_bridge_adapter_trait() {
    let source = read_crate_file(BRIDGE_ENVELOPES_SOURCE);

    assert!(
        !source.contains(UNQUALIFIED_BRIDGE_ADAPTER_LINK),
        "bridge envelope docs must not use unresolved BridgeAdapter links"
    );
    assert!(
        source.contains(QUALIFIED_BRIDGE_ADAPTER_LINK),
        "bridge envelope docs must link to the crate-qualified BridgeAdapter trait"
    );
}

fn read_crate_file(relative_path: &str) -> String {
    std::fs::read_to_string(crate_root().join(relative_path))
        .unwrap_or_else(|error| panic!("failed to read {relative_path}: {error}"))
}

fn crate_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}
