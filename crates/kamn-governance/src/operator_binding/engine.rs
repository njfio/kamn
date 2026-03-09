use crate::operator_binding::principal::OperatorBindingPrincipals;
use crate::operator_binding::validation::validate_proof;
use crate::operator_binding::{
    OperatorBindingAction, OperatorBindingError, OperatorBindingProof, OperatorBindingRecord,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
/// Engine that manages operator bindings and authorization checks.
pub struct OperatorBindingEngine {
    bindings: BTreeMap<(String, String), OperatorBindingRecord>,
}

impl OperatorBindingEngine {
    /// Creates an empty operator binding engine.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a new operator binding with required permissions and optional proof.
    pub fn register_binding(
        &mut self,
        agent_did: &str,
        operator_did: &str,
        proof: Option<OperatorBindingProof>,
        permissions: BTreeSet<OperatorBindingAction>,
    ) -> Result<(), OperatorBindingError> {
        let principals = OperatorBindingPrincipals::parse(agent_did, operator_did)?;
        if permissions.is_empty() {
            return Err(OperatorBindingError::EmptyPermissions);
        }
        if let Some(proof_value) = &proof {
            validate_proof(proof_value, principals.operator_did.as_str())?;
        }

        let key = (
            principals.agent_did.as_str().to_owned(),
            principals.operator_did.as_str().to_owned(),
        );
        if self.bindings.contains_key(&key) {
            return Err(OperatorBindingError::DuplicateBinding {
                agent_did: principals.agent_did.as_str().to_owned(),
                operator_did: principals.operator_did.as_str().to_owned(),
            });
        }

        self.bindings.insert(
            key,
            OperatorBindingRecord {
                agent_did: principals.agent_did.as_str().to_owned(),
                operator_did: principals.operator_did.as_str().to_owned(),
                proof,
                permissions,
                revoked: false,
            },
        );
        Ok(())
    }

    /// Authorizes operator action against a non-revoked binding record.
    pub fn authorize(
        &self,
        agent_did: &str,
        operator_did: &str,
        action: OperatorBindingAction,
    ) -> Result<(), OperatorBindingError> {
        let principals = OperatorBindingPrincipals::parse(agent_did, operator_did)?;
        let record = self.lookup(
            principals.agent_did.as_str(),
            principals.operator_did.as_str(),
        )?;
        if record.revoked {
            return Err(OperatorBindingError::RevokedBinding {
                agent_did: principals.agent_did.as_str().to_owned(),
                operator_did: principals.operator_did.as_str().to_owned(),
            });
        }
        if !record.permissions.contains(&action) {
            return Err(OperatorBindingError::UnauthorizedAction {
                operator_did: principals.operator_did.as_str().to_owned(),
                action,
            });
        }
        Ok(())
    }

    /// Revokes an existing binding if caller has revoke permission.
    pub fn revoke_binding(
        &mut self,
        agent_did: &str,
        operator_did: &str,
    ) -> Result<(), OperatorBindingError> {
        let principals = OperatorBindingPrincipals::parse(agent_did, operator_did)?;
        let key = (
            principals.agent_did.as_str().to_owned(),
            principals.operator_did.as_str().to_owned(),
        );
        let record =
            self.bindings
                .get_mut(&key)
                .ok_or_else(|| OperatorBindingError::MissingBinding {
                    agent_did: principals.agent_did.as_str().to_owned(),
                    operator_did: principals.operator_did.as_str().to_owned(),
                })?;

        if record.revoked {
            return Err(OperatorBindingError::RevokedBinding {
                agent_did: principals.agent_did.as_str().to_owned(),
                operator_did: principals.operator_did.as_str().to_owned(),
            });
        }
        if !record.permissions.contains(&OperatorBindingAction::Revoke) {
            return Err(OperatorBindingError::UnauthorizedAction {
                operator_did: principals.operator_did.as_str().to_owned(),
                action: OperatorBindingAction::Revoke,
            });
        }
        record.revoked = true;
        Ok(())
    }

    /// Returns binding record for `(agent_did, operator_did)`.
    pub fn binding_for(
        &self,
        agent_did: &str,
        operator_did: &str,
    ) -> Result<&OperatorBindingRecord, OperatorBindingError> {
        let principals = OperatorBindingPrincipals::parse(agent_did, operator_did)?;
        self.lookup(
            principals.agent_did.as_str(),
            principals.operator_did.as_str(),
        )
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
