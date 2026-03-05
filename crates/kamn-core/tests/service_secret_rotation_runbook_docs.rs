use std::{fs, path::PathBuf};

const RUNBOOK_PATH: &str = "docs/ops/runbooks/service-secret-rotation.md";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root should resolve")
}

fn read_repo_text(rel_path: &str) -> String {
    let path = repo_root().join(rel_path);
    fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!("expected {} to be readable: {}", path.display(), error)
    })
}

#[test]
fn runbook_contains_versioned_rotation_markers_and_phases() {
    let runbook = read_repo_text(RUNBOOK_PATH);
    assert!(runbook.contains("runbook_schema_version=kamn.ops.service-secret-rotation-runbook.v1"));
    assert!(runbook.contains("## Key Generation"));
    assert!(runbook.contains("## Staged Rollout"));
    assert!(runbook.contains("## Rollback"));
    assert!(runbook.contains("## Verification Checklist"));
}

#[test]
fn runbook_contains_required_env_ownership_boundaries() {
    let runbook = read_repo_text(RUNBOOK_PATH);
    assert!(runbook.contains("KAMN_SERVICE_API_AUTH_PRIVATE_KEY_HEX"));
    assert!(runbook.contains("KAMN_SERVICE_API_AUTH_PUBLIC_KEY_HEX"));
    assert!(runbook.contains("KAMN_SIGNER_PRIVATE_KEY_HEX"));
    assert!(runbook.contains("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX"));
    assert!(runbook.contains("owner_boundary.security_team"));
    assert!(runbook.contains("owner_boundary.runtime_ops"));
}

#[test]
fn runbook_links_are_present_in_ops_and_contributor_docs() {
    let deployment = read_repo_text("docs/ops/deployment.md");
    let contributing = read_repo_text(".github/CONTRIBUTING.md");
    assert!(deployment.contains("docs/ops/runbooks/service-secret-rotation.md"));
    assert!(contributing.contains("docs/ops/runbooks/service-secret-rotation.md"));
}

#[test]
fn runbook_contains_validation_commands_aligned_with_runtime_contracts() {
    let runbook = read_repo_text(RUNBOOK_PATH);
    assert!(runbook.contains("cargo test -p kamn-node main_tests::runtime_tests::regression_kolme_live_signer_key_source_policy_rejects_fallback_secret_path_with_deterministic_reason_code -- --exact --nocapture"));
    assert!(runbook.contains("bash scripts/runtime/validate_service_api_request_auth_live.sh"));
    assert!(runbook.contains("cargo test -p kamn-core --test service_secret_rotation_runbook_docs"));
}
