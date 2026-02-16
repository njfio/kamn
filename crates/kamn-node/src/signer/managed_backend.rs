use std::env;
use std::io::Read;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use k256::ecdsa::{RecoveryId, Signature, VerifyingKey};
use kamn_core::{
    ConfigError, KolmeRuntimeCommitRequest, SecureSignerBackend, SecureSignerProvider,
    SignerBackend, SignerBackendError, SignerProviderHandshakeMatrix, SigningRequest,
};

use super::{decode_kolme_hex_bytes, encode_kolme_hex_lower, KolmeLiveSignerSelection};
use crate::{
    KOLME_LIVE_MANAGED_SIGNER_COMMAND_ENV, KOLME_LIVE_MANAGED_SIGNER_POLL_INTERVAL_MILLIS,
    KOLME_LIVE_MANAGED_SIGNER_REQUIRED_ENV, KOLME_LIVE_MANAGED_SIGNER_TIMEOUT_SECONDS_DEFAULT,
    KOLME_LIVE_MANAGED_SIGNER_TIMEOUT_SECONDS_ENV, KOLME_LIVE_SIGNER_KEY_SOURCE_MANAGED_EXTERNAL,
    KOLME_LIVE_SIGNER_PROFILE_PRIMARY, KOLME_LIVE_SIGNER_PROFILE_SECONDARY,
    KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX_ENV, KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX_SECONDARY_ENV,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct ManagedExternalBackendSignature {
    signature_hex: String,
    recovery_id: u8,
    signer_public_key_hex: String,
}

fn map_kolme_live_secure_signer_backend_error(error: SignerBackendError) -> ConfigError {
    let (reason_code, message) = match &error {
        SignerBackendError::ProviderUnavailable { .. } => {
            ("managed_signer_provider_unavailable", error.to_string())
        }
        SignerBackendError::ProviderHandshakeRejected { .. } => (
            "managed_signer_provider_handshake_rejected",
            error.to_string(),
        ),
        SignerBackendError::FallbackDeniedByRolePolicy { .. } => (
            "managed_signer_fallback_denied_by_role_policy",
            error.to_string(),
        ),
        _ => ("managed_signer_backend_error", error.to_string()),
    };
    ConfigError::RuntimeKolmeLive(format!(
        "managed-external secure signer backend error: {message} ({reason_code})"
    ))
}

fn resolve_optional_kolme_live_managed_signer_command() -> Result<Option<String>, ConfigError> {
    match env::var(KOLME_LIVE_MANAGED_SIGNER_COMMAND_ENV) {
        Ok(command) => {
            let trimmed = command.trim();
            if trimmed.is_empty() {
                return Err(ConfigError::RuntimeKolmeLive(format!(
                    "{KOLME_LIVE_MANAGED_SIGNER_COMMAND_ENV} must not be empty when present (managed_signer_backend_unavailable)"
                )));
            }
            Ok(Some(trimmed.to_owned()))
        }
        Err(env::VarError::NotPresent) => Ok(None),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::RuntimeKolmeLive(format!(
            "{KOLME_LIVE_MANAGED_SIGNER_COMMAND_ENV} must be valid utf-8 when present (managed_signer_backend_unavailable)"
        ))),
    }
}

pub(super) fn resolve_required_kolme_live_managed_signer_command() -> Result<String, ConfigError> {
    resolve_optional_kolme_live_managed_signer_command()?.ok_or_else(|| {
        ConfigError::RuntimeKolmeLive(format!(
            "{KOLME_LIVE_MANAGED_SIGNER_COMMAND_ENV} must be set when managed-external signing is selected (managed_signer_backend_required_missing)"
        ))
    })
}

pub(crate) fn resolve_kolme_live_managed_signer_required_marker() -> Result<bool, ConfigError> {
    match env::var(KOLME_LIVE_MANAGED_SIGNER_REQUIRED_ENV) {
        Ok(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                return Err(ConfigError::RuntimeKolmeLive(format!(
                    "{KOLME_LIVE_MANAGED_SIGNER_REQUIRED_ENV} must not be empty when present (managed_signer_backend_required_invalid)"
                )));
            }
            match trimmed {
                "true" => Ok(true),
                "false" => Ok(false),
                _ => Err(ConfigError::RuntimeKolmeLive(format!(
                    "{KOLME_LIVE_MANAGED_SIGNER_REQUIRED_ENV} must be 'true' or 'false', found '{trimmed}' (managed_signer_backend_required_invalid)"
                ))),
            }
        }
        Err(env::VarError::NotPresent) => Ok(false),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::RuntimeKolmeLive(format!(
            "{KOLME_LIVE_MANAGED_SIGNER_REQUIRED_ENV} must be valid utf-8 when present (managed_signer_backend_required_invalid)"
        ))),
    }
}

