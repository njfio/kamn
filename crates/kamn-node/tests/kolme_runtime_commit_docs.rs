const DOC: &str = include_str!("../../../docs/architecture/kolme-runtime-commit.md");

#[test]
fn doc_contains_signer_provenance_failure_taxonomy_markers() {
    assert!(DOC.contains("### Signer Provenance Failure Taxonomy"));
    assert!(DOC.contains("production_signer_key_source_env_local_forbidden"));
    assert!(DOC.contains("fallback_signer_secret_present_violation"));
    assert!(DOC.contains("managed_signer_raw_private_key_forbidden"));
    assert!(DOC.contains("managed_signer_backend_required_missing"));
    assert!(DOC.contains("managed_signer_key_reference_missing"));
    assert!(DOC.contains("managed_signer_key_reference_invalid"));
    assert!(DOC.contains("managed_signer_public_key_marker_missing"));
    assert!(DOC.contains("managed_signer_public_key_marker_invalid"));
    assert!(DOC.contains("managed_signer_backend_response_provenance_missing"));
    assert!(DOC.contains("managed_signer_backend_response_provenance_malformed"));
    assert!(DOC.contains("managed_signer_backend_response_provenance_mismatch"));
    assert!(DOC.contains("runtime_signer_key_source_profile_pair_disallowed"));
    assert!(DOC.contains("runtime_signer_rotation_epoch_stale"));
    assert!(DOC.contains("runtime_signer_attestation_quorum_shortfall"));
    assert!(DOC.contains("runtime_signer_quorum_linkage_violation"));
    assert!(DOC.contains("runtime_signer_failover_attestation_required_approvals_insufficient"));
    assert!(DOC.contains("runtime_signer_failover_attestation_previous_profile_not_approved"));
    assert!(DOC.contains("signer_quorum_linkage_contract_version=v1"));
    assert!(DOC.contains("signer_quorum_linked"));
}
