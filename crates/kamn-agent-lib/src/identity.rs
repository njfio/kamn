use crate::errors::AgentLibError;
use kamn_sdk::{service_public_key_for_private_key, AgentDid};
use std::env;
use zeroize::Zeroize;

const DETERMINISTIC_IDENTITY_OPT_IN_ENV: &str = "KAMN_AGENT_LIB_ALLOW_DETERMINISTIC_IDENTITY";
const DETERMINISTIC_IDENTITY_DISABLED_REASON: &str = "deterministic identity derivation disabled by default; use AgentIdentity::from_did_and_signing_key or set KAMN_AGENT_LIB_ALLOW_DETERMINISTIC_IDENTITY=1 for local development";
const FNV1A_64_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV1A_64_PRIME: u64 = 0x0000_0100_0000_01b3;

/// Agent identity material used by phase-1 authenticated operations.
#[derive(Debug, PartialEq, Eq)]
pub struct AgentIdentity {
    did: AgentDid,
    signing_key: String,
    encryption_key: String,
}

impl AgentIdentity {
    /// Builds deterministic identity material from an agent name.
    pub fn from_agent_name(name: &str) -> Result<Self, AgentLibError> {
        deterministic_identity_allowed()?;
        let normalized_name = normalize_agent_name(name)?;
        let signing_key = derive_deterministic_service_signing_key_hex(normalized_name.as_str())?;
        let signer_public_key = service_public_key_for_private_key(signing_key.as_str())?;
        let method_specific_id = format!("pkh-{}", signer_public_key.to_ascii_lowercase());
        let did = AgentDid::with_public_key_hex_binding(
            method_specific_id.as_str(),
            signer_public_key.as_str(),
        )
        .map_err(|error| {
            AgentLibError::Internal(format!(
                "failed to derive deterministic did key binding: {error}"
            ))
        })?;
        Ok(Self {
            did,
            signing_key,
            encryption_key: format!("x25519:{normalized_name}:encryption"),
        })
    }

    /// Builds identity material from explicit DID and signing-key inputs.
    pub fn from_did_and_signing_key(did: &str, signing_key: &str) -> Result<Self, AgentLibError> {
        let parsed_did = AgentDid::parse(did)?;
        if signing_key.trim().is_empty() {
            return Err(AgentLibError::InvalidInput {
                field: "signing_key",
                reason: "must not be empty".to_owned(),
            });
        }
        Ok(Self {
            did: parsed_did,
            signing_key: signing_key.trim().to_owned(),
            encryption_key: "x25519:derived:encryption".to_owned(),
        })
    }

    /// Returns the identity DID.
    pub fn did(&self) -> &AgentDid {
        &self.did
    }

    /// Returns deterministic signing-key material.
    pub fn signing_key(&self) -> &str {
        self.signing_key.as_str()
    }

    /// Returns deterministic encryption-key material.
    pub fn encryption_key(&self) -> &str {
        self.encryption_key.as_str()
    }
}

impl Drop for AgentIdentity {
    fn drop(&mut self) {
        self.signing_key.zeroize();
        self.encryption_key.zeroize();
    }
}

fn deterministic_identity_allowed() -> Result<(), AgentLibError> {
    deterministic_identity_allowed_from_env(
        env::var(DETERMINISTIC_IDENTITY_OPT_IN_ENV),
        cfg!(any(test, debug_assertions)),
    )
}

fn deterministic_identity_allowed_from_env(
    env_value: Result<String, env::VarError>,
    allow_insecure_default: bool,
) -> Result<(), AgentLibError> {
    if allow_insecure_default {
        return Ok(());
    }
    let is_enabled = match env_value {
        Ok(value) => matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes"
        ),
        Err(env::VarError::NotPresent) => false,
        Err(env::VarError::NotUnicode(_)) => false,
    };
    if is_enabled {
        return Ok(());
    }
    Err(AgentLibError::UnsupportedOperation(
        DETERMINISTIC_IDENTITY_DISABLED_REASON,
    ))
}

fn normalize_agent_name(name: &str) -> Result<String, AgentLibError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(AgentLibError::InvalidInput {
            field: "agent_name",
            reason: "must not be empty".to_owned(),
        });
    }

    if !trimmed
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        return Err(AgentLibError::InvalidInput {
            field: "agent_name",
            reason: "must use [a-zA-Z0-9_-] only".to_owned(),
        });
    }

    Ok(trimmed.to_ascii_lowercase())
}

fn derive_deterministic_service_signing_key_hex(
    normalized_name: &str,
) -> Result<String, AgentLibError> {
    let mut bytes = derive_name_seed_bytes(normalized_name);
    for offset in 0u16..=255 {
        bytes[31] = bytes[31].wrapping_add(offset as u8);
        let candidate = hex_encode(bytes.as_slice());
        if service_public_key_for_private_key(candidate.as_str()).is_ok() {
            return Ok(candidate);
        }
    }
    Err(AgentLibError::Internal(
        "failed to derive valid deterministic secp256k1 private key".to_owned(),
    ))
}

fn fnv1a_round_u64(state: u64, value: u64) -> u64 {
    (state ^ value).wrapping_mul(FNV1A_64_PRIME)
}