fn resolve_kolme_live_managed_signer_timeout_seconds() -> Result<u64, ConfigError> {
    match env::var(KOLME_LIVE_MANAGED_SIGNER_TIMEOUT_SECONDS_ENV) {
        Ok(raw_timeout) => {
            let trimmed = raw_timeout.trim();
            if trimmed.is_empty() {
                return Err(ConfigError::RuntimeKolmeLive(format!(
                    "{KOLME_LIVE_MANAGED_SIGNER_TIMEOUT_SECONDS_ENV} must not be empty when present (managed_signer_backend_timeout_invalid)"
                )));
            }
            let timeout = trimmed.parse::<u64>().map_err(|_| {
                ConfigError::RuntimeKolmeLive(format!(
                    "{KOLME_LIVE_MANAGED_SIGNER_TIMEOUT_SECONDS_ENV} must be a positive integer, found '{trimmed}' (managed_signer_backend_timeout_invalid)"
                ))
            })?;
            if timeout == 0 {
                return Err(ConfigError::RuntimeKolmeLive(format!(
                    "{KOLME_LIVE_MANAGED_SIGNER_TIMEOUT_SECONDS_ENV} must be greater than zero (managed_signer_backend_timeout_invalid)"
                )));
            }
            Ok(timeout)
        }
        Err(env::VarError::NotPresent) => Ok(KOLME_LIVE_MANAGED_SIGNER_TIMEOUT_SECONDS_DEFAULT),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::RuntimeKolmeLive(format!(
            "{KOLME_LIVE_MANAGED_SIGNER_TIMEOUT_SECONDS_ENV} must be valid utf-8 when present (managed_signer_backend_timeout_invalid)"
        ))),
    }
}

fn parse_kolme_live_managed_signer_command_output(
    stdout: &str,
) -> Result<ManagedExternalBackendSignature, ConfigError> {
    let mut signature_hex = None;
    let mut recovery_id = None;
    let mut signer_public_key_hex = None;
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let (key, value) = trimmed.split_once('=').ok_or_else(|| {
            ConfigError::RuntimeKolmeLive(format!(
                "managed-external signer backend response line must be key=value, found '{trimmed}' (managed_signer_backend_response_malformed)"
            ))
        })?;
        match key.trim() {
            "signature_hex" => {
                let value = value.trim();
                if value.is_empty() {
                    return Err(ConfigError::RuntimeKolmeLive(
                        "managed-external signer backend response missing signature_hex value (managed_signer_backend_response_malformed)"
                            .to_owned(),
                    ));
                }
                signature_hex = Some(value.to_owned());
            }
            "recovery_id" => {
                let value = value.trim();
                if value.is_empty() {
                    return Err(ConfigError::RuntimeKolmeLive(
                        "managed-external signer backend response missing recovery_id value (managed_signer_backend_response_malformed)"
                            .to_owned(),
                    ));
                }
                recovery_id = Some(value.parse::<u8>().map_err(|_| {
                    ConfigError::RuntimeKolmeLive(format!(
                        "managed-external signer backend response recovery_id must be an integer, found '{value}' (managed_signer_backend_response_malformed)"
                    ))
                })?);
            }
            "signer_public_key_hex" => {
                let value = value.trim();
                if value.is_empty() {
                    return Err(ConfigError::RuntimeKolmeLive(
                        "managed-external signer backend response missing signer_public_key_hex value (managed_signer_backend_response_provenance_missing)"
                            .to_owned(),
                    ));
                }
                signer_public_key_hex = Some(value.to_owned());
            }
            _ => {}
        }
    }
    let signature_hex = signature_hex.ok_or_else(|| {
        ConfigError::RuntimeKolmeLive(
            "managed-external signer backend response missing signature_hex key (managed_signer_backend_response_malformed)"
                .to_owned(),
        )
    })?;
    let recovery_id = recovery_id.ok_or_else(|| {
        ConfigError::RuntimeKolmeLive(
            "managed-external signer backend response missing recovery_id key (managed_signer_backend_response_malformed)"
                .to_owned(),
        )
    })?;
    let signature_bytes = decode_kolme_hex_bytes(
        signature_hex.as_str(),
        "managed_external_signer_backend_signature_hex",
    )
    .map_err(|error| {
        ConfigError::RuntimeKolmeLive(format!(
            "managed-external signer backend response signature hex is invalid: {error} (managed_signer_backend_response_malformed)"
        ))
    })?;
    if signature_bytes.len() != 64 {
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "managed-external signer backend response signature hex must decode to 64 bytes, found {} (managed_signer_backend_response_malformed)",
            signature_bytes.len()
        )));
    }
    if RecoveryId::from_byte(recovery_id).is_none() {
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "managed-external signer backend response recovery_id must be within secp256k1 range [0,3], found {recovery_id} (managed_signer_backend_response_malformed)"
        )));
    }
    let signer_public_key_hex = signer_public_key_hex.ok_or_else(|| {
        ConfigError::RuntimeKolmeLive(
            "managed-external signer backend response missing signer_public_key_hex key (managed_signer_backend_response_provenance_missing)"
                .to_owned(),
        )
    })?;
    let signer_public_key_bytes = decode_kolme_hex_bytes(
        signer_public_key_hex.as_str(),
        "managed_external_signer_backend_signer_public_key_hex",
    )
    .map_err(|error| {
        ConfigError::RuntimeKolmeLive(format!(
            "managed-external signer backend response signer_public_key_hex is invalid: {error} (managed_signer_backend_response_provenance_malformed)"
        ))
    })?;
    if signer_public_key_bytes.len() != 33 {
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "managed-external signer backend response signer_public_key_hex must decode to 33 bytes compressed secp256k1 key material, found {} (managed_signer_backend_response_provenance_malformed)",
            signer_public_key_bytes.len()
        )));
    }
    let signer_verifying_key =
        VerifyingKey::from_sec1_bytes(signer_public_key_bytes.as_slice()).map_err(|error| {
            ConfigError::RuntimeKolmeLive(format!(
                "managed-external signer backend response signer_public_key_hex is not valid secp256k1 key material: {error} (managed_signer_backend_response_provenance_malformed)"
            ))
        })?;
    Ok(ManagedExternalBackendSignature {
        signature_hex,
        recovery_id,
        signer_public_key_hex: encode_kolme_hex_lower(
            signer_verifying_key.to_encoded_point(true).as_bytes(),
        ),
    })
}

