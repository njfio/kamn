use crate::AgentDid;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

const HUMAN_DID_PREFIX: &str = "kamn:did:human:";
const CANONICAL_PROOF_TYPE: &str = "Ed25519Signature2020";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OperatorBindingAction {
    Configure,
    Revoke,
    ReadHistory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatorBindingProof {
    pub type_name: String,
    pub created: String,
    pub verification_method: String,
    pub proof_value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatorBindingRecord {
    pub agent_did: String,
    pub operator_did: String,
    pub proof: Option<OperatorBindingProof>,
    pub permissions: BTreeSet<OperatorBindingAction>,
    pub revoked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OperatorBindingEngine {
    bindings: BTreeMap<(String, String), OperatorBindingRecord>,
}

impl OperatorBindingEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_binding(
        &mut self,
        agent_did: &str,
        operator_did: &str,
        proof: Option<OperatorBindingProof>,
        permissions: BTreeSet<OperatorBindingAction>,
    ) -> Result<(), OperatorBindingError> {
        validate_agent_did(agent_did)?;
        validate_operator_did(operator_did)?;
        if permissions.is_empty() {
            return Err(OperatorBindingError::EmptyPermissions);
        }

        if let Some(proof_value) = &proof {
            validate_proof(proof_value, operator_did)?;
        }

        let key = (agent_did.to_owned(), operator_did.to_owned());
        if self.bindings.contains_key(&key) {
            return Err(OperatorBindingError::DuplicateBinding {
                agent_did: agent_did.to_owned(),
                operator_did: operator_did.to_owned(),
            });
        }

        self.bindings.insert(
            key,
            OperatorBindingRecord {
                agent_did: agent_did.to_owned(),
                operator_did: operator_did.to_owned(),
                proof,
                permissions,
                revoked: false,
            },
        );
        Ok(())
    }

    pub fn authorize(
        &self,
        agent_did: &str,
        operator_did: &str,
        action: OperatorBindingAction,
    ) -> Result<(), OperatorBindingError> {
        validate_agent_did(agent_did)?;
        validate_operator_did(operator_did)?;

        let record = self.lookup(agent_did, operator_did)?;
        if record.revoked {
            return Err(OperatorBindingError::RevokedBinding {
                agent_did: agent_did.to_owned(),
                operator_did: operator_did.to_owned(),
            });
        }
        if !record.permissions.contains(&action) {
            return Err(OperatorBindingError::UnauthorizedAction {
                operator_did: operator_did.to_owned(),
                action,
            });
        }
        Ok(())
    }

    pub fn revoke_binding(
        &mut self,
        agent_did: &str,
        operator_did: &str,
    ) -> Result<(), OperatorBindingError> {
        validate_agent_did(agent_did)?;
        validate_operator_did(operator_did)?;

        let key = (agent_did.to_owned(), operator_did.to_owned());
        let record =
            self.bindings
                .get_mut(&key)
                .ok_or_else(|| OperatorBindingError::MissingBinding {
                    agent_did: agent_did.to_owned(),
                    operator_did: operator_did.to_owned(),
                })?;

        if record.revoked {
            return Err(OperatorBindingError::RevokedBinding {
                agent_did: agent_did.to_owned(),
                operator_did: operator_did.to_owned(),
            });
        }

        if !record.permissions.contains(&OperatorBindingAction::Revoke) {
            return Err(OperatorBindingError::UnauthorizedAction {
                operator_did: operator_did.to_owned(),
                action: OperatorBindingAction::Revoke,
            });
        }

        record.revoked = true;
        Ok(())
    }

    pub fn binding_for(
        &self,
        agent_did: &str,
        operator_did: &str,
    ) -> Result<&OperatorBindingRecord, OperatorBindingError> {
        validate_agent_did(agent_did)?;
        validate_operator_did(operator_did)?;
        self.lookup(agent_did, operator_did)
    }

    fn lookup(
        &self,
        agent_did: &str,
        operator_did: &str,
    ) -> Result<&OperatorBindingRecord, OperatorBindingError> {
        self.bindings
            .get(&(agent_did.to_owned(), operator_did.to_owned()))
            .ok_or_else(|| OperatorBindingError::MissingBinding {
                agent_did: agent_did.to_owned(),
                operator_did: operator_did.to_owned(),
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperatorBindingError {
    EmptyPermissions,
    EmptyProofField(&'static str),
    InvalidAgentDid(String),
    InvalidOperatorDid(String),
    InvalidProofType(String),
    ProofVerificationMethodMismatch {
        expected_prefix: String,
        actual: String,
    },
    DuplicateBinding {
        agent_did: String,
        operator_did: String,
    },
    MissingBinding {
        agent_did: String,
        operator_did: String,
    },
    RevokedBinding {
        agent_did: String,
        operator_did: String,
    },
    UnauthorizedAction {
        operator_did: String,
        action: OperatorBindingAction,
    },
}

impl fmt::Display for OperatorBindingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPermissions => write!(f, "permissions must not be empty"),
            Self::EmptyProofField(field) => write!(f, "proof field must not be empty: {field}"),
            Self::InvalidAgentDid(value) => write!(f, "invalid agent did: {value}"),
            Self::InvalidOperatorDid(value) => write!(f, "invalid operator did: {value}"),
            Self::InvalidProofType(value) => write!(f, "invalid proof type: {value}"),
            Self::ProofVerificationMethodMismatch {
                expected_prefix,
                actual,
            } => write!(
                f,
                "proof verification method mismatch, expected prefix {expected_prefix}, got {actual}"
            ),
            Self::DuplicateBinding {
                agent_did,
                operator_did,
            } => write!(f, "duplicate operator binding: {agent_did} + {operator_did}"),
            Self::MissingBinding {
                agent_did,
                operator_did,
            } => write!(f, "operator binding not found: {agent_did} + {operator_did}"),
            Self::RevokedBinding {
                agent_did,
                operator_did,
            } => write!(f, "operator binding revoked: {agent_did} + {operator_did}"),
            Self::UnauthorizedAction {
                operator_did,
                action,
            } => write!(
                f,
                "operator {operator_did} is unauthorized for action {action:?}"
            ),
        }
    }
}

impl std::error::Error for OperatorBindingError {}

fn validate_agent_did(value: &str) -> Result<(), OperatorBindingError> {
    AgentDid::parse(value)
        .map_err(|error| OperatorBindingError::InvalidAgentDid(error.to_string()))?;
    Ok(())
}

fn validate_operator_did(value: &str) -> Result<(), OperatorBindingError> {
    if !value.starts_with(HUMAN_DID_PREFIX) {
        return Err(OperatorBindingError::InvalidOperatorDid(format!(
            "invalid human did prefix: {value}"
        )));
    }
    let method_specific_id = &value[HUMAN_DID_PREFIX.len()..];
    if method_specific_id.is_empty() {
        return Err(OperatorBindingError::InvalidOperatorDid(
            "human did method-specific id must not be empty".to_owned(),
        ));
    }
    if !method_specific_id
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_')
    {
        return Err(OperatorBindingError::InvalidOperatorDid(format!(
            "human did has invalid characters: {method_specific_id}"
        )));
    }
    Ok(())
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<(), OperatorBindingError> {
    if value.trim().is_empty() {
        return Err(OperatorBindingError::EmptyProofField(field));
    }
    Ok(())
}

fn validate_proof(
    proof: &OperatorBindingProof,
    operator_did: &str,
) -> Result<(), OperatorBindingError> {
    validate_non_empty("type_name", &proof.type_name)?;
    validate_non_empty("created", &proof.created)?;
    validate_non_empty("verification_method", &proof.verification_method)?;
    validate_non_empty("proof_value", &proof.proof_value)?;

    if proof.type_name != CANONICAL_PROOF_TYPE {
        return Err(OperatorBindingError::InvalidProofType(
            proof.type_name.clone(),
        ));
    }

    let expected_prefix = format!("{operator_did}#");
    if !proof.verification_method.starts_with(&expected_prefix) {
        return Err(OperatorBindingError::ProofVerificationMethodMismatch {
            expected_prefix,
            actual: proof.verification_method.clone(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        OperatorBindingAction, OperatorBindingEngine, OperatorBindingError, OperatorBindingProof,
    };
    use std::collections::BTreeSet;

    fn proof_for(operator_did: &str) -> OperatorBindingProof {
        OperatorBindingProof {
            type_name: "Ed25519Signature2020".to_owned(),
            created: "2026-02-07T20:00:00Z".to_owned(),
            verification_method: format!("{operator_did}#keys-1"),
            proof_value: "z58proof".to_owned(),
        }
    }

    fn permissions(values: &[OperatorBindingAction]) -> BTreeSet<OperatorBindingAction> {
        values.iter().copied().collect()
    }

    #[test]
    fn register_rejects_invalid_operator_did() {
        let mut engine = OperatorBindingEngine::new();
        assert_eq!(
            engine.register_binding(
                "kamn:did:agent:agent-1",
                "did:example:operator",
                None,
                permissions(&[OperatorBindingAction::Configure]),
            ),
            Err(OperatorBindingError::InvalidOperatorDid(
                "invalid human did prefix: did:example:operator".to_owned()
            ))
        );
    }

    #[test]
    fn register_rejects_invalid_proof_verification_method() {
        let mut engine = OperatorBindingEngine::new();
        assert_eq!(
            engine.register_binding(
                "kamn:did:agent:agent-2",
                "kamn:did:human:operator-2",
                Some(OperatorBindingProof {
                    type_name: "Ed25519Signature2020".to_owned(),
                    created: "2026-02-07T20:00:00Z".to_owned(),
                    verification_method: "kamn:did:human:other#keys-1".to_owned(),
                    proof_value: "z58proof".to_owned(),
                }),
                permissions(&[OperatorBindingAction::Configure]),
            ),
            Err(OperatorBindingError::ProofVerificationMethodMismatch {
                expected_prefix: "kamn:did:human:operator-2#".to_owned(),
                actual: "kamn:did:human:other#keys-1".to_owned(),
            })
        );
    }

    #[test]
    fn revoke_requires_revoke_permission() {
        let mut engine = OperatorBindingEngine::new();
        engine
            .register_binding(
                "kamn:did:agent:agent-3",
                "kamn:did:human:operator-3",
                Some(proof_for("kamn:did:human:operator-3")),
                permissions(&[OperatorBindingAction::ReadHistory]),
            )
            .expect("binding should register");

        assert_eq!(
            engine.revoke_binding("kamn:did:agent:agent-3", "kamn:did:human:operator-3"),
            Err(OperatorBindingError::UnauthorizedAction {
                operator_did: "kamn:did:human:operator-3".to_owned(),
                action: OperatorBindingAction::Revoke,
            })
        );
    }

    #[test]
    fn authorize_allows_granted_action() {
        let mut engine = OperatorBindingEngine::new();
        engine
            .register_binding(
                "kamn:did:agent:agent-4",
                "kamn:did:human:operator-4",
                Some(proof_for("kamn:did:human:operator-4")),
                permissions(&[
                    OperatorBindingAction::Configure,
                    OperatorBindingAction::ReadHistory,
                    OperatorBindingAction::Revoke,
                ]),
            )
            .expect("binding should register");

        engine
            .authorize(
                "kamn:did:agent:agent-4",
                "kamn:did:human:operator-4",
                OperatorBindingAction::ReadHistory,
            )
            .expect("authorization should pass");
    }

    #[test]
    fn authorize_rejects_missing_binding() {
        let engine = OperatorBindingEngine::new();
        assert_eq!(
            engine.authorize(
                "kamn:did:agent:agent-9",
                "kamn:did:human:operator-9",
                OperatorBindingAction::ReadHistory,
            ),
            Err(OperatorBindingError::MissingBinding {
                agent_did: "kamn:did:agent:agent-9".to_owned(),
                operator_did: "kamn:did:human:operator-9".to_owned(),
            })
        );
    }
}
