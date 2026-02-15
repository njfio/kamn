use std::fs;

fn read_repo_file(path: &str) -> String {
    let root = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{root}/{path}");
    fs::read_to_string(&full_path).unwrap_or_else(|error| {
        panic!("failed to read {path}: {error}");
    })
}

#[test]
fn dependency_contract_enables_native_libp2p_transport_feature_for_kamn_core() {
    let manifest = read_repo_file("Cargo.toml");
    assert!(
        manifest.contains(
            "kamn-core = { path = \"../kamn-core\", features = [\"libp2p-live-transport\"] }"
        ),
        "kamn-node dependency contract must enable native libp2p transport feature for kamn-core"
    );
}
