pub const BASELINE_SIGNATURE_ALGORITHM: &str = "ed25519";
pub const BASELINE_SIGNATURE_PROFILE_ID: &str = "baseline-v1";
pub const LEGACY_SIGNATURE_PROFILE_ID: &str = "legacy-unversioned";
pub const UNKNOWN_SIGNATURE_ALGORITHM_ID: &str = "secp256k1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureProfileMetadata {
    pub algorithm: String,
    pub profile_id: String,
}

pub fn baseline_signature_algorithm() -> &'static str {
    BASELINE_SIGNATURE_ALGORITHM
}

pub fn baseline_signature_profile_id() -> &'static str {
    BASELINE_SIGNATURE_PROFILE_ID
}

pub fn parse_signature_profile_metadata(signature: &str) -> Option<SignatureProfileMetadata> {
    let suffix = signature.strip_prefix("sig:")?;
    let mut segments = suffix.splitn(3, ':');
    let algorithm = segments.next()?.trim();
    let profile_id = segments.next()?.trim();
    let payload_segments = segments.next()?.trim();

    if algorithm.is_empty() || profile_id.is_empty() || payload_segments.is_empty() {
        return None;
    }

    Some(SignatureProfileMetadata {
        algorithm: algorithm.to_owned(),
        profile_id: profile_id.to_owned(),
    })
}

pub fn baseline_signature_for_fields(
    sender: &str,
    nonce: u64,
    state_hash: &str,
    payload: &str,
) -> String {
    format!(
        "sig:{}:{}:{}:{}:{}:{}",
        baseline_signature_algorithm(),
        baseline_signature_profile_id(),
        sender,
        nonce,
        state_hash,
        payload.len()
    )
}

pub fn legacy_signature_for_fields(
    sender: &str,
    nonce: u64,
    state_hash: &str,
    payload: &str,
) -> String {
    format!("sig:{}:{}:{}:{}", sender, nonce, state_hash, payload.len())
}

pub fn unknown_signature_profile_for_fields(
    sender: &str,
    nonce: u64,
    state_hash: &str,
    payload: &str,
) -> String {
    format!(
        "sig:{}:baseline-v0:{}:{}:{}:{}",
        baseline_signature_algorithm(),
        sender,
        nonce,
        state_hash,
        payload.len()
    )
}

pub fn unknown_signature_algorithm_for_fields(
    sender: &str,
    nonce: u64,
    state_hash: &str,
    payload: &str,
) -> String {
    format!(
        "sig:{}:{}:{}:{}:{}:{}",
        UNKNOWN_SIGNATURE_ALGORITHM_ID,
        baseline_signature_profile_id(),
        sender,
        nonce,
        state_hash,
        payload.len()
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureProfileCompatibilityFixture {
    pub fixture_id: &'static str,
    pub signature: String,
    pub should_verify: bool,
}

pub fn signature_profile_compatibility_fixtures_for_fields(
    sender: &str,
    nonce: u64,
    state_hash: &str,
    payload: &str,
) -> Vec<SignatureProfileCompatibilityFixture> {
    vec![
        SignatureProfileCompatibilityFixture {
            fixture_id: BASELINE_SIGNATURE_PROFILE_ID,
            signature: baseline_signature_for_fields(sender, nonce, state_hash, payload),
            should_verify: true,
        },
        SignatureProfileCompatibilityFixture {
            fixture_id: LEGACY_SIGNATURE_PROFILE_ID,
            signature: legacy_signature_for_fields(sender, nonce, state_hash, payload),
            should_verify: false,
        },
        SignatureProfileCompatibilityFixture {
            fixture_id: "baseline-v0",
            signature: unknown_signature_profile_for_fields(sender, nonce, state_hash, payload),
            should_verify: false,
        },
        SignatureProfileCompatibilityFixture {
            fixture_id: "secp256k1+baseline-v1",
            signature: unknown_signature_algorithm_for_fields(sender, nonce, state_hash, payload),
            should_verify: false,
        },
    ]
}

pub fn signature_matches_supported_profile_for_fields(
    signature: &str,
    sender: &str,
    nonce: u64,
    state_hash: &str,
    payload: &str,
) -> bool {
    let Some(metadata) = parse_signature_profile_metadata(signature) else {
        return false;
    };
    if metadata.algorithm != baseline_signature_algorithm() {
        return false;
    }
    if metadata.profile_id != baseline_signature_profile_id() {
        return false;
    }

    signature == baseline_signature_for_fields(sender, nonce, state_hash, payload)
}

#[cfg(test)]
mod tests {
    use super::{
        baseline_signature_algorithm, baseline_signature_for_fields, baseline_signature_profile_id,
        legacy_signature_for_fields, parse_signature_profile_metadata,
        signature_matches_supported_profile_for_fields,
        signature_profile_compatibility_fixtures_for_fields, BASELINE_SIGNATURE_PROFILE_ID,
        LEGACY_SIGNATURE_PROFILE_ID, UNKNOWN_SIGNATURE_ALGORITHM_ID,
    };

    #[test]
    fn baseline_signature_profile_is_deterministic() {
        let signature_a = baseline_signature_for_fields("agent-a", 1, "state:genesis", "payload-1");
        let signature_b = baseline_signature_for_fields("agent-a", 1, "state:genesis", "payload-1");
        assert_eq!(signature_a, signature_b);
    }

    #[test]
    fn baseline_signature_profile_includes_nonce_and_payload_length() {
        let signature = baseline_signature_for_fields("agent-a", 9, "state:x", "abcdef");
        assert_eq!(signature, "sig:ed25519:baseline-v1:agent-a:9:state:x:6");
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
        assert_eq!(fixtures[3].fixture_id, "secp256k1+baseline-v1");
        assert!(fixtures[0].should_verify);
        assert!(!fixtures[1].should_verify);
        assert!(!fixtures[2].should_verify);
        assert!(!fixtures[3].should_verify);
    }

    #[test]
    fn baseline_signature_profile_algorithm_helper_matches_constant() {
        assert_eq!(baseline_signature_algorithm(), "ed25519");
    }

    #[test]
    fn parse_signature_profile_metadata_extracts_algorithm_and_profile() {
        let signature = baseline_signature_for_fields("agent-a", 1, "state:genesis", "payload-1");
        assert_eq!(
            parse_signature_profile_metadata(&signature),
            Some(super::SignatureProfileMetadata {
                algorithm: "ed25519".to_owned(),
                profile_id: BASELINE_SIGNATURE_PROFILE_ID.to_owned(),
            })
        );
    }

    #[test]
    fn parse_signature_profile_metadata_extracts_legacy_tags_and_rejects_malformed_signatures() {
        assert_eq!(
            parse_signature_profile_metadata("sig:agent-a:1:state:genesis:9"),
            Some(super::SignatureProfileMetadata {
                algorithm: "agent-a".to_owned(),
                profile_id: "1".to_owned(),
            })
        );
        assert_eq!(
            parse_signature_profile_metadata("sig:ed25519:baseline-v1"),
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
}
