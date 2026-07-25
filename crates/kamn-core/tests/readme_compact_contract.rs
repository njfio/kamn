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

fn heading_position(heading: &str) -> usize {
    README
        .find(heading)
        .unwrap_or_else(|| panic!("README.md must contain heading {heading}"))
}

fn relative_markdown_links() -> impl Iterator<Item = &'static str> {
    README
        .split("](")
        .skip(1)
        .filter_map(|suffix| suffix.split(')').next())
        .filter(|link| !link.contains("://") && !link.starts_with('#'))
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

#[test]
fn spec_c04_readme_progresses_from_human_overview_to_maintainer_depth() {
    let headings = [
        "## Why KAMN",
        "## Quickstart",
        "## How It Fits Together",
        "## Authority Flow",
        "## What Is Proven",
        "## Repository Map",
        "## Build And Verify",
        "## For Agents And Maintainers",
        "## Go Deeper",
    ];
    let positions: Vec<_> = headings.into_iter().map(heading_position).collect();
    assert!(
        positions.windows(2).all(|pair| pair[0] < pair[1]),
        "README.md headings must follow progressive disclosure order"
    );
}

#[test]
fn spec_c05_readme_exposes_architecture_and_authority_diagrams() {
    assert!(README.contains("<!-- diagram:kamn-runtime-architecture -->"));
    assert!(README.contains("<!-- diagram:receipt-authority-flow -->"));
    assert!(README.contains("flowchart LR"));
    assert!(README.contains("sequenceDiagram"));
    assert!(README.contains("Service receipts, not ambient actor trust"));
}

#[test]
fn spec_c06_readme_relative_links_resolve() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..");
    for link in relative_markdown_links() {
        let path = link.split('#').next().unwrap_or(link);
        assert!(
            root.join(path).exists(),
            "README link does not resolve: {link}"
        );
    }
}
