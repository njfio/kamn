use k256::ecdsa::{RecoveryId, Signature, SigningKey, VerifyingKey};
use k256::elliptic_curve::rand_core::OsRng;
use k256::elliptic_curve::zeroize::Zeroize;
use std::sync::OnceLock;

/// Canonical algorithm identifier for supported baseline signatures.
pub const BASELINE_SIGNATURE_ALGORITHM: &str = "deterministic-v1";
/// Canonical profile identifier for supported baseline signatures.
pub const BASELINE_SIGNATURE_PROFILE_ID: &str = "baseline-v1";
/// Legacy unversioned profile identifier retained for compatibility fixtures.
pub const LEGACY_SIGNATURE_PROFILE_ID: &str = "legacy-unversioned";
/// Canonical unsupported algorithm identifier used in negative fixtures.
pub const UNKNOWN_SIGNATURE_ALGORITHM_ID: &str = "unknown-algorithm";
/// Canonical algorithm identifier for service-auth cryptographic signatures.
pub const SERVICE_AUTH_SIGNATURE_ALGORITHM: &str = "secp256k1";
/// Canonical profile identifier for service-auth cryptographic signatures.
pub const SERVICE_AUTH_SIGNATURE_PROFILE_ID: &str = "baseline-v2";
/// Environment variable that carries service-auth private key material (hex).
pub const SERVICE_AUTH_SIGNATURE_PRIVATE_KEY_ENV: &str = "KAMN_SERVICE_API_AUTH_PRIVATE_KEY_HEX";
/// Environment variable that carries service-auth public key material (hex).
pub const SERVICE_AUTH_SIGNATURE_PUBLIC_KEY_ENV: &str = "KAMN_SERVICE_API_AUTH_PUBLIC_KEY_HEX";

/// Error taxonomy for service-auth cryptographic signing and verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceAuthSignatureError {
    /// Required input field was empty.
    EmptyField(&'static str),
    /// Nonce must be positive.
    InvalidNonce,
    /// Signature payload did not match expected segmented form.
    InvalidSignatureFormat,
    /// Signature algorithm segment was unsupported.
    UnsupportedAlgorithm(String),
    /// Signature profile segment was unsupported.
    UnsupportedProfile(String),
    /// Signature recovery-id segment was malformed.
    InvalidRecoveryId,
    /// Signature hex payload was malformed.
    InvalidSignatureHex,
    /// Private-key hex payload was malformed.
    InvalidPrivateKeyHex,
    /// Public-key hex payload was malformed.
    InvalidPublicKeyHex,
    /// Failed to sign canonical message payload.
    SigningFailure,
    /// Failed to verify canonical message payload.
    VerificationFailure,
}

impl std::fmt::Display for ServiceAuthSignatureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyField(field) => write!(f, "{field} must not be empty"),
            Self::InvalidNonce => write!(f, "nonce must be positive"),
            Self::InvalidSignatureFormat => write!(f, "signature format is invalid"),
            Self::UnsupportedAlgorithm(value) => {
                write!(f, "unsupported signature algorithm: {value}")
            }
            Self::UnsupportedProfile(value) => write!(f, "unsupported signature profile: {value}"),
            Self::InvalidRecoveryId => write!(f, "signature recovery id is invalid"),
            Self::InvalidSignatureHex => write!(f, "signature hex payload is invalid"),
            Self::InvalidPrivateKeyHex => write!(f, "private key hex payload is invalid"),
            Self::InvalidPublicKeyHex => write!(f, "public key hex payload is invalid"),
            Self::SigningFailure => write!(f, "failed to sign canonical payload"),
            Self::VerificationFailure => write!(f, "failed to verify canonical payload"),
        }
    }
}

impl std::error::Error for ServiceAuthSignatureError {}

/// Parsed metadata extracted from a `sig:<algorithm>:<profile_id>:...` signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureProfileMetadata {
    /// Signature algorithm identifier segment.
    pub algorithm: String,
    /// Signature profile identifier segment.
    pub profile_id: String,
}

