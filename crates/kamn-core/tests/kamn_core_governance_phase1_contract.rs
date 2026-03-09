use std::fs;

fn repo_file(path: &str) -> String {
    let root = env!("CARGO_MANIFEST_DIR");
    let full = format!("{root}/../../{path}");
    fs::read_to_string(full).unwrap_or_default()
}

#[test]
fn governance_phase1_contract_declares_workspace_and_core_dependencies() {
    let workspace = repo_file("Cargo.toml");
    let core_cargo = repo_file("crates/kamn-core/Cargo.toml");
    let governance_cargo = repo_file("crates/kamn-governance/Cargo.toml");

    assert!(
        workspace.contains("\"crates/kamn-governance\""),
        "workspace Cargo.toml should register crates/kamn-governance"
    );
    assert!(
        core_cargo.contains("kamn-governance = { path = \"../kamn-governance\" }"),
        "kamn-core Cargo.toml should depend on kamn-governance"
    );
    assert!(
        !governance_cargo.contains("kamn-core"),
        "kamn-governance must not depend back on kamn-core"
    );
}

#[test]
fn governance_phase1_contract_extracts_governance_primitives_and_keeps_dashboards_in_core() {
    let governance_lib = repo_file("crates/kamn-governance/src/lib.rs");
    let workflow_shim = repo_file("crates/kamn-core/src/governance_workflow.rs");
    let binding_shim = repo_file("crates/kamn-core/src/operator_binding.rs");
    let actions_shim = repo_file("crates/kamn-core/src/operator_actions.rs");
    let dashboard_api = repo_file("crates/kamn-core/src/operator_dashboard_api.rs");
    let dashboard_ui = repo_file("crates/kamn-core/src/operator_dashboard_ui.rs");

    for marker in [
        "pub mod governance_workflow;",
        "pub mod operator_binding;",
        "pub mod operator_actions;",
    ] {
        assert!(
            governance_lib.contains(marker),
            "kamn-governance lib should declare extracted module: {marker}"
        );
    }

    assert!(
        workflow_shim.contains("pub use kamn_governance::governance_workflow::*;"),
        "kamn-core governance_workflow shim should re-export from kamn_governance"
    );
    assert!(
        binding_shim.contains("pub use kamn_governance::operator_binding::*;"),
        "kamn-core operator_binding shim should re-export from kamn_governance"
    );
    assert!(
        actions_shim.contains("pub use kamn_governance::operator_actions::*;"),
        "kamn-core operator_actions shim should re-export from kamn_governance"
    );

    assert!(
        dashboard_api.contains("pub struct OperatorDashboardApi"),
        "operator_dashboard_api should remain implemented in kamn-core during phase1"
    );
    assert!(
        dashboard_ui.contains("pub struct OperatorDashboardUi"),
        "operator_dashboard_ui should remain implemented in kamn-core during phase1"
    );
}

#[test]
fn governance_phase1_contract_documents_new_crate_boundary() {
    let arch_index = repo_file("docs/architecture/README.md");
    let governance_doc = repo_file("docs/architecture/kamn-governance.md");

    assert!(
        arch_index.contains("docs/architecture/kamn-governance.md"),
        "architecture index should link kamn-governance.md"
    );
    assert!(
        governance_doc.contains("kamn_governance_phase1_scope=governance_workflow,operator_binding,operator_actions"),
        "kamn-governance doc should record the phase1 scope"
    );
    assert!(
        governance_doc.contains("kamn_governance_phase1_retained_in_core=operator_dashboard_api,operator_dashboard_ui"),
        "kamn-governance doc should record the retained dashboard modules"
    );
}
