use std::fs;
use std::path::{Path, PathBuf};

const ROOT_MAX_LINES: usize = 180;

#[test]
fn data_layer_m4_escrow_integration_root_is_extracted() {
    let src_root = source_root();
    let root = src_root.join("data_layer_m4_escrow_integration.rs");
    let root_source = fs::read_to_string(&root).expect("read root source");

    assert_root_budget(&root, &root_source);
    assert_module_layout(&src_root);
    assert_root_markers(&root_source);
}

fn source_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn assert_root_budget(root: &Path, root_source: &str) {
    let root_lines = root_source.lines().count();
    assert!(
        root_lines <= ROOT_MAX_LINES,
        "expected {} <= {} lines after extraction, found {}",
        root.display(),
        ROOT_MAX_LINES,
        root_lines
    );
}

fn assert_module_layout(src_root: &Path) {
    for relative in expected_modules() {
        let path = src_root.join(relative);
        assert!(
            path.exists(),
            "missing extracted module: {}",
            path.display()
        );
    }
}

fn assert_root_markers(root_source: &str) {
    for marker in expected_markers() {
        assert!(
            root_source.contains(marker),
            "expected root shell to contain marker: {marker}"
        );
    }
}

fn expected_modules() -> &'static [&'static str] {
    &[
        "data_layer_m4_escrow_integration/models.rs",
        "data_layer_m4_escrow_integration/transitions.rs",
        "data_layer_m4_escrow_integration/visibility.rs",
        "data_layer_m4_escrow_integration/settlement_evidence.rs",
        "data_layer_m4_escrow_integration/validation.rs",
        "data_layer_m4_escrow_integration/tests.rs",
    ]
}

fn expected_markers() -> &'static [&'static str] {
    &[
        "mod models;",
        "mod transitions;",
        "mod visibility;",
        "mod settlement_evidence;",
        "mod validation;",
        "#[cfg(test)]",
        "mod tests;",
    ]
}