/// Returns the canonical baseline signature algorithm identifier.
pub fn baseline_signature_algorithm() -> &'static str {
    BASELINE_SIGNATURE_ALGORITHM
}

/// Returns the canonical baseline signature profile identifier.
pub fn baseline_signature_profile_id() -> &'static str {
    BASELINE_SIGNATURE_PROFILE_ID
}

/// Parses signature profile metadata from a canonical `sig:` signature string.
///
/// Returns `None` when the signature does not match the expected segmented form
/// or required segments are empty.
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

fn decode_hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(10 + (byte - b'a')),
        b'A'..=b'F' => Some(10 + (byte - b'A')),
        _ => None,
    }
}

fn decode_hex_bytes(value: &str) -> Result<Vec<u8>, ServiceAuthSignatureError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || !trimmed.len().is_multiple_of(2) {
        return Err(ServiceAuthSignatureError::InvalidSignatureHex);
    }

    let mut decoded = Vec::with_capacity(trimmed.len() / 2);
    for pair in trimmed.as_bytes().chunks_exact(2) {
        let high =
            decode_hex_nibble(pair[0]).ok_or(ServiceAuthSignatureError::InvalidSignatureHex)?;
        let low =
            decode_hex_nibble(pair[1]).ok_or(ServiceAuthSignatureError::InvalidSignatureHex)?;
        decoded.push((high << 4) | low);
    }
    Ok(decoded)
}

fn encode_hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

fn wipe_bytes(bytes: &mut [u8]) {
    bytes.zeroize();
}

/// Generates an ephemeral secp256k1 private key (hex) for non-production fallback flows.
pub fn generate_ephemeral_service_auth_private_key_hex() -> String {
    let signing_key = SigningKey::random(&mut OsRng);
    encode_hex_lower(signing_key.to_bytes().as_ref())
}

/// Returns one process-stable debug fallback private key (hex) for non-production flows.
pub fn debug_fallback_signer_private_key_hex() -> Option<&'static str> {
    static DEBUG_FALLBACK: OnceLock<Option<String>> = OnceLock::new();
    DEBUG_FALLBACK
        .get_or_init(|| {
            let candidate = generate_ephemeral_service_auth_private_key_hex();
            if service_auth_public_key_hex_from_private_key_hex(candidate.as_str()).is_ok() {
                Some(candidate)
            } else {
                None
            }
        })
        .as_deref()
}

fn canonical_service_auth_message(
    sender: &str,
    nonce: u64,
    state_hash: &str,
    payload: &str,
) -> String {
    format!(
        "sender_len={}\nsender={sender}\nnonce={nonce}\nstate_hash_len={}\nstate_hash={state_hash}\npayload_len={}\npayload={payload}",
        sender.len(),
        state_hash.len(),
        payload.len()
    )
}

/// Returns canonical service-auth signing payload for sender/nonce/state/body fields.
pub fn service_auth_signing_payload_for_fields(
    sender: &str,
    nonce: u64,
    state_hash: &str,
    payload: &str,
) -> Result<String, ServiceAuthSignatureError> {
    if sender.trim().is_empty() {
        return Err(ServiceAuthSignatureError::EmptyField("sender"));
    }
    if nonce == 0 {
        return Err(ServiceAuthSignatureError::InvalidNonce);
    }
    if state_hash.trim().is_empty() {
        return Err(ServiceAuthSignatureError::EmptyField("state_hash"));
    }
    Ok(canonical_service_auth_message(
        sender, nonce, state_hash, payload,
    ))
}

