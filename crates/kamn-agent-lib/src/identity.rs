use crate::errors::AgentLibError;
use kamn_sdk::{service_public_key_for_private_key, AgentDid};
use std::env;
use std::sync::atomic::{compiler_fence, Ordering};

const DETERMINISTIC_IDENTITY_ALLOW_ENV: &str = "KAMN_AGENT_LIB_ALLOW_DETERMINISTIC_IDENTITY";
const FNV_OFFSET_BASIS_64: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME_64: u64 = 0x0000_0100_0000_01b3;
const NAME_SEED_INDEX_SALT_XOR: u64 = 0x9e37_79b9_7f4a_7c15;

/// Agent identity material used by phase-1 authenticated operations.
#[derive(Debug, PartialEq, Eq)]
pub struct AgentIdentity {
    did: AgentDid,
    signing_key: String,
    encryption_key: String,
}

impl Drop for AgentIdentity {
    fn drop(&mut self) {
        let mut signing_key_bytes = drain_signing_key_bytes(&mut self.signing_key);
        wipe_secret_bytes(signing_key_bytes.as_mut_slice());
    }
}

impl AgentIdentity {
    /// Builds deterministic identity material from an agent name.
    ///
    /// # Security Warning
    /// Deterministic identities are intended for non-production workflows.
    /// Production builds deny this path by default unless
    /// `KAMN_AGENT_LIB_ALLOW_DETERMINISTIC_IDENTITY` is explicitly enabled.
    pub fn from_agent_name(name: &str) -> Result<Self, AgentLibError> {
        agent_identity_from_name_with_policy(name, deterministic_identity_allowed())
    }

