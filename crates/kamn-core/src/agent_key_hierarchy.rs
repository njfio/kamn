use std::collections::BTreeMap;
use std::fmt;

/// Logical role binding for agent key material.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyRole {
    /// Long-lived identity root key role.
    Identity,
    /// Signing key role used for message and transaction signatures.
    Signing,
    /// Agreement key role used for session/key-agreement flows.
    Agreement,
}

/// Ephemeral session key metadata tracked per session identifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EphemeralSessionKey {
    /// Session-scoped key identifier.
    pub key_id: String,
    /// Expiration timestamp in seconds.
    pub expires_at_secs: u64,
}

/// In-memory key hierarchy manager for role-bound and ephemeral keys.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentKeyHierarchy {
    identity_key_id: String,
    signing_key_id: String,
    agreement_key_id: String,
    ephemeral_by_session: BTreeMap<String, EphemeralSessionKey>,
}

impl AgentKeyHierarchy {
    /// Creates a new hierarchy with distinct non-empty role key identifiers.
    pub fn new(
        identity_key_id: &str,
        signing_key_id: &str,
        agreement_key_id: &str,
    ) -> Result<Self, AgentKeyHierarchyError> {
        ensure_non_empty("identity", identity_key_id)?;
        ensure_non_empty("signing", signing_key_id)?;
        ensure_non_empty("agreement", agreement_key_id)?;
        ensure_distinct(identity_key_id, signing_key_id, agreement_key_id)?;

        Ok(Self {
            identity_key_id: identity_key_id.to_owned(),
            signing_key_id: signing_key_id.to_owned(),
            agreement_key_id: agreement_key_id.to_owned(),
            ephemeral_by_session: BTreeMap::new(),
        })
    }

    /// Returns the current key identifier for the requested role.
    pub fn current_key(&self, role: KeyRole) -> Result<&str, AgentKeyHierarchyError> {
        Ok(match role {
            KeyRole::Identity => &self.identity_key_id,
            KeyRole::Signing => &self.signing_key_id,
            KeyRole::Agreement => &self.agreement_key_id,
        })
    }

    /// Rotates the signing key while enforcing non-empty and cross-role uniqueness.
    pub fn rotate_signing_key(&mut self, key_id: &str) -> Result<(), AgentKeyHierarchyError> {
        ensure_non_empty("signing", key_id)?;
        if key_id == self.identity_key_id || key_id == self.agreement_key_id {
            return Err(AgentKeyHierarchyError::DuplicateRoleKey(key_id.to_owned()));
        }
        self.signing_key_id = key_id.to_owned();
        Ok(())
    }

    /// Rotates the agreement key while enforcing non-empty and cross-role uniqueness.
    pub fn rotate_agreement_key(&mut self, key_id: &str) -> Result<(), AgentKeyHierarchyError> {
        ensure_non_empty("agreement", key_id)?;
        if key_id == self.identity_key_id || key_id == self.signing_key_id {
            return Err(AgentKeyHierarchyError::DuplicateRoleKey(key_id.to_owned()));
        }
        self.agreement_key_id = key_id.to_owned();
        Ok(())
    }

    /// Registers a new ephemeral key for a session.
    ///
    /// Fails when the session id is empty, the key id is empty, the expiry is
    /// zero, or the session already exists.
    pub fn register_ephemeral(
        &mut self,
        session_id: &str,
        key_id: &str,
        expires_at_secs: u64,
    ) -> Result<(), AgentKeyHierarchyError> {
        if session_id.trim().is_empty() {
            return Err(AgentKeyHierarchyError::EmptySessionId);
        }
        ensure_non_empty("ephemeral", key_id)?;
        if expires_at_secs == 0 {
            return Err(AgentKeyHierarchyError::InvalidExpiry(expires_at_secs));
        }
        if self.ephemeral_by_session.contains_key(session_id) {
            return Err(AgentKeyHierarchyError::DuplicateSession(
                session_id.to_owned(),
            ));
        }

        self.ephemeral_by_session.insert(
            session_id.to_owned(),
            EphemeralSessionKey {
                key_id: key_id.to_owned(),
                expires_at_secs,
            },
        );
        Ok(())
    }