/// Builds a service-auth signature using secp256k1 over the canonical sender/nonce/state/body payload.
pub fn service_auth_sign_with_private_key_hex(
    sender: &str,
    nonce: u64,
    state_hash: &str,
    payload: &str,
    private_key_hex: &str,
) -> Result<String, ServiceAuthSignatureError> {
    if private_key_hex.trim().is_empty() {
        return Err(ServiceAuthSignatureError::EmptyField("private_key_hex"));
    }
    let message = service_auth_signing_payload_for_fields(sender, nonce, state_hash, payload)?;
    let mut private_key_bytes = decode_hex_bytes(private_key_hex)
        .map_err(|_| ServiceAuthSignatureError::InvalidPrivateKeyHex)?;
    let signing_key = match SigningKey::from_slice(private_key_bytes.as_slice()) {
        Ok(key) => key,
        Err(_) => {
            wipe_bytes(private_key_bytes.as_mut_slice());
            return Err(ServiceAuthSignatureError::InvalidPrivateKeyHex);
        }
    };
    wipe_bytes(private_key_bytes.as_mut_slice());
    let (signature, recovery_id) = signing_key
        .sign_recoverable(message.as_bytes())
        .map_err(|_| ServiceAuthSignatureError::SigningFailure)?;
    let signature_hex = encode_hex_lower(signature.to_bytes().as_ref());
    Ok(format!(
        "sig:{SERVICE_AUTH_SIGNATURE_ALGORITHM}:{SERVICE_AUTH_SIGNATURE_PROFILE_ID}:{}:{signature_hex}",
        recovery_id.to_byte()
    ))
}

/// Resolves compressed secp256k1 public key hex for a private key (hex).
pub fn service_auth_public_key_hex_from_private_key_hex(
    private_key_hex: &str,
) -> Result<String, ServiceAuthSignatureError> {
    if private_key_hex.trim().is_empty() {
        return Err(ServiceAuthSignatureError::EmptyField("private_key_hex"));
    }
    let mut private_key_bytes = decode_hex_bytes(private_key_hex)
        .map_err(|_| ServiceAuthSignatureError::InvalidPrivateKeyHex)?;
    let signing_key = match SigningKey::from_slice(private_key_bytes.as_slice()) {
        Ok(key) => key,
        Err(_) => {
            wipe_bytes(private_key_bytes.as_mut_slice());
            return Err(ServiceAuthSignatureError::InvalidPrivateKeyHex);
        }
    };
    wipe_bytes(private_key_bytes.as_mut_slice());
    Ok(encode_hex_lower(
        signing_key
            .verifying_key()
            .to_encoded_point(true)
            .as_bytes(),
    ))
}

/// Verifies a service-auth signature with expected compressed secp256k1 public key hex.
pub fn service_auth_verify_with_public_key_hex(
    signature: &str,
    sender: &str,
    nonce: u64,
    state_hash: &str,
    payload: &str,
    expected_public_key_hex: &str,
) -> Result<(), ServiceAuthSignatureError> {
    if signature.trim().is_empty() {
        return Err(ServiceAuthSignatureError::EmptyField("signature"));
    }
    if expected_public_key_hex.trim().is_empty() {
        return Err(ServiceAuthSignatureError::EmptyField(
            "expected_public_key_hex",
        ));
    }
    let message = service_auth_signing_payload_for_fields(sender, nonce, state_hash, payload)?;

    let mut segments = signature.splitn(5, ':');
    let prefix = segments.next().unwrap_or_default();
    let algorithm = segments.next().unwrap_or_default();
    let profile_id = segments.next().unwrap_or_default();
    let recovery_id_raw = segments.next().unwrap_or_default();
    let signature_hex = segments.next().unwrap_or_default();

    if prefix != "sig" {
        return Err(ServiceAuthSignatureError::InvalidSignatureFormat);
    }
    if algorithm != SERVICE_AUTH_SIGNATURE_ALGORITHM {
        return Err(ServiceAuthSignatureError::UnsupportedAlgorithm(
            algorithm.to_owned(),
        ));
    }
    if profile_id != SERVICE_AUTH_SIGNATURE_PROFILE_ID {
        return Err(ServiceAuthSignatureError::UnsupportedProfile(
            profile_id.to_owned(),
        ));
    }
    if recovery_id_raw.trim().is_empty() || signature_hex.trim().is_empty() {
        return Err(ServiceAuthSignatureError::InvalidSignatureFormat);
    }

    let recovery_id = recovery_id_raw
        .parse::<u8>()
        .ok()
        .and_then(RecoveryId::from_byte)
        .ok_or(ServiceAuthSignatureError::InvalidRecoveryId)?;
    let signature_bytes = decode_hex_bytes(signature_hex)
        .map_err(|_| ServiceAuthSignatureError::InvalidSignatureHex)?;
    if signature_bytes.len() != 64 {
        return Err(ServiceAuthSignatureError::InvalidSignatureHex);
    }
    let signature = Signature::from_slice(signature_bytes.as_slice())
        .map_err(|_| ServiceAuthSignatureError::InvalidSignatureHex)?;
    let recovered = VerifyingKey::recover_from_msg(message.as_bytes(), &signature, recovery_id)
        .map_err(|_| ServiceAuthSignatureError::VerificationFailure)?;

    let expected_key_bytes = decode_hex_bytes(expected_public_key_hex)
        .map_err(|_| ServiceAuthSignatureError::InvalidPublicKeyHex)?;
    if expected_key_bytes.len() != 33 {
        return Err(ServiceAuthSignatureError::InvalidPublicKeyHex);
    }
    let expected_key = VerifyingKey::from_sec1_bytes(expected_key_bytes.as_slice())
        .map_err(|_| ServiceAuthSignatureError::InvalidPublicKeyHex)?;
    if expected_key != recovered {
        return Err(ServiceAuthSignatureError::VerificationFailure);
    }
    Ok(())
}

