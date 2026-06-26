use super::{
    baseline_signature_algorithm, baseline_signature_profile_id, parse_signature_profile_metadata,
    BASELINE_SIGNATURE_PROFILE_ID, LEGACY_SIGNATURE_PROFILE_ID, UNKNOWN_SIGNATURE_ALGORITHM_ID,
};
use crate::signature_profile::encoding::baseline_signature_for_fields as render_baseline_signature;

pub fn baseline_signature_for_fields(
    sender: &str,
    nonce: u64,
    state_hash: &str,
    payload: &str,
) -> String {
    render_baseline_signature(sender, nonce, state_hash, payload)
}

pub fn legacy_signature_for_fields(
    sender: &str,
    nonce: u64,
    state_hash: &str,
    payload: &str,
) -> String {
    format!("sig:{sender}:{nonce}:{state_hash}:{}", payload.len())
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
            signature: render_baseline_signature(sender, nonce, state_hash, payload),
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
            fixture_id: "unknown-algorithm+baseline-v1",
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

    let expected_signature = render_baseline_signature(sender, nonce, state_hash, payload);
    crate::constant_time_eq::constant_time_eq_str(signature, expected_signature.as_str())
}
