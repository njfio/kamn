use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate dir")
        .parent()
        .expect("repo root")
        .to_path_buf()
}

fn assert_contains(path: &str, marker: &str) {
    let full_path = repo_root().join(path);
    let content = fs::read_to_string(&full_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", full_path.display()));
    assert!(
        content.contains(marker),
        "expected marker `{marker}` in {}",
        full_path.display()
    );
}

#[test]
fn governance_coverage_expansion_surface_exists() {
    assert_contains(
        "crates/kamn-governance/tests/governance_parameter_policy_fail_closed.rs",
        "parameter_change_rejects_unsupported_target_version_fail_closed",
    );
    assert_contains(
        "crates/kamn-governance/tests/governance_parameter_policy_fail_closed.rs",
        "parameter_change_rejects_value_outside_declared_bounds_fail_closed",
    );
    assert_contains(
        "crates/kamn-governance/tests/operator_actions_fail_closed.rs",
        "read_history_denied_records_audit_entry_fail_closed",
    );
    assert_contains(
        "crates/kamn-governance/tests/operator_actions_fail_closed.rs",
        "revoke_binding_denied_records_audit_entry_fail_closed",
    );
}
