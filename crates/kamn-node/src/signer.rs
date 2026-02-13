use std::env;
use std::io::Read;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use k256::ecdsa::{RecoveryId, Signature, SigningKey, VerifyingKey};
use kamn_core::{
    ConfigError, KolmeApiBroadcastRequest, KolmeApiNextNonceRequest,
    KolmeRuntimeCommitHttpTransport, KolmeRuntimeCommitProviderError, KolmeRuntimeCommitRequest,
    SecureSignerBackend, SecureSignerProvider, SignerBackend, SignerBackendError, SignerKeyRole,
    SignerProviderHandshakeMatrix, SigningRequest,
};

use super::wire_payload::render_kolme_live_native_direct_message;
use super::{
    KOLME_LIVE_MANAGED_SIGNER_COMMAND_ENV, KOLME_LIVE_MANAGED_SIGNER_POLL_INTERVAL_MILLIS,
    KOLME_LIVE_MANAGED_SIGNER_REQUIRED_ENV, KOLME_LIVE_MANAGED_SIGNER_TIMEOUT_SECONDS_DEFAULT,
    KOLME_LIVE_MANAGED_SIGNER_TIMEOUT_SECONDS_ENV, KOLME_LIVE_NONCE_PATH,
    KOLME_LIVE_SIGNER_KEY_REF_ENV, KOLME_LIVE_SIGNER_KEY_REF_SECONDARY_ENV,
    KOLME_LIVE_SIGNER_KEY_SOURCE_ENV_LOCAL, KOLME_LIVE_SIGNER_KEY_SOURCE_MANAGED_EXTERNAL,
    KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_ENV, KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK_ENV,
    KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY_ENV, KOLME_LIVE_SIGNER_PROFILE_ENV,
    KOLME_LIVE_SIGNER_PROFILE_PRIMARY, KOLME_LIVE_SIGNER_PROFILE_SECONDARY,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct KolmeLiveSignerSelection {
    pub(crate) profile: &'static str,
    pub(crate) key_source: &'static str,
    pub(crate) private_key_env: &'static str,
    pub(crate) key_reference_env: &'static str,
}

#[derive(Debug, Clone)]
pub(crate) struct KolmeForkSecp256k1SignerAdapter {
    signing_key: SigningKey,
    private_key_env: &'static str,
}

pub(crate) fn normalize_kolme_live_signer_profile_selector(
    value: &str,
) -> Result<&'static str, ConfigError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ConfigError::RuntimeKolmeLive(
            "--kolme-live-signer-profile must not be empty".to_owned(),
        ));
    }
    match trimmed {
        KOLME_LIVE_SIGNER_PROFILE_PRIMARY => Ok(KOLME_LIVE_SIGNER_PROFILE_PRIMARY),
        KOLME_LIVE_SIGNER_PROFILE_SECONDARY => Ok(KOLME_LIVE_SIGNER_PROFILE_SECONDARY),
        _ => Err(ConfigError::RuntimeKolmeLive(format!(
            "--kolme-live-signer-profile must be one of {KOLME_LIVE_SIGNER_PROFILE_PRIMARY}, {KOLME_LIVE_SIGNER_PROFILE_SECONDARY}; found {trimmed}"
        ))),
    }
}

pub(crate) fn normalize_kolme_live_signer_key_source(
    value: &str,
) -> Result<&'static str, ConfigError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ConfigError::RuntimeKolmeLive(
            "--kolme-live-signer-key-source must not be empty".to_owned(),
        ));
    }
    match trimmed {
        KOLME_LIVE_SIGNER_KEY_SOURCE_ENV_LOCAL => Ok(KOLME_LIVE_SIGNER_KEY_SOURCE_ENV_LOCAL),
        KOLME_LIVE_SIGNER_KEY_SOURCE_MANAGED_EXTERNAL => {
            Ok(KOLME_LIVE_SIGNER_KEY_SOURCE_MANAGED_EXTERNAL)
        }
        _ => Err(ConfigError::RuntimeKolmeLive(format!(
            "--kolme-live-signer-key-source must be one of {KOLME_LIVE_SIGNER_KEY_SOURCE_ENV_LOCAL}, {KOLME_LIVE_SIGNER_KEY_SOURCE_MANAGED_EXTERNAL}; found {trimmed}"
        ))),
    }
}

