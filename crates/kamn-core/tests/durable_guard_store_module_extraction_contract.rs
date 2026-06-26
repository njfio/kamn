use std::fs;
use std::path::{Path, PathBuf};

const ROOT: &str = "src/durable_guard_store.rs";
const ROOT_MAX_LINES: usize = 180;
const MODULES: &[(&str, &str)] = &[
    ("mod bundle;", "src/durable_guard_store/bundle.rs"),
    ("mod stores;", "src/durable_guard_store/stores.rs"),
    ("mod wire_codec;", "src/durable_guard_store/wire_codec.rs"),
    (
        "mod legacy_codec;",
        "src/durable_guard_store/legacy_codec.rs",
    ),
    (
        "mod policy_codec;",
        "src/durable_guard_store/policy_codec.rs",
    ),
    (
        "#[cfg(test)]\nmod tests;",
        "src/durable_guard_store/tests.rs",
    ),
];

#[test]
fn durable_guard_store_root_is_extracted() {
    let root_path = manifest_path(ROOT);
    let root = fs::read_to_string(&root_path).expect("read durable guard store root");
    assert_root_budget(&root);
    assert_module_layout(&root);
    assert_root_delegates(&root);
}

fn assert_root_budget(root: &str) {
    let line_count = root.lines().count();
    assert!(
        line_count <= ROOT_MAX_LINES,
        "expected {ROOT} <= {ROOT_MAX_LINES} lines after extraction, found {line_count}"
    );
}

fn assert_module_layout(root: &str) {
    for (marker, module_path) in MODULES {
        assert!(
            root.contains(marker),
            "expected root to declare marker `{marker}`"
        );
        assert!(
            manifest_path(module_path).exists(),
            "expected extracted module {module_path} to exist"
        );
    }
}

fn assert_root_delegates(root: &str) {
    assert!(
        !root.contains("fn serialize_bundle(")
            && !root.contains("fn deserialize_bundle(")
            && !root.contains("fn deserialize_bundle_legacy(")
            && !root.contains("struct ChannelPolicyBuilder")
            && !root.contains("struct FileDurableGuardSnapshotStore")
            && !root.contains("struct SqliteDurableGuardSnapshotStore")
            && !root.contains("mod tests {")
            && !root.contains("fn bundle_serialization_roundtrip()"),
        "expected durable_guard_store root to delegate extracted implementation details"
    );
}

fn manifest_path(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}
