pub const BASELINE_SIGNATURE_PROFILE_ID: &str = "baseline-v1";
pub const LEGACY_SIGNATURE_PROFILE_ID: &str = "legacy-unversioned";

pub fn baseline_signature_profile_id() -> &'static str {
    BASELINE_SIGNATURE_PROFILE_ID
}

pub fn baseline_signature_for_fields(
    sender: &str,
    nonce: u64,
    state_hash: &str,
    payload: &str,
) -> String {
    format!(
        "sig:{}:{}:{}:{}:{}",
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
        "sig:baseline-v0:{}:{}:{}:{}",
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
    ]
}

pub fn signature_matches_supported_profile_for_fields(
    signature: &str,
    sender: &str,
    nonce: u64,
    state_hash: &str,
    payload: &str,
) -> bool {
    signature == baseline_signature_for_fields(sender, nonce, state_hash, payload)
}

#[cfg(test)]
mod tests {
    use super::{
        baseline_signature_for_fields, baseline_signature_profile_id, legacy_signature_for_fields,
        signature_matches_supported_profile_for_fields,
        signature_profile_compatibility_fixtures_for_fields, BASELINE_SIGNATURE_PROFILE_ID,
        LEGACY_SIGNATURE_PROFILE_ID,
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
        assert_eq!(signature, "sig:baseline-v1:agent-a:9:state:x:6");
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
        assert_eq!(fixtures.len(), 3);
        assert_eq!(fixtures[0].fixture_id, BASELINE_SIGNATURE_PROFILE_ID);
        assert_eq!(fixtures[1].fixture_id, LEGACY_SIGNATURE_PROFILE_ID);
        assert_eq!(fixtures[2].fixture_id, "baseline-v0");
        assert!(fixtures[0].should_verify);
        assert!(!fixtures[1].should_verify);
        assert!(!fixtures[2].should_verify);
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