fn decode_kolme_hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(10 + (byte - b'a')),
        b'A'..=b'F' => Some(10 + (byte - b'A')),
        _ => None,
    }
}

fn decode_kolme_hex_bytes(value: &str, env_name: &str) -> Result<Vec<u8>, ConfigError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "{env_name} must not be empty"
        )));
    }
    if !trimmed.len().is_multiple_of(2) {
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "{env_name} must have an even number of hex characters"
        )));
    }
    let mut decoded = Vec::with_capacity(trimmed.len() / 2);
    for pair in trimmed.as_bytes().chunks_exact(2) {
        let high = decode_kolme_hex_nibble(pair[0]).ok_or_else(|| {
            ConfigError::RuntimeKolmeLive(format!(
                "{env_name} contains invalid hex character '{}'",
                pair[0] as char
            ))
        })?;
        let low = decode_kolme_hex_nibble(pair[1]).ok_or_else(|| {
            ConfigError::RuntimeKolmeLive(format!(
                "{env_name} contains invalid hex character '{}'",
                pair[1] as char
            ))
        })?;
        decoded.push((high << 4) | low);
    }
    Ok(decoded)
}

pub(crate) fn encode_kolme_hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

fn derive_kolme_live_managed_signing_key_material(key_reference: &str) -> [u8; 32] {
    let mut material = [0u8; 32];
    for (index, byte) in key_reference.as_bytes().iter().copied().enumerate() {
        let slot = index % material.len();
        let salt = ((index as u8).wrapping_mul(17)).wrapping_add(31);
        material[slot] = material[slot].wrapping_add(byte).wrapping_add(salt);
    }
    if material.iter().all(|value| *value == 0) {
        material[0] = 1;
    }
    material
}

