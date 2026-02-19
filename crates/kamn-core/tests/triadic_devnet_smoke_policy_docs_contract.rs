use std::path::Path;

const POLICY_COMMAND: &str = "check_triadic_devnet_smoke_policy.py";

fn repo_file(path: &str) -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    std::fs::read_to_string(root.join(path))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

#[test]
fn unit_triadic_devnet_policy_docs_contract_planning_doc_references_policy_command() {
    let planning_doc = repo_file("docs/planning/kolme-devnet-ops.md");
    assert!(
        planning_doc.contains(POLICY_COMMAND),
        "expected planning docs to reference triadic devnet policy checker command"
    );
}

#[test]
fn functional_triadic_devnet_policy_docs_contract_ci_strategy_references_policy_command() {
    let ci_doc = repo_file("docs/ci/strategy.md");
    assert!(
        ci_doc.contains(POLICY_COMMAND),
        "expected CI strategy docs to reference triadic devnet policy checker command"
    );
}

#[test]
fn integration_triadic_devnet_policy_docs_contract_readme_references_policy_command() {
    let readme = repo_file("README.md");
    assert!(
        readme.contains(POLICY_COMMAND),
        "expected README to reference triadic devnet policy checker command"
    );
}

#[test]
fn regression_triadic_devnet_policy_docs_contract_contract_lane_invokes_policy_command() {
    let contract_lane_impl =
        repo_file("scripts/kolme/contracts/triadic_devnet_smoke_contract_lane.py");
    assert!(
        contract_lane_impl.contains(POLICY_COMMAND),
        "expected triadic contract lane implementation to invoke triadic policy checker command"
    );
}
