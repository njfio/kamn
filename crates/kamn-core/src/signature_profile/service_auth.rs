use k256::ecdsa::{RecoveryId, Signature, VerifyingKey};
use k256::elliptic_curve::rand_core::OsRng;
use std::sync::OnceLock;

use super::{
    decode_hex_bytes, encode_hex_lower, service_auth_signing_payload_for_fields,
    ServiceAuthSignatureError, ServiceAuthSigningKey, SERVICE_AUTH_SIGNATURE_ALGORITHM,
    SERVICE_AUTH_SIGNATURE_PROFILE_ID,
};

/// Runs the generate ephemeral service auth private key hex contract helper.
pub fn generate_ephemeral_service_auth_private_key_hex() -> String {
    let signing_key = k256::ecdsa::SigningKey::random(&mut OsRng);
    encode_hex_lower(signing_key.to_bytes().as_ref())
}

/// Runs the debug fallback signer private key hex contract helper.
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

/// Runs the service auth sign with private key hex contract helper.
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
    let signing_key = ServiceAuthSigningKey::from_private_key_hex(private_key_hex)?;
    service_auth_sign_with_signing_key(sender, nonce, state_hash, payload, &signing_key)
}

pub(crate) fn service_auth_sign_with_signing_key(
    sender: &str,
    nonce: u64,
    state_hash: &str,
    payload: &str,
    signing_key: &ServiceAuthSigningKey,
) -> Result<String, ServiceAuthSignatureError> {
    let message = service_auth_signing_payload_for_fields(sender, nonce, state_hash, payload)?;
    let (signature, recovery_id) = signing_key.sign_message(message.as_str())?;
    Ok(render_signature(&signature, recovery_id))
}

fn render_signature(signature: &Signature, recovery_id: RecoveryId) -> String {
    let signature_hex = encode_hex_lower(signature.to_bytes().as_ref());
    format!(
        "sig:{SERVICE_AUTH_SIGNATURE_ALGORITHM}:{SERVICE_AUTH_SIGNATURE_PROFILE_ID}:{}:{signature_hex}",
        recovery_id.to_byte()
    )
}

/// Runs the service auth public key hex from private key hex contract helper.
pub fn service_auth_public_key_hex_from_private_key_hex(
    private_key_hex: &str,
) -> Result<String, ServiceAuthSignatureError> {
    if private_key_hex.trim().is_empty() {
        return Err(ServiceAuthSignatureError::EmptyField("private_key_hex"));
    }
    let signing_key = ServiceAuthSigningKey::from_private_key_hex(private_key_hex)?;
    Ok(signing_key.public_key_hex())
}

/// Runs the service auth verify with public key hex contract helper.
pub fn service_auth_verify_with_public_key_hex(
    signature: &str,
    sender: &str,
    nonce: u64,
    state_hash: &str,
    payload: &str,
    expected_public_key_hex: &str,
) -> Result<(), ServiceAuthSignatureError> {
    validate_verify_inputs(signature, expected_public_key_hex)?;
    let message = service_auth_signing_payload_for_fields(sender, nonce, state_hash, payload)?;
    let (recovery_id, signature) = parse_signature(signature)?;
    let recovered = VerifyingKey::recover_from_msg(message.as_bytes(), &signature, recovery_id)
        .map_err(|_| ServiceAuthSignatureError::VerificationFailure)?;
    verify_recovered_key(recovered, expected_public_key_hex)
}

fn validate_verify_inputs(
    signature: &str,
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
    Ok(())
}

fn parse_signature(signature: &str) -> Result<(RecoveryId, Signature), ServiceAuthSignatureError> {
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
    Ok((recovery_id, signature))
}

fn verify_recovered_key(
    recovered: VerifyingKey,
    expected_public_key_hex: &str,
) -> Result<(), ServiceAuthSignatureError> {
    let expected_key_bytes = decode_hex_bytes(expected_public_key_hex)
        .map_err(|_| ServiceAuthSignatureError::InvalidPublicKeyHex)?;
    if expected_key_bytes.len() != 33 {
        return Err(ServiceAuthSignatureError::InvalidPublicKeyHex);
    }
    let expected_key = VerifyingKey::from_sec1_bytes(expected_key_bytes.as_slice())
        .map_err(|_| ServiceAuthSignatureError::InvalidPublicKeyHex)?;
    let expected_key_point = expected_key.to_encoded_point(true);
    let recovered_key_point = recovered.to_encoded_point(true);
    if !crate::constant_time_eq::constant_time_eq_bytes(
        expected_key_point.as_bytes(),
        recovered_key_point.as_bytes(),
    ) {
        return Err(ServiceAuthSignatureError::VerificationFailure);
    }
    Ok(())
}