pub(crate) fn build_kolme_live_managed_signing_key(
    key_reference: &str,
) -> Result<SigningKey, ConfigError> {
    let key_material = derive_kolme_live_managed_signing_key_material(key_reference);
    SigningKey::from_slice(&key_material).map_err(|error| {
        ConfigError::RuntimeKolmeLive(format!(
            "failed to derive managed-external signer key material for {key_reference}: {error} (managed_signer_key_material_invalid)"
        ))
    })
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct ManagedExternalBackendSignature {
    signature_hex: String,
    recovery_id: u8,
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
                        "managed-external signer backend response missing signature_hex value (managed_signer_backend_response_malformed)".to_owned(),
                    ));
                }
                signature_hex = Some(value.to_owned());
            }
            "recovery_id" => {
                let value = value.trim();
                if value.is_empty() {
                    return Err(ConfigError::RuntimeKolmeLive(
                        "managed-external signer backend response missing recovery_id value (managed_signer_backend_response_malformed)".to_owned(),
                    ));
                }
                recovery_id = Some(value.parse::<u8>().map_err(|_| {
                    ConfigError::RuntimeKolmeLive(format!(
                        "managed-external signer backend response recovery_id must be an integer, found '{value}' (managed_signer_backend_response_malformed)"
                    ))
                })?);
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
    Ok(ManagedExternalBackendSignature {
        signature_hex,
        recovery_id,
    })
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

pub(crate) fn sign_kolme_live_managed_external_message(
    key_reference: &str,
    request: &KolmeRuntimeCommitRequest,
    nonce: u64,
    canonical_message: &str,
    provider_handshake_matrix: SignerProviderHandshakeMatrix,
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
    let signing_key = build_kolme_live_managed_signing_key(key_reference)?;
    let command = resolve_optional_kolme_live_managed_signer_command()?.ok_or_else(|| {
        ConfigError::RuntimeKolmeLive(format!(
            "{KOLME_LIVE_MANAGED_SIGNER_COMMAND_ENV} must be set when managed-external signing is selected (managed_signer_backend_required_missing)"
        ))
    })?;
    let timeout_seconds = resolve_kolme_live_managed_signer_timeout_seconds()?;
    let backend_signature = execute_kolme_live_managed_signer_backend_command(
        command.as_str(),
        timeout_seconds,
        key_reference,
        &signing_request,
        canonical_message,
    )?;
    let verifier = KolmeForkSecp256k1SignerAdapter {
        signing_key,
        private_key_env: KOLME_LIVE_MANAGED_SIGNER_COMMAND_ENV,
    };
    verifier
        .verify_message(
            canonical_message,
            backend_signature.signature_hex.as_str(),
            backend_signature.recovery_id,
        )
        .map_err(|error| {
            ConfigError::RuntimeKolmeLive(format!(
                "managed-external signer backend response failed secp256k1 verification: {error} (managed_signer_backend_response_malformed)"
            ))
        })?;
    Ok((
        backend_signature.signature_hex,
        backend_signature.recovery_id,
    ))
}

fn resolve_kolme_live_signer_profile_selector_from_env() -> Result<Option<&'static str>, ConfigError>
{
    let profile_value = match env::var(KOLME_LIVE_SIGNER_PROFILE_ENV) {
        Ok(value) => value,
        Err(env::VarError::NotPresent) => return Ok(None),
        Err(env::VarError::NotUnicode(_)) => {
            return Err(ConfigError::RuntimeKolmeLive(format!(
                "{KOLME_LIVE_SIGNER_PROFILE_ENV} must be valid utf-8"
            )))
        }
    };
    let trimmed = profile_value.trim();
    if trimmed.is_empty() {
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "{KOLME_LIVE_SIGNER_PROFILE_ENV} must not be empty"
        )));
    }
    match trimmed {
        KOLME_LIVE_SIGNER_PROFILE_PRIMARY => Ok(Some(KOLME_LIVE_SIGNER_PROFILE_PRIMARY)),
        KOLME_LIVE_SIGNER_PROFILE_SECONDARY => Ok(Some(KOLME_LIVE_SIGNER_PROFILE_SECONDARY)),
        _ => Err(ConfigError::RuntimeKolmeLive(format!(
            "{KOLME_LIVE_SIGNER_PROFILE_ENV} has unsupported profile: {trimmed}"
        ))),
    }
}

#[cfg(test)]
pub(crate) fn resolve_kolme_live_signer_private_key_env_name(
    strict_signer_profile: Option<&str>,
) -> Result<(&'static str, &'static str), ConfigError> {
    let (profile, private_key_env, _) =
        resolve_kolme_live_signer_env_name_set(strict_signer_profile)?;
    Ok((profile, private_key_env))
}

fn resolve_kolme_live_signer_env_name_set(
    strict_signer_profile: Option<&str>,
) -> Result<(&'static str, &'static str, &'static str), ConfigError> {
    let profile_from_env = resolve_kolme_live_signer_profile_selector_from_env()?;
    let profile_value = if let Some(profile) = strict_signer_profile {
        let strict_profile = normalize_kolme_live_signer_profile_selector(profile)?;
        if let Some(env_profile) = profile_from_env {
            if env_profile != strict_profile {
                return Err(ConfigError::RuntimeKolmeLive(format!(
                    "strict signer profile mismatch: --kolme-live-signer-profile={strict_profile} conflicts with {KOLME_LIVE_SIGNER_PROFILE_ENV}={env_profile}"
                )));
            }
        }
        strict_profile
    } else {
        profile_from_env.unwrap_or(KOLME_LIVE_SIGNER_PROFILE_PRIMARY)
    };
    match profile_value {
        KOLME_LIVE_SIGNER_PROFILE_PRIMARY => Ok((
            KOLME_LIVE_SIGNER_PROFILE_PRIMARY,
            KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_ENV,
            KOLME_LIVE_SIGNER_KEY_REF_ENV,
        )),
        KOLME_LIVE_SIGNER_PROFILE_SECONDARY => Ok((
            KOLME_LIVE_SIGNER_PROFILE_SECONDARY,
            KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY_ENV,
            KOLME_LIVE_SIGNER_KEY_REF_SECONDARY_ENV,
        )),
        _ => Err(ConfigError::RuntimeKolmeLive(format!(
            "internal signer profile normalization invariant violated: {profile_value}"
        ))),
    }
}

