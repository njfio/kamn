use std::fs;
use std::path::{Path, PathBuf};

const DOC_PATH: &str = "docs/validation/sdk-tcp-vertical-slice.md";
const DEMO_SCRIPT_PATH: &str = "scripts/sdk/run_tcp_signed_relay_demo.sh";
const DOC_MARKERS: &[&str] = &[
    "# SDK TCP Vertical Slice",
    "two identities",
    "signed handshake",
    "successful relay",
    "replay or tamper rejection",
    "bash scripts/sdk/run_tcp_signed_relay_demo.sh",
    "What This Proves",
    "What This Does Not Prove",
];
const SCRIPT_MARKERS: &[&str] = &[
    "tcp_signed_relay_listener",
    "tcp_signed_relay_sender",
    "verified=true",
    "adapter=tcp",
    "tcp signed relay demo completed.",
];

#[test]
fn regression_sdk_tcp_vertical_slice_doc_exists_with_required_markers() {
    let doc = read_workspace_file(DOC_PATH);
    for marker in DOC_MARKERS {
        assert!(
            doc.contains(marker),
            "sdk tcp vertical-slice doc missing required marker: {marker}"
        );
    }
}

#[test]
fn regression_sdk_tcp_demo_script_retains_required_markers() {
    let script = read_workspace_file(DEMO_SCRIPT_PATH);
    for marker in SCRIPT_MARKERS {
        assert!(
            script.contains(marker),
            "sdk tcp demo script missing required marker: {marker}"
        );
    }
}

fn read_workspace_file(relative_path: &str) -> String {
    let path = workspace_root().join(relative_path);
    let path_display = path.display();
    assert!(path.exists(), "expected path to exist: {path_display}");
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {path_display}: {error}"))
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root should resolve")
}
