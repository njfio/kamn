const NODE_MAIN_SOURCE: &str = include_str!("../main.rs");
const NODE_CARGO_MANIFEST: &str = include_str!("../../Cargo.toml");

#[test]
fn async_runtime_contract_main_entrypoint_declares_tokio_main_attribute() {
    assert!(
        NODE_MAIN_SOURCE.contains("#[tokio::main"),
        "expected kamn-node entrypoint to declare #[tokio::main] runtime boundary"
    );
}

#[test]
fn async_runtime_contract_manifest_declares_tokio_required_features() {
    assert!(
        NODE_CARGO_MANIFEST.contains("tokio ="),
        "expected kamn-node manifest to declare tokio dependency"
    );
    for feature in ["rt-multi-thread", "macros", "net", "time", "signal"] {
        assert!(
            NODE_CARGO_MANIFEST.contains(feature),
            "expected kamn-node tokio dependency to include required feature: {feature}"
        );
    }
}