fn read_required_kolme_live_key_reference_from_env(
    selection: &KolmeLiveSignerSelection,
) -> Result<String, ConfigError> {
    let key_reference = match env::var(selection.key_reference_env) {
        Ok(value) => value,
        Err(env::VarError::NotPresent) => {
            return Err(ConfigError::RuntimeKolmeLive(format!(
                "{} must be set for signer profile {} when --kolme-live-signer-key-source={} (managed_signer_key_reference_missing)",
                selection.key_reference_env,
                selection.profile,
                KOLME_LIVE_SIGNER_KEY_SOURCE_MANAGED_EXTERNAL
            )))
        }
        Err(env::VarError::NotUnicode(_)) => {
            return Err(ConfigError::RuntimeKolmeLive(format!(
                "{} must be valid utf-8 for signer profile {} (managed_signer_key_reference_invalid)",
                selection.key_reference_env, selection.profile
            )))
        }
    };
    normalize_kolme_live_managed_signer_key_reference(
        key_reference.as_str(),
        selection.profile,
        selection.key_reference_env,
    )
}

fn normalize_kolme_live_managed_signer_key_reference(
    value: &str,
    profile: &str,
    key_reference_env: &str,
) -> Result<String, ConfigError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "{key_reference_env} must not be empty for signer profile {profile} (managed_signer_key_reference_invalid)"
        )));
    }
    SecureSignerProvider::from_key_id(trimmed).map_err(|error| {
        ConfigError::RuntimeKolmeLive(format!(
            "{key_reference_env} contains invalid secure key reference for signer profile {profile}: {error} (managed_signer_key_reference_invalid)"
        ))
    })?;
    let key_role = SignerKeyRole::from_key_id(trimmed).map_err(|error| {
        ConfigError::RuntimeKolmeLive(format!(
            "{key_reference_env} contains invalid signer role for signer profile {profile}: {error} (managed_signer_key_reference_invalid)"
        ))
    })?;
    if key_role != SignerKeyRole::Operator {
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "{key_reference_env} must resolve to signer role operator for signer profile {profile}; found {} (managed_signer_key_reference_role_invalid)",
            key_role.label()
        )));
    }
    Ok(trimmed.to_owned())
}

fn resolve_kolme_live_signer_selection(
    strict_signer_profile: Option<&str>,
    strict_signer_key_source: Option<&str>,
) -> Result<KolmeLiveSignerSelection, ConfigError> {
    let key_source = if let Some(key_source) = strict_signer_key_source {
        normalize_kolme_live_signer_key_source(key_source)?
    } else if strict_signer_profile.is_some() {
        return Err(ConfigError::RuntimeKolmeLive(
            "--kolme-live-signer-key-source must be declared for strict signer contracts"
                .to_owned(),
        ));
    } else {
        KOLME_LIVE_SIGNER_KEY_SOURCE_ENV_LOCAL
    };
    let (profile, private_key_env, key_reference_env) =
        resolve_kolme_live_signer_env_name_set(strict_signer_profile)?;
    Ok(KolmeLiveSignerSelection {
        profile,
        key_source,
        private_key_env,
        key_reference_env,
    })
}

trait KolmeLiveSignerSecretProvider {
    fn ensure_no_fallback_private_key_path(&self) -> Result<(), ConfigError>;

    fn read_private_key_hex(
        &self,
        selection: &KolmeLiveSignerSelection,
    ) -> Result<String, ConfigError>;
}

struct EnvKolmeLiveSignerSecretProvider;

