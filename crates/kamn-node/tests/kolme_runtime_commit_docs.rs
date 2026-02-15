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
}
