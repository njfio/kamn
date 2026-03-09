use super::attempts::{allowed_record, binding_attempt, denied_record};
use super::error::OperatorActionServiceError;
use super::model::OperatorActionAuditRecord;
use crate::operator_binding::{OperatorBindingAction, OperatorBindingEngine};
use std::collections::BTreeMap;

/// Service that gates operator actions through binding authorization and audit logging.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionedOperatorActionService {
    bindings: OperatorBindingEngine,
    settings: BTreeMap<(String, String), String>,
    audit_log: Vec<OperatorActionAuditRecord>,
}

impl PermissionedOperatorActionService {
    /// Creates a permissioned operator action service with the provided binding engine.
    pub fn new(bindings: OperatorBindingEngine) -> Self {
        Self {
            bindings,
            settings: BTreeMap::new(),
            audit_log: Vec::new(),
        }
    }

    /// Applies a configuration update when binding authorization allows the request.
    pub fn configure(
        &mut self,
        agent_did: &str,
        operator_did: &str,
        config_key: &str,
        config_value: &str,
        requested_at_unix: u64,
    ) -> Result<(), OperatorActionServiceError> {
        super::validation::validate_request(config_key, config_value, requested_at_unix)?;
        if let Err(error) =
            self.bindings
                .authorize(agent_did, operator_did, OperatorBindingAction::Configure)
        {
            self.record_attempt(denied_record(
                agent_did,
                operator_did,
                OperatorBindingAction::Configure,
                config_key,
                Some(config_value.to_owned()),
                requested_at_unix,
            ));
            return Err(OperatorActionServiceError::Binding(error));
        }
        self.settings.insert(
            (agent_did.to_owned(), config_key.to_owned()),
            config_value.to_owned(),
        );
        self.record_attempt(allowed_record(
            agent_did,
            operator_did,
            OperatorBindingAction::Configure,
            config_key,
            Some(config_value.to_owned()),
            requested_at_unix,
        ));
        Ok(())
    }

    /// Revokes an operator binding and records an audit decision.
    pub fn revoke_binding(
        &mut self,
        agent_did: &str,
        operator_did: &str,
        requested_at_unix: u64,
    ) -> Result<(), OperatorActionServiceError> {
        super::validation::require_requested_at(requested_at_unix)?;
        let attempt = binding_attempt(
            agent_did,
            operator_did,
            requested_at_unix,
            OperatorBindingAction::Revoke,
        );
        match self.bindings.revoke_binding(agent_did, operator_did) {
            Ok(()) => {
                self.record_attempt(allowed_record(
                    &attempt.0, &attempt.1, attempt.2, &attempt.3, None, attempt.4,
                ));
                Ok(())
            }
            Err(error) => {
                self.record_attempt(denied_record(
                    &attempt.0, &attempt.1, attempt.2, &attempt.3, None, attempt.4,
                ));
                Err(OperatorActionServiceError::Binding(error))
            }
        }
    }

    /// Reads action audit history after authorization against read-history capability.
    pub fn read_history(
        &mut self,
        agent_did: &str,
        operator_did: &str,
        requested_at_unix: u64,
    ) -> Result<Vec<OperatorActionAuditRecord>, OperatorActionServiceError> {
        super::validation::require_requested_at(requested_at_unix)?;
        let action = OperatorBindingAction::ReadHistory;
        if let Err(error) = self.bindings.authorize(agent_did, operator_did, action) {
            self.record_attempt(denied_record(
                agent_did,
                operator_did,
                action,
                "audit_log",
                None,
                requested_at_unix,
            ));
            return Err(OperatorActionServiceError::Binding(error));
        }
        self.record_attempt(allowed_record(
            agent_did,
            operator_did,
            action,
            "audit_log",
            None,
            requested_at_unix,
        ));
        Ok(self.audit_log.clone())
    }

    /// Returns the configured value for `(agent_did, config_key)` if present.
    pub fn setting(&self, agent_did: &str, config_key: &str) -> Option<String> {
        self.settings
            .get(&(agent_did.to_owned(), config_key.to_owned()))
            .cloned()
    }

    /// Returns a snapshot copy of the full audit log.
    pub fn audit_log(&self) -> Vec<OperatorActionAuditRecord> {
        self.audit_log.clone()
    }

    fn record_attempt(&mut self, record: OperatorActionAuditRecord) {
        self.audit_log.push(record);
    }
}