    /// Builds identity material from explicit DID and signing-key inputs.
    pub fn from_did_and_signing_key(did: &str, signing_key: &str) -> Result<Self, AgentLibError> {
        let parsed_did = AgentDid::parse(did.to_owned())?;
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

fn agent_identity_from_name_with_policy(
    name: &str,
    deterministic_identity_allowed: bool,
) -> Result<AgentIdentity, AgentLibError> {
    let normalized_name = normalize_agent_name(name)?;
    if !deterministic_identity_allowed {
        return Err(AgentLibError::InvalidInput {
            field: "agent_name",
            reason: format!(
                "deterministic identity derivation is disabled by default for production builds; use AgentIdentity::from_did_and_signing_key or set {DETERMINISTIC_IDENTITY_ALLOW_ENV}=1 to opt in explicitly"
            ),
        });
    }
    let signing_key = derive_deterministic_service_signing_key_hex(normalized_name.as_str())?;
    let did = AgentDid::parse(format!("kamn:did:agent:{normalized_name}"))?;
    Ok(AgentIdentity {
        did,
        signing_key,
        encryption_key: format!("x25519:{normalized_name}:encryption"),
    })
}

fn deterministic_identity_allowed() -> bool {
    deterministic_identity_allowed_with_env(
        cfg!(debug_assertions),
        env::var(DETERMINISTIC_IDENTITY_ALLOW_ENV),
    )
}

fn deterministic_identity_allowed_with_env(
    debug_assertions_enabled: bool,
    env_value: Result<String, env::VarError>,
) -> bool {
    if debug_assertions_enabled {
        return true;
    }
    match env_value {
        Ok(value) => {
            let normalized = value.trim().to_ascii_lowercase();
            matches!(normalized.as_str(), "1" | "true" | "yes")
        }
        Err(env::VarError::NotPresent) => false,
        Err(env::VarError::NotUnicode(_)) => false,
    }
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

fn derive_name_seed_bytes(normalized_name: &str) -> [u8; 32] {
    let mut state = FNV_OFFSET_BASIS_64;
    let input = normalized_name.as_bytes();
    let mut output = [0u8; 32];
    for (index, slot) in output.iter_mut().enumerate() {
        state = fnv1a64_round(state, input[index % input.len()]);
        let index_salt = (((index as u64) << 8) ^ NAME_SEED_INDEX_SALT_XOR).to_le_bytes();
        for byte in index_salt {
            state = fnv1a64_round(state, byte);
        }
        *slot = (state >> ((index % 8) * 8)) as u8;
    }
    // Ensure non-zero scalar candidate.
    output[0] |= 0x01;
    output
}

fn fnv1a64_round(state: u64, byte: u8) -> u64 {
    (state ^ (byte as u64)).wrapping_mul(FNV_PRIME_64)
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

fn drain_signing_key_bytes(signing_key: &mut String) -> Vec<u8> {
    core::mem::take(signing_key).into_bytes()
}

fn wipe_secret_bytes(bytes: &mut [u8]) {
    for byte in bytes.iter_mut() {
        *byte = 0;
    }
    compiler_fence(Ordering::SeqCst);
    let _ = std::hint::black_box(bytes);
}

#[cfg(test)]
mod tests {
    use super::AgentIdentity;
    use std::env;

    fn derive_name_seed_bytes_reference_fnv1a(normalized_name: &str) -> [u8; 32] {
        let input = normalized_name.as_bytes();
        let mut state = super::FNV_OFFSET_BASIS_64;
        let mut output = [0u8; 32];
        for (index, slot) in output.iter_mut().enumerate() {
            state = super::fnv1a64_round(state, input[index % input.len()]);
            let index_salt =
                (((index as u64) << 8) ^ super::NAME_SEED_INDEX_SALT_XOR).to_le_bytes();
            for byte in index_salt {
                state = super::fnv1a64_round(state, byte);
            }
            *slot = (state >> ((index % 8) * 8)) as u8;
        }
        output[0] |= 0x01;
        output
    }

    #[test]
    fn spec_c01_derive_name_seed_bytes_matches_explicit_fnv1a_round_reference() {
        let derived = super::derive_name_seed_bytes("alice");
        let expected = derive_name_seed_bytes_reference_fnv1a("alice");
        assert_eq!(derived, expected);
    }

    #[test]
    fn unit_agent_identity_from_name_builds_expected_did_and_keys() {
        let identity = AgentIdentity::from_agent_name("Alice").expect("identity should build");
        assert_eq!(identity.did().as_str(), "kamn:did:agent:alice");
        assert_eq!(
            identity.signing_key(),
            "3d8a1dbfd141e25f472298086ae0ce64b7057017ee110ad5fee6aba21dd89fb3"
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
    fn unit_deterministic_identity_policy_denies_production_without_override() {
        assert!(!super::deterministic_identity_allowed_with_env(
            false,
            Err(env::VarError::NotPresent)
        ));
    }

    #[test]
    fn unit_deterministic_identity_policy_allows_debug_without_override() {
        assert!(super::deterministic_identity_allowed_with_env(
            true,
            Err(env::VarError::NotPresent)
        ));
    }

    #[test]
    fn unit_deterministic_identity_policy_allows_explicit_production_override() {
        assert!(super::deterministic_identity_allowed_with_env(
            false,
            Ok("1".to_owned())
        ));
        assert!(super::deterministic_identity_allowed_with_env(
            false,
            Ok("TRUE".to_owned())
        ));
        assert!(super::deterministic_identity_allowed_with_env(
            false,
            Ok(" yes ".to_owned())
        ));
    }

    #[test]
    fn regression_agent_identity_from_name_rejects_when_policy_disables_deterministic_identity() {
        // Regression: #6064
        let error = super::agent_identity_from_name_with_policy("alice", false)
            .expect_err("deterministic identity should be blocked");
        let rendered = error.to_string();
        assert!(rendered.contains("deterministic identity derivation is disabled"));
        assert!(rendered.contains("from_did_and_signing_key"));
    }

    #[test]
    fn regression_agent_identity_signing_key_scrub_helpers_zeroize_bytes() {
        // Regression: #6128
        let mut signing_key = String::from("0123abcd");
        let mut bytes = super::drain_signing_key_bytes(&mut signing_key);
        assert!(signing_key.is_empty());
        super::wipe_secret_bytes(bytes.as_mut_slice());
        assert!(bytes.iter().all(|byte| *byte == 0));
    }
}
