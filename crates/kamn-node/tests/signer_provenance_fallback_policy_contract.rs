use std::collections::BTreeSet;

const CI_STRATEGY_DOC: &str = include_str!("../../../docs/ci/strategy.md");
const OPS_CONFIGURATION_DOC: &str = include_str!("../../../docs/ops/configuration.md");
const KOLME_RUNTIME_COMMIT_DOC: &str =
    include_str!("../../../docs/architecture/kolme-runtime-commit.md");
const MAIN_SRC: &str = include_str!("../src/main.rs");
const RUNTIME_ORCHESTRATION_SRC: &str = include_str!("../src/runtime_orchestration.rs");
const MANAGED_BACKEND_SRC: &str = include_str!("../src/signer/managed_backend.rs");

const REQUIRED_KEY_SOURCE_REASON_CODES: &[&str] = &[
    "production_signer_key_source_env_local_forbidden",
    "fallback_signer_secret_present_violation",
];

const REQUIRED_PROVENANCE_REASON_CODES: &[&str] = &[
    "managed_signer_backend_response_provenance_missing",
    "managed_signer_backend_response_provenance_malformed",
    "managed_signer_backend_response_provenance_mismatch",
];

const REQUIRED_CI_POLICY_MARKERS: &[&str] = &[
    "### Signer Provenance and Fallback-Prohibition Docs/Config Parity Contract",
    "signer_provenance_fallback_policy_contract_status=active",
    "signer_provenance_fallback_policy_contract_version=v1",
    "signer_provenance_fallback_policy_required_markers_csv=runtime_signer_key_source_policy_reason_codes_csv,managed_signer_backend_response_provenance_missing,managed_signer_backend_response_provenance_malformed,managed_signer_backend_response_provenance_mismatch",
    "cargo test -p kamn-node --test signer_provenance_fallback_policy_contract -- --nocapture",
];

#[test]
fn unit_extracts_runtime_signer_key_source_reason_codes_from_ops_configuration() {
    let value = extract_marker_value(
        OPS_CONFIGURATION_DOC,
        "runtime_signer_key_source_policy_reason_codes_csv",
    )
    .expect("ops configuration must define runtime signer key-source reason-code CSV");
    let entries = parse_csv_entries(value.as_str());

    for reason_code in REQUIRED_KEY_SOURCE_REASON_CODES {
        assert!(
            entries.contains(*reason_code),
            "runtime signer key-source reason-code CSV missing required reason code: {reason_code}"
        );
    }
}

#[test]
fn functional_docs_declare_signer_provenance_fallback_policy_contract_markers() {
    for marker in REQUIRED_CI_POLICY_MARKERS {
        assert!(
            CI_STRATEGY_DOC.contains(marker),
            "ci strategy docs missing signer provenance/fallback policy marker: {marker}"
        );
    }
}

#[test]
fn integration_signer_provenance_fallback_reason_codes_remain_in_source_and_docs_parity() {
    for reason_code in REQUIRED_KEY_SOURCE_REASON_CODES {
        assert!(
            MAIN_SRC.contains(reason_code) || RUNTIME_ORCHESTRATION_SRC.contains(reason_code),
            "runtime signer key-source reason code missing from source taxonomy: {reason_code}"
        );
        assert!(
            OPS_CONFIGURATION_DOC.contains(reason_code),
            "ops configuration docs missing runtime signer key-source reason code: {reason_code}"
        );
    }

    for reason_code in REQUIRED_PROVENANCE_REASON_CODES {
        assert!(
            MANAGED_BACKEND_SRC.contains(reason_code),
            "managed backend source missing provenance reason code: {reason_code}"
        );
        assert!(
            KOLME_RUNTIME_COMMIT_DOC.contains(reason_code),
            "kolme runtime commit docs missing provenance reason code: {reason_code}"
        );
    }
}

#[test]
fn regression_parity_contract_links_fallback_prohibition_with_provenance_taxonomy() {
    let ops_reason_codes_csv = extract_marker_value(
        OPS_CONFIGURATION_DOC,
        "runtime_signer_key_source_policy_reason_codes_csv",
    )
    .expect("ops configuration must declare runtime signer key-source reason-code CSV");
    let ops_reason_codes = parse_csv_entries(ops_reason_codes_csv.as_str());
    let ci_required_markers_csv = extract_marker_value(
        CI_STRATEGY_DOC,
        "signer_provenance_fallback_policy_required_markers_csv",
    )
    .expect("ci strategy docs must declare signer provenance/fallback required-marker CSV");
    let ci_required_markers = parse_csv_entries(ci_required_markers_csv.as_str());

    assert!(
        ops_reason_codes.contains("fallback_signer_secret_present_violation"),
        "fallback prohibition reason code must remain in runtime signer key-source docs taxonomy"
    );
    assert!(
        ci_required_markers.contains("runtime_signer_key_source_policy_reason_codes_csv"),
        "ci strategy contract must require runtime signer key-source reason-code CSV marker"
    );
    assert!(
        ci_required_markers.contains("managed_signer_backend_response_provenance_missing")
            && ci_required_markers.contains("managed_signer_backend_response_provenance_malformed")
            && ci_required_markers.contains("managed_signer_backend_response_provenance_mismatch"),
        "ci strategy contract must require signer provenance reason markers"
    );
}

fn extract_marker_value(document: &str, key: &str) -> Option<String> {
    let marker = format!("{key}=");
    let marker_index = document.find(marker.as_str())?;
    let remainder = &document[(marker_index + marker.len())..];
    let value = remainder
        .split(['`', '\n', '\r'])
        .next()
        .unwrap_or_default()
        .trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}

fn parse_csv_entries(value: &str) -> BTreeSet<&str> {
    value
        .split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .collect()
}
