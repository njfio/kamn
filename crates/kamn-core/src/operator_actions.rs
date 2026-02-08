use crate::{OperatorBindingAction, OperatorBindingEngine, OperatorBindingError};
use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperatorActionOutcome {
    Allowed,
    Denied,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatorActionAuditRecord {
    pub agent_did: String,
    pub operator_did: String,
    pub action: OperatorBindingAction,
    pub target: String,
    pub value: Option<String>,
    pub requested_at_unix: u64,
    pub outcome: OperatorActionOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionedOperatorActionService {
    bindings: OperatorBindingEngine,
    settings: BTreeMap<(String, String), String>,
    audit_log: Vec<OperatorActionAuditRecord>,
}

impl PermissionedOperatorActionService {
    pub fn new(bindings: OperatorBindingEngine) -> Self {
        Self {
            bindings,
            settings: BTreeMap::new(),
            audit_log: Vec::new(),
        }
    }

    pub fn configure(
        &mut self,
        agent_did: &str,
        operator_did: &str,
        config_key: &str,
        config_value: &str,
        requested_at_unix: u64,
    ) -> Result<(), OperatorActionServiceError> {
        if config_key.trim().is_empty() {
            return Err(OperatorActionServiceError::EmptyField("config_key"));
        }
        if config_value.trim().is_empty() {
            return Err(OperatorActionServiceError::EmptyField("config_value"));
        }
        if requested_at_unix == 0 {
            return Err(OperatorActionServiceError::EmptyField("requested_at_unix"));
        }

        if let Err(error) =
            self.bindings
                .authorize(agent_did, operator_did, OperatorBindingAction::Configure)
        {
            self.push_audit(OperatorActionAuditRecord {
                agent_did: agent_did.to_owned(),
                operator_did: operator_did.to_owned(),
                action: OperatorBindingAction::Configure,
                target: config_key.to_owned(),
                value: Some(config_value.to_owned()),
                requested_at_unix,
                outcome: OperatorActionOutcome::Denied,
            });
            return Err(OperatorActionServiceError::Binding(error.to_string()));
        }

        self.settings.insert(
            (agent_did.to_owned(), config_key.to_owned()),
            config_value.to_owned(),
        );
        self.push_audit(OperatorActionAuditRecord {
            agent_did: agent_did.to_owned(),
            operator_did: operator_did.to_owned(),
            action: OperatorBindingAction::Configure,
            target: config_key.to_owned(),
            value: Some(config_value.to_owned()),
            requested_at_unix,
            outcome: OperatorActionOutcome::Allowed,
        });
        Ok(())
    }

    pub fn revoke_binding(
        &mut self,
        agent_did: &str,
        operator_did: &str,
        requested_at_unix: u64,
    ) -> Result<(), OperatorActionServiceError> {
        if requested_at_unix == 0 {
            return Err(OperatorActionServiceError::EmptyField("requested_at_unix"));
        }
        match self.bindings.revoke_binding(agent_did, operator_did) {
            Ok(()) => {
                self.push_audit(OperatorActionAuditRecord {
                    agent_did: agent_did.to_owned(),
                    operator_did: operator_did.to_owned(),
                    action: OperatorBindingAction::Revoke,
                    target: "binding".to_owned(),
                    value: None,
                    requested_at_unix,
                    outcome: OperatorActionOutcome::Allowed,
                });
                Ok(())
            }
            Err(error) => {
                self.push_audit(OperatorActionAuditRecord {
                    agent_did: agent_did.to_owned(),
                    operator_did: operator_did.to_owned(),
                    action: OperatorBindingAction::Revoke,
                    target: "binding".to_owned(),
                    value: None,
                    requested_at_unix,
                    outcome: OperatorActionOutcome::Denied,
                });
                Err(OperatorActionServiceError::Binding(error.to_string()))
            }
        }
    }

    pub fn read_history(
        &mut self,
        agent_did: &str,
        operator_did: &str,
        requested_at_unix: u64,
    ) -> Result<Vec<OperatorActionAuditRecord>, OperatorActionServiceError> {
        if requested_at_unix == 0 {
            return Err(OperatorActionServiceError::EmptyField("requested_at_unix"));
        }

        if let Err(error) =
            self.bindings
                .authorize(agent_did, operator_did, OperatorBindingAction::ReadHistory)
        {
            self.push_audit(OperatorActionAuditRecord {
                agent_did: agent_did.to_owned(),
                operator_did: operator_did.to_owned(),
                action: OperatorBindingAction::ReadHistory,
                target: "audit_log".to_owned(),
                value: None,
                requested_at_unix,
                outcome: OperatorActionOutcome::Denied,
            });
            return Err(OperatorActionServiceError::Binding(error.to_string()));
        }

        self.push_audit(OperatorActionAuditRecord {
            agent_did: agent_did.to_owned(),
            operator_did: operator_did.to_owned(),
            action: OperatorBindingAction::ReadHistory,
            target: "audit_log".to_owned(),
            value: None,
            requested_at_unix,
            outcome: OperatorActionOutcome::Allowed,
        });
        Ok(self.audit_log.clone())
    }

    pub fn setting(&self, agent_did: &str, config_key: &str) -> Option<String> {
        self.settings
            .get(&(agent_did.to_owned(), config_key.to_owned()))
            .cloned()
    }

    pub fn audit_log(&self) -> Vec<OperatorActionAuditRecord> {
        self.audit_log.clone()
    }

    fn push_audit(&mut self, record: OperatorActionAuditRecord) {
        self.audit_log.push(record);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperatorActionServiceError {
    EmptyField(&'static str),
    Binding(String),
}

impl fmt::Display for OperatorActionServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field) => write!(f, "field must not be empty: {field}"),
            Self::Binding(message) => write!(f, "operator binding error: {message}"),
        }
    }
}

impl std::error::Error for OperatorActionServiceError {}

impl From<OperatorBindingError> for OperatorActionServiceError {
    fn from(value: OperatorBindingError) -> Self {
        Self::Binding(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::{OperatorActionServiceError, PermissionedOperatorActionService};
    use crate::OperatorBindingEngine;

    #[test]
    fn configure_rejects_empty_key() {
        let mut service = PermissionedOperatorActionService::new(OperatorBindingEngine::new());
        assert_eq!(
            service.configure("kamn:did:agent:ops", "kamn:did:human:op", "", "on", 1),
            Err(OperatorActionServiceError::EmptyField("config_key"))
        );
    }
}
