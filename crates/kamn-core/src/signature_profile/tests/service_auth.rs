use super::super::{
    baseline_signature_for_fields, service_auth_public_key_hex_from_private_key_hex,
    service_auth_sign_with_private_key_hex, service_auth_signing_payload_for_fields,
    service_auth_verify_with_public_key_hex, ServiceAuthSignatureError,
    SERVICE_AUTH_SIGNATURE_ALGORITHM, SERVICE_AUTH_SIGNATURE_PROFILE_ID,
};

const SOURCE: &str = include_str!("../../signature_profile.rs");
const TEST_SERVICE_AUTH_PRIVATE_KEY_HEX: &str =
    "658c3528422eb527b4c108b8f6d1e5f629543c304ea49cf608c67794424291c4";

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
    let public_key_hex = service_auth_public_key_hex_from_private_key_hex(TEST_SERVICE_AUTH_PRIVATE_KEY_HEX)
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
    let public_key_hex = service_auth_public_key_hex_from_private_key_hex(TEST_SERVICE_AUTH_PRIVATE_KEY_HEX)
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
fn service_auth_signature_verification_rejects_wrong_public_key_with_verification_failure() {
    let signature = service_auth_sign_with_private_key_hex(
        "agent-a",
        7,
        "service-api:chain:1",
        "{\"message\":\"hello\"}",
        TEST_SERVICE_AUTH_PRIVATE_KEY_HEX,
    )
    .expect("signature should render");
    let wrong_public_key_hex = service_auth_public_key_hex_from_private_key_hex(
        "758c3528422eb527b4c108b8f6d1e5f629543c304ea49cf608c67794424291c4",
    )
    .expect("wrong public key should derive");
    assert_eq!(
        service_auth_verify_with_public_key_hex(
            signature.as_str(),
            "agent-a",
            7,
            "service-api:chain:1",
            "{\"message\":\"hello\"}",
            wrong_public_key_hex.as_str(),
        ),
        Err(ServiceAuthSignatureError::VerificationFailure)
    );
}

#[test]
fn service_auth_signature_verification_rejects_malformed_public_key_hex() {
    let signature = service_auth_sign_with_private_key_hex(
        "agent-a",
        7,
        "service-api:chain:1",
        "{\"message\":\"hello\"}",
        TEST_SERVICE_AUTH_PRIVATE_KEY_HEX,
    )
    .expect("signature should render");
    assert_eq!(
        service_auth_verify_with_public_key_hex(
            signature.as_str(),
            "agent-a",
            7,
            "service-api:chain:1",
            "{\"message\":\"hello\"}",
            "abcd",
        ),
        Err(ServiceAuthSignatureError::InvalidPublicKeyHex)
    );
}

#[test]
fn regression_requires_constant_time_service_auth_recovered_key_compare() {
    assert!(
        SOURCE.contains("crate::constant_time_eq::constant_time_eq_bytes("),
        "service-auth verification should use the internal constant-time helper for recovered-key comparison"
    );
    assert!(
        !SOURCE.contains(["if expected_key !=", " recovered {"] .concat().as_str()),
        "service-auth verification must not use direct recovered-key equality"
    );
}

#[test]
fn regression_wipe_bytes_zeroizes_secret_material_buffer() {
    let mut secret = [0x41_u8, 0x42, 0x43, 0x44];
    super::super::encoding::wipe_bytes(&mut secret);
    assert_eq!(secret, [0_u8; 4]);
}

#[test]
fn regression_invalid_private_key_signing_error_does_not_echo_secret_material() {
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
