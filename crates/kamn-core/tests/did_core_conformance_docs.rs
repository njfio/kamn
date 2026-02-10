const PROFILE: &str = include_str!("../../../docs/foundation/did-core-conformance-kamn-method.md");

#[test]
fn profile_contains_did_core_requirement_mapping() {
    assert!(PROFILE.contains("## DID Core Requirement Mapping"));
    assert!(PROFILE.contains("| DID Core Element | Requirement Level | kamn:did Status | Notes |"));
    assert!(PROFILE.contains("| id | REQUIRED | covered | DID subject identifier is mandatory. |"));
    assert!(PROFILE.contains("| verificationMethod | REQUIRED | covered | At least one verification method is required. |"));
    assert!(PROFILE.contains(
        "| authentication | REQUIRED | covered | Authentication relationship must be present. |"
    ));
    assert!(PROFILE
        .contains("| service | OPTIONAL | partial | Service rules remain profile-constrained. |"));
}

#[test]
fn profile_contains_conformance_decisions_and_open_questions() {
    assert!(PROFILE.contains("## Conformance Decisions"));
    assert!(PROFILE.contains("Decision: require verificationMethod and capabilityInvocation."));
    assert!(PROFILE.contains("Decision: reject unsupported DID method prefixes."));
    assert!(PROFILE.contains(
        "Decision: canonical service endpoint is `kamn://messaging/<method-specific-id>` with lowercase scheme/authority and a single normalized path segment."
    ));
    assert!(PROFILE.contains(
        "Decision: endpoint queries/fragments and multi-segment paths are non-conformant."
    ));
    assert!(PROFILE.contains(
        "Decision: unsupported or mixed verification method algorithms remain non-conformant for `kamn:did` baseline profile."
    ));
}

#[test]
fn profile_contains_candidate_test_vectors_and_downstream_categories() {
    assert!(PROFILE.contains("## Candidate Test Vectors"));
    assert!(PROFILE.contains("Vector-C1: valid kamn:did document with required relationships."));
    assert!(PROFILE.contains("Vector-C2: valid update preserving DID subject continuity."));
    assert!(PROFILE.contains(
        "Vector-C3: canonicalization normalizes uppercase scheme/authority/identifier to canonical endpoint form."
    ));
    assert!(PROFILE.contains("Vector-N1: document missing id is rejected."));
    assert!(PROFILE.contains("Vector-N2: unsupported verification method algorithm is rejected."));
    assert!(PROFILE.contains(
        "Vector-N3: service endpoint with unsupported scheme, query/fragment, or multi-segment path is rejected."
    ));
    assert!(PROFILE.contains("## Downstream Test Category Mapping"));
    assert!(PROFILE.contains("Unit: schema-field validator mappings."));
    assert!(PROFILE.contains("Functional: DID document acceptance and rejection examples."));
    assert!(PROFILE.contains("Integration: DID registry interaction expectations."));
    assert!(PROFILE.contains("Regression: previously non-conformant examples remain rejected."));
}

#[test]
fn regression_requires_missing_id_rejection_rule() {
    // Regression: #81
    assert!(PROFILE.contains("Previously non-conformant document (missing id) must be rejected."));
}

#[test]
fn profile_contains_service_endpoint_canonicalization_contract_lane() {
    assert!(PROFILE.contains("## Service Endpoint Canonicalization Conformance Contract"));
    assert!(PROFILE.contains("run_service_endpoint_canonicalization_contract_lane.sh"));
    assert!(PROFILE.contains("generate_service_endpoint_canonicalization_evidence_bundle.sh"));
    assert!(PROFILE.contains("check_service_endpoint_canonicalization_policy.sh"));
    assert!(PROFILE.contains("run_service_endpoint_canonicalization_matrix.py"));
    assert!(PROFILE
        .contains("fixtures/did_core_conformance/service_endpoint_canonicalization_vectors.json"));
    assert!(PROFILE.contains("did_service_endpoint_canonicalization_reason_codes:GO:v1"));
    assert!(PROFILE.contains("did_service_endpoint_canonicalization_reason_codes:NO-GO:v1"));
}

#[test]
fn regression_requires_service_endpoint_non_canonical_rejection_rule() {
    // Regression: #1000
    assert!(PROFILE.contains(
        "non-canonical service endpoint scheme/authority/path combinations must remain rejected (`Regression: #1000`)."
    ));
}