/// Builds a deterministic baseline signature fixture for the provided fields.
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

/// Builds a deterministic legacy signature fixture for compatibility tests.
pub fn legacy_signature_for_fields(
    sender: &str,
    nonce: u64,
    state_hash: &str,
    payload: &str,
) -> String {
    format!("sig:{}:{}:{}:{}", sender, nonce, state_hash, payload.len())
}

/// Builds a fixture with an unsupported profile identifier.
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

/// Builds a fixture with an unsupported signature algorithm identifier.
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

/// Compatibility fixture entry for signature profile verification behavior.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureProfileCompatibilityFixture {
    /// Fixture identifier used in compatibility tables.
    pub fixture_id: &'static str,
    /// Signature payload used for verification checks.
    pub signature: String,
    /// Whether the fixture should pass supported-profile verification.
    pub should_verify: bool,
}

/// Returns compatibility fixtures for baseline, legacy, and unsupported variants.
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
            fixture_id: "unknown-algorithm+baseline-v1",
            signature: unknown_signature_algorithm_for_fields(sender, nonce, state_hash, payload),
            should_verify: false,
        },
    ]
}

/// Returns whether a signature matches the supported baseline profile.
///
/// This requires:
/// - canonical algorithm id (`deterministic-v1`),
/// - canonical profile id (`baseline-v1`),
/// - deterministic field rendering match.
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

    crate::constant_time_eq::constant_time_eq_str(
        signature,
        baseline_signature_for_fields(sender, nonce, state_hash, payload).as_str(),
    )
}

#[cfg(test)]
mod tests {
    use super::{
        BASELINE_SIGNATURE_PROFILE_ID, LEGACY_SIGNATURE_PROFILE_ID,
        SERVICE_AUTH_SIGNATURE_ALGORITHM, SERVICE_AUTH_SIGNATURE_PROFILE_ID,
        UNKNOWN_SIGNATURE_ALGORITHM_ID, baseline_signature_algorithm,
        baseline_signature_for_fields, baseline_signature_profile_id, legacy_signature_for_fields,
        parse_signature_profile_metadata, service_auth_public_key_hex_from_private_key_hex,
        service_auth_sign_with_private_key_hex, service_auth_signing_payload_for_fields,
        service_auth_verify_with_public_key_hex, signature_matches_supported_profile_for_fields,
        signature_profile_compatibility_fixtures_for_fields,
    };