impl KolmeLiveSignerSecretProvider for EnvKolmeLiveSignerSecretProvider {
    fn ensure_no_fallback_private_key_path(&self) -> Result<(), ConfigError> {
        match env::var(KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK_ENV) {
            Ok(_) => Err(ConfigError::RuntimeKolmeLive(format!(
                "{KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK_ENV} must remain unset (fallback_signer_secret_present_violation)"
            ))),
            Err(env::VarError::NotPresent) => Ok(()),
            Err(env::VarError::NotUnicode(_)) => Err(ConfigError::RuntimeKolmeLive(format!(
                "{KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK_ENV} must be valid utf-8 when present (fallback_signer_secret_present_violation)"
            ))),
        }
    }

    fn read_private_key_hex(
        &self,
        selection: &KolmeLiveSignerSelection,
    ) -> Result<String, ConfigError> {
        match env::var(selection.private_key_env) {
            Ok(private_key_hex) => Ok(private_key_hex),
            Err(env::VarError::NotPresent) => Err(ConfigError::RuntimeKolmeLive(format!(
                "{} must be set for signer profile {}",
                selection.private_key_env, selection.profile
            ))),
            Err(env::VarError::NotUnicode(_)) => Err(ConfigError::RuntimeKolmeLive(format!(
                "{} must be valid utf-8 for signer profile {}",
                selection.private_key_env, selection.profile
            ))),
        }
    }
}

fn read_kolme_live_signer_private_key_hex_with_provider<P: KolmeLiveSignerSecretProvider>(
    strict_signer_profile: Option<&str>,
    strict_signer_key_source: Option<&str>,
    provider: &P,
) -> Result<(String, KolmeLiveSignerSelection), ConfigError> {
    let selection =
        resolve_kolme_live_signer_selection(strict_signer_profile, strict_signer_key_source)?;
    provider.ensure_no_fallback_private_key_path()?;
    if selection.key_source == KOLME_LIVE_SIGNER_KEY_SOURCE_MANAGED_EXTERNAL {
        let key_reference = read_required_kolme_live_key_reference_from_env(&selection)?;
        ensure_kolme_live_managed_external_private_key_env_unset(&selection)?;
        return Err(ConfigError::RuntimeKolmeLive(format!(
            "managed-external signer key reference {key_reference} from {} cannot use private-key signer adapter path; route through managed signer backend execution (managed_signer_private_key_adapter_unsupported)",
            selection.key_reference_env
        )));
    }
    let private_key_hex = provider.read_private_key_hex(&selection)?;
    Ok((private_key_hex, selection))
}

fn ensure_kolme_live_managed_external_private_key_env_unset(
    selection: &KolmeLiveSignerSelection,
) -> Result<(), ConfigError> {
    match env::var(selection.private_key_env) {
        Ok(_) => Err(ConfigError::RuntimeKolmeLive(format!(
            "{} must remain unset for signer profile {} when --kolme-live-signer-key-source={} (managed_signer_raw_private_key_forbidden)",
            selection.private_key_env, selection.profile, KOLME_LIVE_SIGNER_KEY_SOURCE_MANAGED_EXTERNAL
        ))),
        Err(env::VarError::NotUnicode(_)) => Err(ConfigError::RuntimeKolmeLive(format!(
            "{} must be valid utf-8 when present for signer profile {} (managed_signer_raw_private_key_forbidden)",
            selection.private_key_env, selection.profile
        ))),
        Err(env::VarError::NotPresent) => Ok(()),
    }
}

fn read_kolme_live_signer_private_key_hex(
    strict_signer_profile: Option<&str>,
    strict_signer_key_source: Option<&str>,
) -> Result<(String, KolmeLiveSignerSelection), ConfigError> {
    let provider = EnvKolmeLiveSignerSecretProvider;
    read_kolme_live_signer_private_key_hex_with_provider(
        strict_signer_profile,
        strict_signer_key_source,
        &provider,
    )
}