fn verify_kolme_live_managed_signer_backend_signature_provenance(
    canonical_message: &str,
    expected_signer_public_key_hex: &str,
    backend_signature: &ManagedExternalBackendSignature,
) -> Result<(), ConfigError> {
    let expected_signer_public_key_hex = expected_signer_public_key_hex.trim();
    if expected_signer_public_key_hex.is_empty() {
        return Err(ConfigError::RuntimeKolmeLive(
            "expected managed-external signer public key must not be empty (managed_signer_backend_response_provenance_mismatch)"
                .to_owned(),
        ));
    }
    if !backend_signature
        .signer_public_key_hex
        .eq_ignore_ascii_case(expected_signer_public_key_hex)
    {
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "managed-external signer backend response signer_public_key_hex does not match expected runtime signer key material (expected={}, found={}) (managed_signer_backend_response_provenance_mismatch)",
            expected_signer_public_key_hex,
            backend_signature.signer_public_key_hex,
        )));
    }

    let signature_bytes = decode_kolme_hex_bytes(
        backend_signature.signature_hex.as_str(),
        "managed_external_signer_backend_signature_hex",
    )
    .map_err(|error| {
        ConfigError::RuntimeKolmeLive(format!(
            "managed-external signer backend response signature hex is invalid: {error} (managed_signer_backend_response_malformed)"
        ))
    })?;
    let signature = Signature::from_slice(signature_bytes.as_slice()).map_err(|error| {
        ConfigError::RuntimeKolmeLive(format!(
            "managed-external signer backend response signature bytes are invalid secp256k1 material: {error} (managed_signer_backend_response_malformed)"
        ))
    })?;
    let recovery = RecoveryId::from_byte(backend_signature.recovery_id).ok_or_else(|| {
        ConfigError::RuntimeKolmeLive(format!(
            "managed-external signer backend response recovery_id must be within secp256k1 range [0,3], found {} (managed_signer_backend_response_malformed)",
            backend_signature.recovery_id
        ))
    })?;
    let recovered =
        VerifyingKey::recover_from_msg(canonical_message.as_bytes(), &signature, recovery).map_err(
            |error| {
                ConfigError::RuntimeKolmeLive(format!(
                    "failed to recover secp256k1 public key from managed-external signer backend response: {error} (managed_signer_backend_response_malformed)"
                ))
            },
        )?;
    let expected = VerifyingKey::from_sec1_bytes(
        decode_kolme_hex_bytes(
            backend_signature.signer_public_key_hex.as_str(),
            "managed_external_signer_backend_signer_public_key_hex",
        )
        .map_err(|error| {
            ConfigError::RuntimeKolmeLive(format!(
                "managed-external signer backend response signer_public_key_hex is invalid: {error} (managed_signer_backend_response_provenance_malformed)"
            ))
        })?
        .as_slice(),
    )
    .map_err(|error| {
        ConfigError::RuntimeKolmeLive(format!(
            "managed-external signer backend response signer_public_key_hex is not valid secp256k1 key material: {error} (managed_signer_backend_response_provenance_malformed)"
        ))
    })?;
    if recovered != expected {
        return Err(ConfigError::RuntimeKolmeLive(
            "managed-external signer backend response signature does not match signer_public_key_hex (managed_signer_backend_response_provenance_mismatch)"
                .to_owned(),
        ));
    }
    Ok(())
}