    const SOURCE: &str = include_str!("signature_profile.rs");
    const TEST_SERVICE_AUTH_PRIVATE_KEY_HEX: &str =
        "658c3528422eb527b4c108b8f6d1e5f629543c304ea49cf608c67794424291c4";

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
            Some(super::SignatureProfileMetadata {
                algorithm: "deterministic-v1".to_owned(),
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

    #[test]
    fn service_auth_signing_payload_contains_canonical_field_bindings() {
        let payload = service_auth_signing_payload_for_fields("agent-a", 7, "state:1", "hello")
            .expect("payload should render");
        assert!(payload.contains("sender_len=7"));
        assert!(payload.contains("nonce=7"));
        assert!(payload.contains("state_hash_len=7"));
        assert!(payload.contains("payload_len=5"));
    }

    #[test]
    fn service_auth_signature_roundtrip_verifies_with_expected_public_key() {
        let signature = service_auth_sign_with_private_key_hex(
            "agent-a",
            7,
            "service-api:chain:1",
            "{\"message\":\"hello\"}",
            TEST_SERVICE_AUTH_PRIVATE_KEY_HEX,
        )
        .expect("signature should render");
        assert!(signature.starts_with(&format!(
            "sig:{SERVICE_AUTH_SIGNATURE_ALGORITHM}:{SERVICE_AUTH_SIGNATURE_PROFILE_ID}:"
        )));
        let public_key_hex =
            service_auth_public_key_hex_from_private_key_hex(TEST_SERVICE_AUTH_PRIVATE_KEY_HEX)
                .expect("public key should derive");
        service_auth_verify_with_public_key_hex(
            signature.as_str(),
            "agent-a",
            7,
            "service-api:chain:1",
            "{\"message\":\"hello\"}",
            public_key_hex.as_str(),
        )
        .expect("signature should verify");
    }

    #[test]
    fn service_auth_signature_verification_rejects_tampered_payload_and_legacy_format() {
        let signature = service_auth_sign_with_private_key_hex(
            "agent-a",
            7,
            "service-api:chain:1",
            "{\"message\":\"hello\"}",
            TEST_SERVICE_AUTH_PRIVATE_KEY_HEX,
        )
        .expect("signature should render");
        let public_key_hex =
            service_auth_public_key_hex_from_private_key_hex(TEST_SERVICE_AUTH_PRIVATE_KEY_HEX)
                .expect("public key should derive");
        let tampered = service_auth_verify_with_public_key_hex(
            signature.as_str(),
            "agent-a",
            7,
            "service-api:chain:1",
            "{\"message\":\"tampered\"}",
            public_key_hex.as_str(),
        );
        assert!(tampered.is_err());

        let legacy = baseline_signature_for_fields("agent-a", 7, "service-api:chain:1", "hello");
        let legacy_result = service_auth_verify_with_public_key_hex(
            legacy.as_str(),
            "agent-a",
            7,
            "service-api:chain:1",
            "hello",
            public_key_hex.as_str(),
        );
        assert!(legacy_result.is_err());
    }

    #[test]
    fn regression_wipe_bytes_zeroizes_secret_material_buffer() {
        // Regression: #5924
        let mut secret = [0x41_u8, 0x42, 0x43, 0x44];
        super::wipe_bytes(&mut secret);
        assert_eq!(secret, [0_u8; 4]);
    }

    #[test]
    fn regression_invalid_private_key_signing_error_does_not_echo_secret_material() {
        // Regression: #5924
        let private_key_hex = "deadbeefdeadbeefdeadbeefdeadbeef";
        let error = service_auth_sign_with_private_key_hex(
            "agent-a",
            1,
            "service-api:chain:1",
            "{\"message\":\"hello\"}",
            private_key_hex,
        )
        .expect_err("invalid private key input should fail");
        let rendered = error.to_string();
        assert!(!rendered.contains(private_key_hex));
    }
}
