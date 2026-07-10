use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crate dir should have workspace root")
        .to_path_buf()
}

fn source_path(relative: &str) -> PathBuf {
    repo_root().join(relative)
}

fn read(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

fn assert_root_shell_markers(root_text: &str) {
    for marker in [
        "mod anti_spam;",
        "mod request_auth;",
        "mod scope_policy;",
        "mod support;",
        "mod tests;",
    ] {
        assert!(
            root_text.contains(marker),
            "expected root shell marker `{marker}`"
        );
    }
}

fn assert_root_shell_budget(root_path: &Path, root_text: &str) {
    let line_count = root_text.lines().count();
    assert!(
        line_count <= 180,
        "expected {} to be <= 180 lines after extraction, got {line_count}",
        root_path.display()
    );
}

fn assert_root_shell_excludes_inline_markers(root_text: &str) {
    for marker in [
        "fn require_valid_sender_did_header(",
        "fn authorize_service_api_request_with_legacy_policy(",
        "fn resolve_signer_public_key_for_request(",
        "pub(super) fn enforce_request_scope_policy(",
        "pub(super) async fn enforce_sender_anti_spam(",
        "pub(super) fn map_anti_spam_rejection_to_reasoned_error(",
        "fn required_scope_for_route(",
        "mod tests {",
    ] {
        assert!(
            !root_text.contains(marker),
            "expected root shell to exclude inline marker `{marker}`"
        );
    }
}

fn assert_expected_modules(root: &Path) {
    for relative in [
        "anti_spam.rs",
        "request_auth.rs",
        "scope_policy.rs",
        "support.rs",
        "tests.rs",
    ] {
        let module_path = root.join(relative);
        assert!(
            module_path.exists(),
            "expected extracted module {}",
            module_path.display()
        );
        let module_text = read(&module_path);
        assert!(
            module_text.lines().count() <= 200,
            "expected {} to stay within 200 lines, got {}",
            module_path.display(),
            module_text.lines().count()
        );
    }
}

#[test]
fn service_api_auth_root_is_extracted() {
    let root_path = source_path("crates/kamn-node/src/service_api_endpoint/auth.rs");
    let root_text = read(&root_path);

    assert_root_shell_markers(&root_text);
    assert_root_shell_budget(&root_path, &root_text);
    assert_root_shell_excludes_inline_markers(&root_text);
    assert_expected_modules(&source_path(
        "crates/kamn-node/src/service_api_endpoint/auth",
    ));
}

#[test]
fn escrow_release_revalidation_preserves_authorization_response() {
    let source = std::fs::read_to_string(source_path(
        "crates/kamn-node/src/service_api_endpoint/middleware_impl/http_routes/mutations/\
             update_routes/state_routes.rs",
    ))
    .expect("escrow release route source should exist");

    assert!(source.contains("Err(response) => return *response"));
    assert!(!source.contains(".map_err(|_| \"escrow release authorization changed"));
}