fn execute_kolme_live_managed_signer_backend_command(
    command: &str,
    timeout_seconds: u64,
    key_reference: &str,
    signing_request: &SigningRequest,
    canonical_message: &str,
) -> Result<ManagedExternalBackendSignature, ConfigError> {
    let nonce = signing_request.nonce.to_string();
    let mut child = Command::new("sh")
        .arg("-c")
        .arg(command)
        .env("KAMN_MANAGED_SIGNER_KEY_REFERENCE", key_reference)
        .env("KAMN_MANAGED_SIGNER_ACTOR_DID", signing_request.sender.as_str())
        .env("KAMN_MANAGED_SIGNER_NONCE", nonce.as_str())
        .env("KAMN_MANAGED_SIGNER_STATE_ROOT", signing_request.state_hash.as_str())
        .env("KAMN_MANAGED_SIGNER_CANONICAL_MESSAGE", canonical_message)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            ConfigError::RuntimeKolmeLive(format!(
                "managed-external signer backend unavailable: failed to spawn command from {KOLME_LIVE_MANAGED_SIGNER_COMMAND_ENV}: {error} (managed_signer_backend_unavailable)"
            ))
        })?;
    let mut stdout = child.stdout.take().ok_or_else(|| {
        ConfigError::RuntimeKolmeLive(
            "managed-external signer backend unavailable: stdout pipe was not configured (managed_signer_backend_unavailable)"
                .to_owned(),
        )
    })?;
    let mut stderr = child.stderr.take().ok_or_else(|| {
        ConfigError::RuntimeKolmeLive(
            "managed-external signer backend unavailable: stderr pipe was not configured (managed_signer_backend_unavailable)"
                .to_owned(),
        )
    })?;
    let deadline = Instant::now() + Duration::from_secs(timeout_seconds);
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(ConfigError::RuntimeKolmeLive(format!(
                        "managed-external signer backend timed out after {timeout_seconds}s (managed_signer_backend_timeout)"
                    )));
                }
                thread::sleep(Duration::from_millis(
                    KOLME_LIVE_MANAGED_SIGNER_POLL_INTERVAL_MILLIS,
                ));
            }
            Err(error) => {
                return Err(ConfigError::RuntimeKolmeLive(format!(
                    "managed-external signer backend unavailable while waiting for completion: {error} (managed_signer_backend_unavailable)"
                )))
            }
        }
    };
    let mut stdout_text = String::new();
    stdout.read_to_string(&mut stdout_text).map_err(|error| {
        ConfigError::RuntimeKolmeLive(format!(
            "managed-external signer backend unavailable: failed to read stdout: {error} (managed_signer_backend_unavailable)"
        ))
    })?;
    let mut stderr_text = String::new();
    stderr.read_to_string(&mut stderr_text).map_err(|error| {
        ConfigError::RuntimeKolmeLive(format!(
            "managed-external signer backend unavailable: failed to read stderr: {error} (managed_signer_backend_unavailable)"
        ))
    })?;
    if !status.success() {
        let stderr_trimmed = stderr_text.trim();
        let stderr_summary = if stderr_trimmed.is_empty() {
            "no stderr output"
        } else {
            stderr_trimmed
        };
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "managed-external signer backend unavailable: command exited with status {status} ({stderr_summary}) (managed_signer_backend_unavailable)"
        )));
    }
    parse_kolme_live_managed_signer_command_output(stdout_text.as_str())
}

