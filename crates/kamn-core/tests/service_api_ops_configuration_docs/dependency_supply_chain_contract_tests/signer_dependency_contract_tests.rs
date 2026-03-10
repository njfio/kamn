use super::*;

const METADATA_GOVERNANCE_REMEDIATION_MARKERS: &[&str] = &[
    "## Dependency-License Metadata Governance CI/Local Boundary Contract (Issue #4035)",
    "metadata_governance_reason_taxonomy_version=kamn.ci.dependency-license-metadata-governance-reason-taxonomy.v1",
    "metadata_governance_reason_codes_csv=expected_license_empty,no_crate_manifests_found,license_policy_file_not_found,license_policy_marker_mismatch,manifest_not_found,manifest_invalid_toml,package_section_missing,license_missing,license_mismatch,metadata_governance_local_heavy_opt_in_required",
    "metadata_governance_remediation_map_version=v1",
    "cargo test -p kamn-core --test ci_strategy_docs doc_enforces_dependency_license_metadata_remediation_markers_cover_reason_codes -- --exact",
    "Regression: #4035",
];

const METADATA_GOVERNANCE_REMEDIATION_CODES: &[&str] = &[
    "expected_license_empty",
    "no_crate_manifests_found",
    "license_policy_file_not_found",
    "license_policy_marker_mismatch",
    "manifest_not_found",
    "manifest_invalid_toml",
    "package_section_missing",
    "license_missing",
    "license_mismatch",
    "metadata_governance_local_heavy_opt_in_required",
];

#[test]
fn service_api_ops_configuration_contains_signer_secret_zeroization_controls() {
    assert!(
        DOC.contains("## Signer Secret Decode Buffer Zeroization Controls (Issues #4165, #4166)")
    );
    assert!(DOC.contains("signer_secret_source_precedence_zeroization_status=verified"));
    assert!(DOC.contains("signer_private_key_parse_zeroization_status=verified"));
    assert!(DOC.contains("signer_transient_key_material_zeroization_status=verified"));
    assert!(DOC.contains("signer_secret_source_precedence_violation"));
    assert!(DOC.contains("managed_signer_private_key_adapter_unsupported"));
    assert!(DOC.contains(
        "signer::tests::regression_signer_secret_source_precedence_failure_zeroizes_env_secret_buffer"
    ));
    assert!(DOC.contains(
        "signer::tests::unit_build_kolme_live_managed_signing_key_zeroizes_transient_key_material"
    ));
    assert!(DOC.contains("Regression: #4165"));
    assert!(DOC.contains("Regression: #4166"));
}

#[test]
fn service_api_ops_configuration_contains_dependency_license_metadata_governance_remediation_markers(
) {
    assert_doc_contains_all(METADATA_GOVERNANCE_REMEDIATION_MARKERS);
    assert_doc_contains_prefixed_entries("metadata_governance_remediation", METADATA_GOVERNANCE_REMEDIATION_CODES);
}
