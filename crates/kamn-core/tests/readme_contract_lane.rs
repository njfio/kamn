use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..");
    fs::read_to_string(root.join(path))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

fn required_snippets() -> Vec<String> {
    repo_file(".ci/readme_contract_required_snippets.txt")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(ToOwned::to_owned)
        .collect()
}

fn normalize_marker(marker: &str) -> String {
    marker.replace("\\\"", "\"").replace("\\\\", "\\")
}

#[test]
fn spec_c01_rust_lane_validates_readme_headers_and_required_markers() {
    let readme = repo_file("README.md");
    let contract_reference = repo_file("docs/developer/readme-contract-reference.md");

    let required_headers = [
        "# KAMN",
        "## What This Repository Contains",
        "## Quickstart",
        "## Workflow",
        "## Architecture Map",
        "## Contract Reference",
        "## Key Links",
    ];
    for header in required_headers {
        assert!(
            readme.contains(header),
            "README contract failed: missing header {header}"
        );
    }

    let markers = required_snippets();
    assert!(
        markers.len() >= 150,
        "expected non-trivial required marker inventory, found {}",
        markers.len()
    );
    for marker in markers {
        let marker = normalize_marker(&marker);
        assert!(
            contract_reference.contains(&marker),
            "README contract failed: missing snippet in docs/developer/readme-contract-reference.md: {marker}"
        );
    }
}

#[test]
fn spec_c02_shell_wrapper_delegates_to_rust_readme_contract_lane() {
    let script = repo_file("scripts/ci/test_readme_contract.sh");
    assert!(
        script.contains("cargo test -p kamn-core --test readme_contract_lane"),
        "shell wrapper must delegate to rust readme contract lane"
    );
    assert!(
        !script.contains("required_snippets=("),
        "shell wrapper should not own inline marker inventory"
    );
    assert!(
        !script.contains("required_headers=("),
        "shell wrapper should not own inline header inventory"
    );
}

#[test]
fn spec_c03_shell_wrapper_loc_is_slimmed_after_migration() {
    let line_count = repo_file("scripts/ci/test_readme_contract.sh")
        .lines()
        .count();
    assert!(
        line_count <= 20,
        "expected slim shell wrapper after migration, found {line_count} lines"
    );
}