fn derive_name_seed_bytes(normalized_name: &str) -> [u8; 32] {
    let mut state: u64 = FNV1A_64_OFFSET_BASIS;
    let input = normalized_name.as_bytes();
    let mut output = [0u8; 32];
    for (index, slot) in output.iter_mut().enumerate() {
        let source = u64::from(input[index % input.len()]);
        let index_mix = ((index as u64) << 8) ^ 0x9e37_79b9_7f4a_7c15;
        let mixed_source = source.wrapping_add(index_mix);
        state = fnv1a_round_u64(state, mixed_source);
        *slot = (state >> ((index % 8) * 8)) as u8;
    }
    // Ensure non-zero scalar candidate.
    output[0] |= 0x01;
    output
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{deterministic_identity_allowed_from_env, fnv1a_round_u64, AgentIdentity};
    use kamn_sdk::{service_public_key_for_private_key, AgentDid};
    use std::env;

    #[test]
    fn unit_agent_identity_from_name_builds_expected_did_and_keys() {
        let identity = AgentIdentity::from_agent_name("Alice").expect("identity should build");
        let expected_public_key = service_public_key_for_private_key(identity.signing_key())
            .expect("deterministic signing key should produce compressed public key");
        let expected_did = AgentDid::with_public_key_hex_binding(
            format!("pkh-{expected_public_key}").as_str(),
            expected_public_key.as_str(),
        )
        .expect("expected did should build");
        assert_eq!(
            identity.did(),
            &expected_did,
            "identity did must embed signer public key key-binding fingerprint"
        );
        assert_eq!(
            identity.signing_key(),
            "094cf4e1f3d974bbf3e72233e2c2937e8fdb094740e0f017e010aa47ac1201ac"
        );
        assert!(identity
            .signing_key()
            .chars()
            .all(|ch| ch.is_ascii_hexdigit() && !ch.is_ascii_uppercase()));
        assert_eq!(identity.encryption_key(), "x25519:alice:encryption");
    }

    #[test]
    fn unit_agent_identity_from_name_derives_distinct_private_keys_per_name() {
        let alice = AgentIdentity::from_agent_name("alice").expect("alice identity should build");
        let bob = AgentIdentity::from_agent_name("bob").expect("bob identity should build");
        assert_ne!(
            alice.signing_key(),
            bob.signing_key(),
            "distinct normalized names should not collide on deterministic signing key derivation"
        );
    }

    #[test]
    fn unit_agent_identity_rejects_invalid_name_characters() {
        let error = AgentIdentity::from_agent_name("alice space")
            .expect_err("invalid characters must be rejected");
        assert!(error.to_string().contains("agent_name"));
    }

    #[test]
    fn regression_deterministic_identity_gate_blocks_non_test_mode_without_opt_in() {
        // Regression: #6187
        let error = deterministic_identity_allowed_from_env(Err(env::VarError::NotPresent), false)
            .expect_err("non-test mode should fail closed when deterministic gate is not enabled");
        assert!(
            error
                .to_string()
                .contains("deterministic identity derivation disabled by default"),
            "gate error should include deterministic disable marker: {error}"
        );
    }

    #[test]
    fn unit_deterministic_identity_gate_allows_test_mode_without_opt_in() {
        deterministic_identity_allowed_from_env(Err(env::VarError::NotPresent), true)
            .expect("test mode should permit deterministic identity derivation");
    }

    #[test]
    fn unit_deterministic_identity_gate_allows_non_test_mode_with_opt_in_env() {
        deterministic_identity_allowed_from_env(Ok("1".to_owned()), false)
            .expect("explicit opt-in env should permit deterministic identity derivation");
    }

    #[test]
    fn regression_issue_6209_name_seed_round_uses_fnv1a_ordering() {
        let state = 0xcbf2_9ce4_8422_2325_u64;
        let value = 0x4142_4344_4546_4748_u64;
        let expected_fnv1a = (state ^ value).wrapping_mul(0x0000_0100_0000_01b3_u64);
        let unexpected_fnv1 = state.wrapping_mul(0x0000_0100_0000_01b3_u64) ^ value;
        assert_eq!(fnv1a_round_u64(state, value), expected_fnv1a);
        assert_ne!(fnv1a_round_u64(state, value), unexpected_fnv1);
    }

    #[test]
    fn regression_issue_6207_agent_identity_enforces_drop_zeroization_and_no_clone_derive() {
        let source = include_str!("identity.rs");
        assert!(
            !source.contains("#[derive(Debug, Clone, PartialEq, Eq)]\npub struct AgentIdentity"),
            "AgentIdentity must not derive Clone after #6207"
        );
        assert!(
            source.contains("#[derive(Debug, PartialEq, Eq)]\npub struct AgentIdentity"),
            "AgentIdentity derive contract must remain explicit and clone-free"
        );
        assert!(
            source.contains("impl Drop for AgentIdentity"),
            "AgentIdentity drop-based zeroization must remain present"
        );
        assert!(
            source.contains("self.signing_key.zeroize();"),
            "signing key zeroization marker must remain present"
        );
    }
}
