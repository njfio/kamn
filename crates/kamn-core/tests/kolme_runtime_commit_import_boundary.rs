use std::fs;

fn read_repo_file(path: &str) -> String {
    let root = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{root}/{path}");
    fs::read_to_string(&full_path).unwrap_or_else(|error| {
        panic!("failed to read {path}: {error}");
    })
}

#[test]
fn runtime_commit_import_boundary_uses_direct_kamn_kolme_traits_in_submodules() {
    let migrated_files = [
        "src/kolme_runtime_commit/adapter_backed_client.rs",
        "src/kolme_runtime_commit/api_codec.rs",
        "src/kolme_runtime_commit/block_fallback_reconciler.rs",
        "src/kolme_runtime_commit/errors.rs",
        "src/kolme_runtime_commit/finality_checker.rs",
        "src/kolme_runtime_commit/fork_finality_resolver.rs",
        "src/kolme_runtime_commit/http_transport.rs",
        "src/kolme_runtime_commit/interfaces.rs",
        "src/kolme_runtime_commit/live_provider.rs",
        "src/kolme_runtime_commit/notifications_consumer.rs",
        "src/kolme_runtime_commit/notifications_websocket.rs",
    ];

    for path in migrated_files {
        let source = read_repo_file(path);
        assert!(
            source.contains("use kamn_kolme::"),
            "expected direct kamn_kolme import in {path}"
        );
    }
}

#[test]
fn runtime_commit_import_boundary_preserves_compatibility_reexports() {
    let source = read_repo_file("src/kolme_runtime_commit.rs");
    assert!(
        source.contains("pub use kamn_kolme::{"),
        "compatibility re-export surface should remain available"
    );
    assert!(
        source.contains("KolmeRuntimeCommitProviderTransport"),
        "provider transport compatibility export should remain available"
    );
    assert!(
        source.contains("KolmeRuntimeCommitProviderError"),
        "provider error compatibility export should remain available"
    );
}
