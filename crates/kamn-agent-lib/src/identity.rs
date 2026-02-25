use crate::errors::AgentLibError;
use kamn_sdk::{service_public_key_for_private_key, AgentDid};

/// Agent identity material used by phase-1 authenticated operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentIdentity {
    did: AgentDid,
    signing_key: String,
    encryption_key: String,
}

impl AgentIdentity {
    /// Builds deterministic identity material from an agent name.
    pub fn from_agent_name(name: &str) -> Result<Self, AgentLibError> {
        let normalized_name = normalize_agent_name(name)?;
        let signing_key = derive_deterministic_service_signing_key_hex(normalized_name.as_str())?;
        let did = AgentDid::parse(format!("kamn:did:agent:{normalized_name}"))?;
        Ok(Self {
            did,
            signing_key,
            encryption_key: format!("x25519:{normalized_name}:encryption"),
        })
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
    let mut state: u64 = 0xcbf2_9ce4_8422_2325;
    let input = normalized_name.as_bytes();
    let mut output = [0u8; 32];
    for (index, slot) in output.iter_mut().enumerate() {
        let source = input[index % input.len()];
        state ^= (source as u64).wrapping_add(((index as u64) << 8) ^ 0x9e37_79b9_7f4a_7c15);
        state = state.wrapping_mul(0x0000_0100_0000_01b3);
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
    use super::AgentIdentity;

    #[test]
    fn unit_agent_identity_from_name_builds_expected_did_and_keys() {
        let identity = AgentIdentity::from_agent_name("Alice").expect("identity should build");
        assert_eq!(identity.did().as_str(), "kamn:did:agent:alice");
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
}
