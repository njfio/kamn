use std::path::PathBuf;

use serde_json::Value;

#[test]
fn spec_c01_readme_opens_with_current_mvp_front_door() {
    let readme = read_root_readme();

    require_all(
        "README.md",
        &readme,
        &[
            "KAMN (Kolme AI Agent Messaging Network)",
            "## What This Repository Contains",
            "## Quickstart",
            "### Canonical Pi/devnet transaction",
            "`make demo-mvp` is a local-only compatibility proof",
            "make demo-mvp",
            "cargo run -p kamn-e2e-harness -- verify-mvp-demo",
            "--report .kamn/demo/latest/proof/report.json",
            "docs/validation/mvp-evaluator-demo.md",
            "## Claim Boundaries",
            "local-only",
            "devnet-backed",
            "Solana devnet",
            "developer-test tokens",
            "production readiness",
            "## Architecture Map",
            "## For AI Agents And Maintainers",
        ],
    );

    require_ordered_pairs(
        &readme,
        &[
            ("## What This Repository Contains", "## Quickstart"),
            ("## Quickstart", "## Claim Boundaries"),
            ("## Claim Boundaries", "## Architecture Map"),
            ("## Architecture Map", "## For AI Agents And Maintainers"),
        ],
    );
}

#[test]
fn spec_c02_architecture_index_classifies_mvp_and_test_surfaces() {
    let architecture = read_repo_file("docs/architecture/README.md");

    require_all(
        "docs/architecture/README.md",
        &architecture,
        &[
            "## MVP Surface Classes",
            "canonical runtime",
            "compatibility",
            "local-only",
            "dry-run",
            "placeholder",
            "roadmap",
            "## Test Taxonomy",
            "behavior",
            "integration",
            "live",
            "docs-contract",
            "legacy compatibility",
        ],
    );
}

#[test]
fn spec_c03_superseded_manifest_entrypoints_are_absent() {
    let manifest = read_repo_file("fixtures/ci/superseded_script_deletion_manifest.json");
    let payload: Value = serde_json::from_str(&manifest).expect("valid deletion manifest");
    let deletions = payload["deletions"].as_array().expect("deletions array");

    for deletion in deletions {
        let path = deletion["script_path"].as_str().expect("script path");
        assert!(
            !repo_root().join(path).exists(),
            "superseded path remains: {path}"
        );
    }
}

#[test]
fn spec_c04_developer_reference_separates_canonical_and_local_demo_lanes() {
    let reference = read_repo_file("docs/developer/readme-contract-reference.md");

    require_all(
        "docs/developer/readme-contract-reference.md",
        &reference,
        &[
            "make demo-agent-transaction - canonical Pi/devnet transaction demo",
            "make demo-mvp - local-only compatibility proof",
        ],
    );
}

fn read_root_readme() -> String {
    std::fs::read_to_string(root_readme()).expect("root README.md should be readable")
}

fn root_readme() -> PathBuf {
    repo_root().join("README.md")
}

fn read_repo_file(path: &str) -> String {
    std::fs::read_to_string(repo_root().join(path))
        .unwrap_or_else(|error| panic!("{path} should be readable: {error}"))
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn require_all(label: &str, content: &str, needles: &[&str]) {
    for needle in needles {
        require_contains(label, content, needle);
    }
}

fn require_contains(label: &str, content: &str, needle: &str) {
    assert!(
        content.contains(needle),
        "{label} is missing required MVP front-door content: {needle}"
    );
}

fn require_ordered_pairs(readme: &str, pairs: &[(&str, &str)]) {
    for (before, after) in pairs {
        require_order(readme, before, after);
    }
}

fn require_order(readme: &str, before: &str, after: &str) {
    let before_index = readme
        .find(before)
        .unwrap_or_else(|| panic!("README.md is missing required heading: {before}"));
    let after_index = readme
        .find(after)
        .unwrap_or_else(|| panic!("README.md is missing required heading: {after}"));
    assert!(
        before_index < after_index,
        "README.md should place `{before}` before `{after}`"
    );
}