impl KolmeForkSecp256k1SignerAdapter {
    pub(crate) fn from_private_key_hex(
        private_key_hex: &str,
        private_key_env: &'static str,
    ) -> Result<Self, ConfigError> {
        let private_key_bytes = decode_kolme_hex_bytes(private_key_hex, private_key_env)?;
        let signing_key =
            SigningKey::from_slice(private_key_bytes.as_slice()).map_err(|error| {
                ConfigError::RuntimeKolmeLive(format!(
                    "{private_key_env} is not a valid secp256k1 private key: {error}",
                ))
            })?;
        Ok(Self {
            signing_key,
            private_key_env,
        })
    }

    pub(crate) fn public_key_compressed_hex(&self) -> String {
        encode_kolme_hex_lower(
            self.signing_key
                .verifying_key()
                .to_encoded_point(true)
                .as_bytes(),
        )
    }

    pub(crate) fn verify_message(
        &self,
        message: &str,
        signature_hex: &str,
        recovery_id: u8,
    ) -> Result<(), ConfigError> {
        let signature_bytes =
            decode_kolme_hex_bytes(signature_hex, "runtime_commit_signature_hex")?;
        if signature_bytes.len() != 64 {
            return Err(ConfigError::RuntimeKolmeLive(
                "runtime commit signature hex must decode to exactly 64 bytes".to_owned(),
            ));
        }
        let signature = Signature::from_slice(signature_bytes.as_slice()).map_err(|error| {
            ConfigError::RuntimeKolmeLive(format!(
                "runtime commit signature bytes are invalid secp256k1 material: {error}",
            ))
        })?;
        let recovery = RecoveryId::from_byte(recovery_id).ok_or_else(|| {
            ConfigError::RuntimeKolmeLive(format!(
                "runtime commit recovery id must be within secp256k1 range [0,3], found {recovery_id}",
            ))
        })?;
        let recovered = VerifyingKey::recover_from_msg(message.as_bytes(), &signature, recovery)
            .map_err(|error| {
                ConfigError::RuntimeKolmeLive(format!(
                    "failed to recover secp256k1 public key from runtime commit signature: {error}",
                ))
            })?;
        if recovered != *self.signing_key.verifying_key() {
            return Err(ConfigError::RuntimeKolmeLive(format!(
                "runtime commit signature recovered public key does not match signer selection {}",
                self.private_key_env,
            )));
        }
        Ok(())
    }

    pub(crate) fn sign_message(&self, message: &str) -> Result<(String, u8), ConfigError> {
        let (signature, recovery_id) = self
            .signing_key
            .sign_recoverable(message.as_bytes())
            .map_err(|error| {
                ConfigError::RuntimeKolmeLive(format!(
                    "failed to sign live runtime commit payload: {error}",
                ))
            })?;
        let signature_hex = encode_kolme_hex_lower(signature.to_bytes().as_ref());
        let recovery_id = recovery_id.to_byte();
        self.verify_message(message, signature_hex.as_str(), recovery_id)?;
        Ok((signature_hex, recovery_id))
    }
}

pub(crate) fn build_kolme_live_signer_adapter(
    strict_signer_profile: Option<&str>,
    strict_signer_key_source: Option<&str>,
) -> Result<(KolmeForkSecp256k1SignerAdapter, KolmeLiveSignerSelection), ConfigError> {
    let (private_key_hex, selection) =
        read_kolme_live_signer_private_key_hex(strict_signer_profile, strict_signer_key_source)?;
    let signer_adapter = KolmeForkSecp256k1SignerAdapter::from_private_key_hex(
        private_key_hex.as_str(),
        selection.private_key_env,
    )?;
    Ok((signer_adapter, selection))
}

