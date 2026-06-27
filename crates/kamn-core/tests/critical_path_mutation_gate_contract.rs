use std::path::{Path, PathBuf};

const GROUP_CHANNEL_SELECTOR_FILE: &str =
    "crates/kamn-core/src/group_channel_crypto/engine/sealing/encrypt.rs";
const LEGACY_GROUP_CHANNEL_SELECTOR_FILE: &str = "crates/kamn-core/src/group_channel_crypto.rs";
const CURRENT_GROUP_CHANNEL_TEST_SELECTOR: &str =
    "group_channel_crypto::tests::regression_contract_tests::encrypt_requires_key_agreement_seed";
const LEGACY_GROUP_CHANNEL_TEST_SELECTOR: &str =
    "group_channel_crypto::tests::encrypt_requires_key_agreement_seed";
const CURRENT_SERVICE_REPLAY_TEST_SELECTOR: &str = "main_tests::service_api_endpoint_tests::ingress_guard_lifecycle_contract_tests::replay_guard_contract_tests::regression_service_api_endpoint_rejects_replayed_request_nonce_for_sender";
const LEGACY_SERVICE_REPLAY_TEST_SELECTOR: &str =
    "main_tests::service_api_endpoint_tests::regression_service_api_endpoint_rejects_replayed_request_nonce_for_sender";
const CURRENT_SIGNER_SELECTOR_FILE: &str = "crates/kamn-node/src/signer/secret_provider.rs";
const LEGACY_SIGNER_LINE_SELECTOR: &str = r#"signer\\.rs:(198:33):"#;
const NONCE_GUARD: &str = "if nonce == 0";

#[test]
fn critical_path_mutation_gate_uses_extracted_group_channel_selector() {
    let script = read_repo_file("scripts/ci/run_critical_path_mutation_gate.sh");
    let source = read_repo_file(GROUP_CHANNEL_SELECTOR_FILE);

    assert_contains(&source, NONCE_GUARD, "extracted encrypt module");
    assert_contains(&script, GROUP_CHANNEL_SELECTOR_FILE, "mutation selector");
    assert_contains(
        &script,
        "--file \"$group_channel_nonce_mutation_file\"",
        "cargo-mutants file argument",
    );
    assert_not_contains(
        &script,
        &format!("grep -n '{NONCE_GUARD}' {LEGACY_GROUP_CHANNEL_SELECTOR_FILE}"),
        "legacy parent-module selector",
    );
}

#[test]
fn critical_path_mutation_gate_uses_current_test_selectors() {
    let script = read_repo_file("scripts/ci/run_critical_path_mutation_gate.sh");

    assert_contains(
        &script,
        CURRENT_GROUP_CHANNEL_TEST_SELECTOR,
        "current group-channel mutation test selector",
    );
    assert_not_contains(
        &script,
        LEGACY_GROUP_CHANNEL_TEST_SELECTOR,
        "stale group-channel mutation test selector",
    );
    assert_contains(
        &script,
        CURRENT_SERVICE_REPLAY_TEST_SELECTOR,
        "current service replay mutation test selector",
    );
    assert_not_contains(
        &script,
        LEGACY_SERVICE_REPLAY_TEST_SELECTOR,
        "stale service replay mutation test selector",
    );
}

#[test]
fn critical_path_mutation_gate_uses_current_signer_selector() {
    let script = read_repo_file("scripts/ci/run_critical_path_mutation_gate.sh");
    let source = read_repo_file(CURRENT_SIGNER_SELECTOR_FILE);

    assert_contains(
        &source,
        "ensure_kolme_live_strict_signer_secret_source_precedence(",
        "strict signer secret-source precedence implementation",
    );
    assert_contains(
        &script,
        CURRENT_SIGNER_SELECTOR_FILE,
        "current signer mutation source file",
    );
    assert_not_contains(
        &script,
        LEGACY_SIGNER_LINE_SELECTOR,
        "stale signer.rs mutation line selector",
    );
}

#[test]
fn critical_path_mutation_gate_spec_records_selector_repair_evidence() {
    let spec =
        read_repo_file("specs/7026-repair-critical-path-mutation-gate-group-channel-selector.md");

    assert_contains(
        &spec,
        "cargo test -p kamn-core --test critical_path_mutation_gate_contract",
        "Rust contract evidence",
    );
    assert_contains(
        &spec,
        "bash scripts/ci/test_run_critical_path_mutation_gate.sh` passed",
        "shell contract evidence",
    );
    assert_contains(
        &spec,
        "cargo clippy --workspace --all-targets --all-features -- -D warnings",
        "strict clippy closeout evidence",
    );
    assert_contains(&spec, "`make check` passed", "make check closeout evidence");
}

fn assert_contains(haystack: &str, needle: &str, label: &str) {
    assert!(haystack.contains(needle), "missing {label}: {needle}");
}

fn assert_not_contains(haystack: &str, needle: &str, label: &str) {
    assert!(!haystack.contains(needle), "unexpected {label}: {needle}");
}

fn read_repo_file(relative_path: &str) -> String {
    std::fs::read_to_string(repo_root().join(relative_path))
        .unwrap_or_else(|error| panic!("failed to read {relative_path}: {error}"))
}

fn repo_root() -> PathBuf {
    manifest_dir()
        .parent()
        .and_then(Path::parent)
        .expect("kamn-core manifest should live under crates/kamn-core")
        .to_path_buf()
}

fn manifest_dir() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}
