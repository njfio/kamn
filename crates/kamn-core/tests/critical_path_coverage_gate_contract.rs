use std::path::{Path, PathBuf};

const COVERAGE_GATE: &str = "scripts/ci/run_critical_path_coverage_gate.sh";
const COVERAGE_THRESHOLDS: &str = ".ci/critical-path-coverage-thresholds.json";
const EXTRACTED_GROUP_CHANNEL_TARGET: &str =
    "crates/kamn-core/src/group_channel_crypto/engine/lifecycle.rs";
const LEGACY_GROUP_CHANNEL_TARGET: &str = "crates/kamn-core/src/group_channel_crypto.rs";

const CURRENT_EXACT_SELECTORS: &[&str] = &[
    "group_channel_crypto::tests::regression_contract_tests::encrypt_requires_key_agreement_seed",
    "group_channel_crypto::tests::lifecycle_contract_tests::encrypt_decrypt_roundtrip_requires_authorized_recipient",
    "http_transport_contract_tests::response_parsing_contract_tests::regression_http_transport_maps_401_to_authorization_unavailable_error",
    "http_transport_contract_tests::submit_finality_contract_tests::functional_http_transport_includes_authorization_header_when_configured",
    "http_transport_contract_tests::submit_finality_contract_tests::regression_http_transport_timeout_maps_to_provider_timeout",
    "tls_transport_contract_tests::regression_https_transport_maps_certificate_errors_to_unavailable",
    "runtime_orchestration::full_supervisor::tests::unit_full_supervisor_http_probe_accepts_success_status",
    "runtime_orchestration::full_supervisor::tests::unit_full_supervisor_inter_tick_probes_execute_once_per_lane",
    "main_tests::service_api_endpoint_tests::residual_root_contract_tests::error_envelope_contract_tests::unit_service_api_endpoint_error_envelopes_use_reason_code_and_message_contracts",
    "signer::tests::nonce_contract_tests::unit_nonce_retry_classifier_marks_transient_provider_errors",
    "signer::tests::nonce_contract_tests::unit_nonce_retry_backoff_policy_is_deterministic_and_bounded",
    "signer::tests::adapter_contract_tests::unit_signer_private_key_parse_zeroizes_hex_buffer_on_success",
    "signer::tests::preflight_contract_tests::unit_signer_preflight_defaults_to_single_signer_quorum_ready",
    "signer::tests::preflight_contract_tests::regression_signer_preflight_rejects_stale_failover_rotation_epoch",
    "signer::tests::preflight_contract_tests::regression_signer_preflight_rejects_non_failover_rotation_epoch_regression",
    "signer::tests::secret_source_contract_tests::regression_signer_secret_source_precedence_failure_zeroizes_env_secret_buffer",
    "main_tests::signer_tests::signer_direct_profile_contract_tests::direct_signature_contract_tests::unit_kolme_live_signer_adapter_signs_and_verifies_runtime_message",
];

const STALE_EXACT_SELECTORS: &[&str] = &[
    "group_channel_crypto::tests::encrypt_requires_key_agreement_seed",
    "group_channel_crypto::tests::encrypt_decrypt_roundtrip_requires_authorized_recipient",
    "runtime_orchestration::tests::unit_full_supervisor_http_probe_accepts_success_status",
    "runtime_orchestration::tests::unit_full_supervisor_inter_tick_probes_execute_once_per_lane",
    "main_tests::service_api_endpoint_tests::unit_service_api_endpoint_error_envelopes_use_reason_code_and_message_contracts",
    "signer::tests::unit_nonce_retry_classifier_marks_transient_provider_errors",
    "signer::tests::unit_nonce_retry_backoff_policy_is_deterministic_and_bounded",
    "signer::tests::unit_signer_private_key_parse_zeroizes_hex_buffer_on_success",
    "signer::tests::unit_signer_preflight_defaults_to_single_signer_quorum_ready",
    "signer::tests::regression_signer_preflight_rejects_stale_failover_rotation_epoch",
    "signer::tests::regression_signer_preflight_rejects_non_failover_rotation_epoch_regression",
    "signer::tests::regression_signer_secret_source_precedence_failure_zeroizes_env_secret_buffer",
];

#[test]
fn critical_path_coverage_gate_uses_current_exact_selectors_after_test_splits() {
    let script = read_repo_file(COVERAGE_GATE);

    for selector in CURRENT_EXACT_SELECTORS {
        assert_contains(&script, selector, "current coverage exact selector");
    }
    for selector in STALE_EXACT_SELECTORS {
        assert_not_contains(&script, selector, "stale coverage exact selector");
    }
}

#[test]
fn critical_path_coverage_thresholds_follow_extracted_group_channel_target() {
    let thresholds = read_repo_file(COVERAGE_THRESHOLDS);

    assert_contains(
        &thresholds,
        EXTRACTED_GROUP_CHANNEL_TARGET,
        "extracted group-channel coverage target",
    );
    assert_not_contains(
        &thresholds,
        LEGACY_GROUP_CHANNEL_TARGET,
        "legacy group-channel parent target",
    );
}

#[test]
fn critical_path_coverage_gate_resolves_llvm_tools_before_running_llvm_cov() {
    let script = read_repo_file(COVERAGE_GATE);

    for marker in [
        "LLVM_COV",
        "LLVM_PROFDATA",
        "rustc --print sysroot",
        ".rustup/toolchains",
        "llvm-profdata",
    ] {
        assert_contains(&script, marker, "llvm tool resolution marker");
    }
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