pub(crate) fn resolve_kolme_live_nonce(
    base_url: &str,
    transport: &mut KolmeRuntimeCommitHttpTransport,
    pubkey: &str,
) -> Result<u64, ConfigError> {
    let request = KolmeApiNextNonceRequest::new(pubkey)
        .map_err(|error| ConfigError::RuntimeKolmeLive(error.to_string()))?;
    let response = transport
        .fetch_next_nonce(base_url, KOLME_LIVE_NONCE_PATH, &request)
        .map_err(|error| match error {
            KolmeRuntimeCommitProviderError::Timeout => {
                ConfigError::RuntimeKolmeLive("nonce request timed out".to_owned())
            }
            KolmeRuntimeCommitProviderError::Unavailable { reason } => {
                ConfigError::RuntimeKolmeLive(format!("nonce request unavailable: {reason}"))
            }
            KolmeRuntimeCommitProviderError::MalformedResponse { reason } => {
                ConfigError::RuntimeKolmeLive(format!("nonce response malformed: {reason}"))
            }
        })?;
    Ok(response.next_nonce)
}

pub(crate) fn build_kolme_live_direct_signed_wire_payload(
    base_url: &str,
    transport: &mut KolmeRuntimeCommitHttpTransport,
    request: &KolmeRuntimeCommitRequest,
    strict_signer_profile: Option<&str>,
    strict_signer_key_source: Option<&str>,
) -> Result<(String, KolmeLiveSignerSelection), ConfigError> {
    let signer_selection =
        resolve_kolme_live_signer_selection(strict_signer_profile, strict_signer_key_source)?;
    let (canonical_message, signature_hex, recovery_id) = if signer_selection.key_source
        == KOLME_LIVE_SIGNER_KEY_SOURCE_MANAGED_EXTERNAL
    {
        let provider = EnvKolmeLiveSignerSecretProvider;
        provider.ensure_no_fallback_private_key_path()?;
        ensure_kolme_live_managed_external_private_key_env_unset(&signer_selection)?;
        let _managed_signer_required_marker = resolve_kolme_live_managed_signer_required_marker()?;
        let key_reference = read_required_kolme_live_key_reference_from_env(&signer_selection)?;
        let managed_signing_key = build_kolme_live_managed_signing_key(key_reference.as_str())?;
        let pubkey = encode_kolme_hex_lower(
            managed_signing_key
                .verifying_key()
                .to_encoded_point(true)
                .as_bytes(),
        );
        let nonce = resolve_kolme_live_nonce(base_url, transport, pubkey.as_str())?;
        let canonical_message =
            render_kolme_live_native_direct_message(request, pubkey.as_str(), nonce)?;
        let (signature_hex, recovery_id) = sign_kolme_live_managed_external_message(
            key_reference.as_str(),
            request,
            nonce,
            canonical_message.as_str(),
            SignerProviderHandshakeMatrix::with_uniform_availability(true),
        )?;
        let verifier = KolmeForkSecp256k1SignerAdapter {
            signing_key: managed_signing_key,
            private_key_env: signer_selection.key_reference_env,
        };
        verifier.verify_message(
            canonical_message.as_str(),
            signature_hex.as_str(),
            recovery_id,
        )?;
        (canonical_message, signature_hex, recovery_id)
    } else {
        let (signer_adapter, signer_selection_from_adapter) =
            build_kolme_live_signer_adapter(strict_signer_profile, strict_signer_key_source)?;
        let pubkey = signer_adapter.public_key_compressed_hex();
        let nonce = resolve_kolme_live_nonce(base_url, transport, pubkey.as_str())?;
        let canonical_message =
            render_kolme_live_native_direct_message(request, pubkey.as_str(), nonce)?;
        let (signature_hex, recovery_id) =
            signer_adapter.sign_message(canonical_message.as_str())?;
        debug_assert_eq!(
            signer_selection.profile,
            signer_selection_from_adapter.profile
        );
        debug_assert_eq!(
            signer_selection.key_source,
            signer_selection_from_adapter.key_source
        );
        debug_assert_eq!(
            signer_selection.private_key_env,
            signer_selection_from_adapter.private_key_env
        );
        (canonical_message, signature_hex, recovery_id)
    };
    let request = KolmeApiBroadcastRequest::new(
        canonical_message.as_str(),
        signature_hex.as_str(),
        recovery_id,
    )
    .map_err(|error| ConfigError::RuntimeKolmeLive(error.to_string()))?;
    Ok((request.to_json_payload(), signer_selection))
}
