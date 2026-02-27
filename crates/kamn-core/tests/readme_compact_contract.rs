use std::fs;
use std::path::PathBuf;

const README: &str = include_str!("../../../README.md");

fn repo_file(path: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..");
    fs::read_to_string(root.join(path))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

#[test]
fn spec_c01_readme_line_cap_is_200_lines_or_less() {
    let line_count = README.lines().count();
    assert!(
        line_count <= 200,
        "README.md must stay <= 200 lines for onboarding; found {line_count}"
    );
}

#[test]
fn spec_c02_readme_keeps_onboarding_entrypoints() {
    assert!(README.contains("# KAMN"));
    assert!(README.contains("## Quickstart"));
    assert!(README.contains("docs/architecture/README.md"));
}

#[test]
fn spec_c03_readme_references_contract_detail_docs() {
    assert!(README.contains("docs/developer/readme-contract-reference.md"));
    let contract_doc = repo_file("docs/developer/readme-contract-reference.md");
    assert!(
        contract_doc.contains("README Contract Reference"),
        "contract reference doc must expose a stable heading"
    );
}