    /// Returns the ephemeral key metadata for a session id.
    pub fn ephemeral_key(
        &self,
        session_id: &str,
    ) -> Result<&EphemeralSessionKey, AgentKeyHierarchyError> {
        self.ephemeral_by_session
            .get(session_id)
            .ok_or_else(|| AgentKeyHierarchyError::SessionNotFound(session_id.to_owned()))
    }

    /// Removes an ephemeral session binding.
    ///
    /// Returns an error when the session id is not registered.
    pub fn retire_ephemeral(&mut self, session_id: &str) -> Result<(), AgentKeyHierarchyError> {
        if self.ephemeral_by_session.remove(session_id).is_none() {
            return Err(AgentKeyHierarchyError::SessionNotFound(
                session_id.to_owned(),
            ));
        }
        Ok(())
    }
}

/// Errors produced by key hierarchy validation and mutation operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentKeyHierarchyError {
    /// A role-bound or ephemeral key id was empty.
    EmptyKeyId(&'static str),
    /// A key id was reused across role bindings.
    DuplicateRoleKey(String),
    /// Session id for an ephemeral binding was empty.
    EmptySessionId,
    /// Ephemeral session already exists.
    DuplicateSession(String),
    /// Ephemeral session was not found.
    SessionNotFound(String),
    /// Ephemeral expiry value was invalid.
    InvalidExpiry(u64),
}

impl fmt::Display for AgentKeyHierarchyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyKeyId(role) => write!(f, "{role} key id must not be empty"),
            Self::DuplicateRoleKey(value) => write!(f, "key id reused across roles: {value}"),
            Self::EmptySessionId => write!(f, "session id must not be empty"),
            Self::DuplicateSession(value) => write!(f, "ephemeral session already exists: {value}"),
            Self::SessionNotFound(value) => write!(f, "ephemeral session not found: {value}"),
            Self::InvalidExpiry(value) => write!(f, "invalid ephemeral expiry: {value}"),
        }
    }
}

impl std::error::Error for AgentKeyHierarchyError {}

fn ensure_non_empty(role: &'static str, key_id: &str) -> Result<(), AgentKeyHierarchyError> {
    if key_id.trim().is_empty() {
        return Err(AgentKeyHierarchyError::EmptyKeyId(role));
    }
    Ok(())
}

fn ensure_distinct(
    identity_key_id: &str,
    signing_key_id: &str,
    agreement_key_id: &str,
) -> Result<(), AgentKeyHierarchyError> {
    if identity_key_id == signing_key_id
        || identity_key_id == agreement_key_id
        || signing_key_id == agreement_key_id
    {
        return Err(AgentKeyHierarchyError::DuplicateRoleKey(
            identity_key_id.to_owned(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{AgentKeyHierarchy, AgentKeyHierarchyError};

    #[test]
    fn constructor_rejects_duplicate_role_keys() {
        assert_eq!(
            AgentKeyHierarchy::new("id:key:v1", "id:key:v1", "agr:key:v1"),
            Err(AgentKeyHierarchyError::DuplicateRoleKey(
                "id:key:v1".to_owned()
            ))
        );
    }

    #[test]
    fn rotate_signing_rejects_duplicate_role_binding() {
        let mut hierarchy = AgentKeyHierarchy::new("id:key:v1", "sig:key:v1", "agr:key:v1")
            .expect("hierarchy should initialize");

        assert_eq!(
            hierarchy.rotate_signing_key("agr:key:v1"),
            Err(AgentKeyHierarchyError::DuplicateRoleKey(
                "agr:key:v1".to_owned()
            ))
        );
    }
}
