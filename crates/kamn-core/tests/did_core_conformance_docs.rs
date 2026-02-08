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
    assert!(PROFILE.contains("## Conformance Decisions and Open Questions"));
    assert!(PROFILE.contains("Decision: require verificationMethod and capabilityInvocation."));
    assert!(PROFILE.contains("Decision: reject unsupported DID method prefixes."));
    assert!(PROFILE.contains("Open question: service endpoint canonicalization strategy."));
}

#[test]
fn profile_contains_candidate_test_vectors_and_downstream_categories() {
    assert!(PROFILE.contains("## Candidate Test Vectors"));
    assert!(PROFILE.contains("Vector-C1: valid kamn:did document with required relationships."));
    assert!(PROFILE.contains("Vector-C2: valid update preserving DID subject continuity."));
    assert!(PROFILE.contains("Vector-N1: document missing id is rejected."));
    assert!(PROFILE.contains("Vector-N2: unsupported verification method algorithm is rejected."));
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
