use std::fs;
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root")
        .to_path_buf()
}

fn module_path(relative: &str) -> PathBuf {
    workspace_root().join(relative)
}

fn file_text(relative: &str) -> String {
    fs::read_to_string(module_path(relative)).unwrap_or_else(|err| {
        panic!("failed to read {relative}: {err}")
    })
}

fn file_lines(relative: &str) -> usize {
    file_text(relative).lines().count()
}

#[test]
fn data_layer_m8_compliance_lifecycle_root_is_extracted() {
    let root = "crates/kamn-core/src/data_layer_m8_compliance_lifecycle.rs";
    let root_text = file_text(root);

    let expected_modules = [
        "models.rs",
        "policy.rs",
        "registry.rs",
        "lifecycle.rs",
        "errors.rs",
        "tests.rs",
    ];

    for module in expected_modules {
        let module_rel = format!(
            "crates/kamn-core/src/data_layer_m8_compliance_lifecycle/{module}"
        );
        assert!(module_path(&module_rel).exists(), "missing extracted module {module_rel}");
        assert!(
            file_lines(&module_rel) <= 200,
            "extracted module {module_rel} exceeds 200 lines"
        );
        let marker = format!("mod {}", module.trim_end_matches(".rs"));
        assert!(
            root_text.contains(&marker),
            "root shell missing module marker {marker}"
        );
    }

    for legacy_marker in [
        "pub enum DataLayerM8RetentionClass",
        "pub struct DataLayerM8ComplianceRegistry",
        "pub enum DataLayerM8ComplianceError",
        "pub fn data_layer_m8_retention_window_seconds",
        "#[cfg(test)]",
        "mod tests",
    ] {
        assert!(
            !root_text.contains(legacy_marker),
            "root shell still contains legacy marker {legacy_marker}"
        );
    }

    assert!(
        file_lines(root) <= 180,
        "root shell should be <= 180 lines, got {}",
        file_lines(root)
    );
}