fn managed_signer_public_key_env_for_profile(profile: &str) -> Result<&'static str, ConfigError> {
    match profile {
        KOLME_LIVE_SIGNER_PROFILE_PRIMARY => Ok(KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX_ENV),
        KOLME_LIVE_SIGNER_PROFILE_SECONDARY => Ok(KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX_SECONDARY_ENV),
        _ => Err(ConfigError::RuntimeKolmeLive(format!(
            "unsupported managed-external signer profile for public key marker resolution: {profile} (managed_signer_public_key_marker_invalid)"
        ))),
    }
}

fn normalize_managed_signer_public_key_hex(
    value: &str,
    env_name: &str,
) -> Result<String, ConfigError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "{env_name} must not be empty (managed_signer_public_key_marker_invalid)"
        )));
    }
    let key_bytes = decode_kolme_hex_bytes(trimmed, env_name).map_err(|error| {
        ConfigError::RuntimeKolmeLive(format!(
            "{env_name} is invalid: {error} (managed_signer_public_key_marker_invalid)"
        ))
    })?;
    if key_bytes.len() != 33 {
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "{env_name} must decode to 33 bytes compressed secp256k1 key material, found {} (managed_signer_public_key_marker_invalid)",
            key_bytes.len()
        )));
    }
    let verifying_key = VerifyingKey::from_sec1_bytes(key_bytes.as_slice()).map_err(|error| {
        ConfigError::RuntimeKolmeLive(format!(
            "{env_name} is not valid secp256k1 key material: {error} (managed_signer_public_key_marker_invalid)"
        ))
    })?;
    Ok(encode_kolme_hex_lower(
        verifying_key.to_encoded_point(true).as_bytes(),
    ))
}

pub(super) fn resolve_required_managed_signer_public_key_hex(
    signer_selection: &KolmeLiveSignerSelection,
) -> Result<String, ConfigError> {
    let env_name = managed_signer_public_key_env_for_profile(signer_selection.profile)?;
    match env::var(env_name) {
        Ok(value) => normalize_managed_signer_public_key_hex(value.as_str(), env_name),
        Err(env::VarError::NotPresent) => Err(ConfigError::RuntimeKolmeLive(format!(
            "{env_name} must be set when --kolme-live-signer-key-source={} (managed_signer_public_key_marker_missing)",
            KOLME_LIVE_SIGNER_KEY_SOURCE_MANAGED_EXTERNAL
        ))),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::RuntimeKolmeLive(format!(
            "{env_name} must be valid utf-8 when present (managed_signer_public_key_marker_invalid)"
        ))),
    }
}

pub(crate) fn sign_kolme_live_managed_external_message(
    key_reference: &str,
    request: &KolmeRuntimeCommitRequest,
    nonce: u64,
    canonical_message: &str,
    provider_handshake_matrix: SignerProviderHandshakeMatrix,
    expected_signer_public_key_hex: &str,
) -> Result<(String, u8), ConfigError> {
    let _provider = SecureSignerProvider::from_key_id(key_reference).map_err(|error| {
        ConfigError::RuntimeKolmeLive(format!(
            "managed-external signer key reference parse failed before secure routing: {error} (managed_signer_key_reference_invalid)"
        ))
    })?;
    let signing_request = SigningRequest::new(
        key_reference,
        request.actor_did.as_str(),
        nonce,
        canonical_message,
        request.state_root.as_str(),
    )
    .map_err(|error| {
        ConfigError::RuntimeKolmeLive(format!(
            "managed-external signer request validation failed: {error} (managed_signer_request_invalid)"
        ))
    })?;
    let secure_backend =
        SecureSignerBackend::with_provider_handshake_matrix(provider_handshake_matrix);
    let _backend_signature = secure_backend
        .sign(&signing_request)
        .map_err(map_kolme_live_secure_signer_backend_error)?;
    let command = resolve_required_kolme_live_managed_signer_command()?;
    let timeout_seconds = resolve_kolme_live_managed_signer_timeout_seconds()?;
    let backend_signature = execute_kolme_live_managed_signer_backend_command(
        command.as_str(),
        timeout_seconds,
        key_reference,
        &signing_request,
        canonical_message,
    )?;
    verify_kolme_live_managed_signer_backend_signature_provenance(
        canonical_message,
        expected_signer_public_key_hex,
        &backend_signature,
    )?;
    Ok((
        backend_signature.signature_hex,
        backend_signature.recovery_id,
    ))
}
