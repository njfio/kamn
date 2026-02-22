use crate::errors::AgentLibError;
use kamn_sdk::AgentDid;

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
        let did = AgentDid::parse(format!("kamn:did:agent:{normalized_name}"))?;
        Ok(Self {
            did,
            signing_key: format!("ed25519:{normalized_name}:signing"),
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

#[cfg(test)]
mod tests {
    use super::AgentIdentity;

    #[test]
    fn unit_agent_identity_from_name_builds_expected_did_and_keys() {
        let identity = AgentIdentity::from_agent_name("Alice").expect("identity should build");
        assert_eq!(identity.did().as_str(), "kamn:did:agent:alice");
        assert_eq!(identity.signing_key(), "ed25519:alice:signing");
        assert_eq!(identity.encryption_key(), "x25519:alice:encryption");
    }

    #[test]
    fn unit_agent_identity_rejects_invalid_name_characters() {
        let error = AgentIdentity::from_agent_name("alice space")
            .expect_err("invalid characters must be rejected");
        assert!(error.to_string().contains("agent_name"));
    }
}
