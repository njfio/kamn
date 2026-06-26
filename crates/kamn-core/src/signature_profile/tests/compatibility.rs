use super::super::{
    baseline_signature_algorithm, baseline_signature_for_fields, baseline_signature_profile_id,
    legacy_signature_for_fields, parse_signature_profile_metadata,
    signature_matches_supported_profile_for_fields,
    signature_profile_compatibility_fixtures_for_fields, SignatureProfileMetadata,
    BASELINE_SIGNATURE_PROFILE_ID, LEGACY_SIGNATURE_PROFILE_ID, UNKNOWN_SIGNATURE_ALGORITHM_ID,
};

const SOURCE: &str = include_str!("../fixtures.rs");

#[test]
fn baseline_signature_profile_is_deterministic() {
    let signature_a = baseline_signature_for_fields("agent-a", 1, "state:genesis", "payload-1");
    let signature_b = baseline_signature_for_fields("agent-a", 1, "state:genesis", "payload-1");
    assert_eq!(signature_a, signature_b);
}

#[test]
fn baseline_signature_profile_includes_nonce_and_payload_length() {
    let signature = baseline_signature_for_fields("agent-a", 9, "state:x", "abcdef");
    assert_eq!(
        signature,
        "sig:deterministic-v1:baseline-v1:agent-a:9:state:x:6"
    );
}

#[test]
fn baseline_signature_profile_id_helper_matches_constant() {
    assert_eq!(
        baseline_signature_profile_id(),
        BASELINE_SIGNATURE_PROFILE_ID
    );
}

#[test]
fn legacy_signature_profile_fixture_is_non_versioned() {
    let signature = legacy_signature_for_fields("agent-a", 9, "state:x", "abcdef");
    assert_eq!(signature, "sig:agent-a:9:state:x:6");
}

#[test]
fn signature_profile_fixture_matrix_marks_only_baseline_v1_as_supported() {
    let fixtures = signature_profile_compatibility_fixtures_for_fields(
        "agent-a",
        1,
        "state:genesis",
        "payload-1",
    );
    assert_eq!(fixtures.len(), 4);
    assert_eq!(fixtures[0].fixture_id, BASELINE_SIGNATURE_PROFILE_ID);
    assert_eq!(fixtures[1].fixture_id, LEGACY_SIGNATURE_PROFILE_ID);
    assert_eq!(fixtures[2].fixture_id, "baseline-v0");
    assert_eq!(fixtures[3].fixture_id, "unknown-algorithm+baseline-v1");
    assert!(fixtures[0].should_verify);
    assert!(!fixtures[1].should_verify);
    assert!(!fixtures[2].should_verify);
    assert!(!fixtures[3].should_verify);
}

#[test]
fn baseline_signature_profile_algorithm_helper_matches_constant() {
    assert_eq!(baseline_signature_algorithm(), "deterministic-v1");
}

#[test]
fn parse_signature_profile_metadata_extracts_algorithm_and_profile() {
    let signature = baseline_signature_for_fields("agent-a", 1, "state:genesis", "payload-1");
    assert_eq!(
        parse_signature_profile_metadata(&signature),
        Some(SignatureProfileMetadata {
            algorithm: "deterministic-v1".to_owned(),
            profile_id: BASELINE_SIGNATURE_PROFILE_ID.to_owned(),
        })
    );
}

#[test]
fn parse_signature_profile_metadata_extracts_legacy_tags_and_rejects_malformed_signatures() {
    assert_eq!(
        parse_signature_profile_metadata("sig:agent-a:1:state:genesis:9"),
        Some(SignatureProfileMetadata {
            algorithm: "agent-a".to_owned(),
            profile_id: "1".to_owned(),
        })
    );
    assert_eq!(
        parse_signature_profile_metadata("sig:deterministic-v1:baseline-v1"),
        None
    );
    assert_eq!(parse_signature_profile_metadata("bad"), None);
}

#[test]
fn signature_profile_matcher_rejects_unknown_algorithm_fixture() {
    let signature = format!(
        "sig:{}:{}:{}:{}:{}:{}",
        UNKNOWN_SIGNATURE_ALGORITHM_ID,
        BASELINE_SIGNATURE_PROFILE_ID,
        "agent-a",
        1,
        "state:genesis",
        "payload-1".len()
    );
    assert!(!signature_matches_supported_profile_for_fields(
        &signature,
        "agent-a",
        1,
        "state:genesis",
        "payload-1"
    ));
}

#[test]
fn signature_profile_matcher_accepts_baseline_and_rejects_migration_fixtures() {
    let fixtures = signature_profile_compatibility_fixtures_for_fields(
        "agent-a",
        1,
        "state:genesis",
        "payload-1",
    );
    for fixture in fixtures {
        assert_eq!(
            signature_matches_supported_profile_for_fields(
                &fixture.signature,
                "agent-a",
                1,
                "state:genesis",
                "payload-1"
            ),
            fixture.should_verify,
            "fixture {} should map to deterministic compatibility expectation",
            fixture.fixture_id
        );
    }
}

#[test]
fn regression_requires_constant_time_signature_profile_compare() {
    assert!(
        SOURCE.contains("crate::constant_time_eq::constant_time_eq_str("),
        "signature profile matcher should use the scoped constant-time helper"
    );
    assert!(
        !SOURCE.contains(
            [
                "signature == baseline_signature_for_fields(",
                "sender, nonce, state_hash, payload)",
            ]
            .concat()
            .as_str(),
        ),
        "signature profile matcher must not use direct signature equality"
    );
}
