use super::decisions::{
    agent_decision, auditor_decision, deny_decision, owner_decision, validate_escrow_id,
};
use super::matrix::{build_fixture, matrix_decision, validate_case, validate_cases};
use super::models::{
    DataLayerM2ActorRole, DataLayerM2AuthorizationDecision, DataLayerM2MessageScope,
    DataLayerM2MessageScopeValidated, DataLayerM2NegativeAuthorizationAuditFixture,
    DataLayerM2NegativeAuthorizationCase, DataLayerM2NegativeAuthorizationMatrixReport,
};
use crate::data_layer_m2_gateway_access::audit::DataLayerM2AccessAuditLedger;
use crate::data_layer_m2_gateway_access::models::{
    parse_kamn_did, validate_requester_did_for_role, DataLayerM2GatewayError,
};
use crate::data_layer_m2_gateway_access::DATA_LAYER_M2_INVALID_AUDITOR_DID_REASON_CODE;
use std::collections::{BTreeMap, BTreeSet};

/// ABAC engine for M2 message visibility decisions.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DataLayerM2AbacEngine {
    pub(super) escrow_auditors_by_escrow: BTreeMap<String, BTreeSet<String>>,
    pub(super) disputed_escrows: BTreeSet<String>,
}

impl DataLayerM2AbacEngine {
    /// Creates a new value for this public contract type.
    pub fn new() -> Self {
        Self::default()
    }

    /// Runs the register escrow auditor contract operation.
    pub fn register_escrow_auditor(
        &mut self,
        escrow_id: &str,
        auditor_did: &str,
    ) -> Result<(), DataLayerM2GatewayError> {
        validate_escrow_id(escrow_id)?;
        let auditor_did = parse_kamn_did(
            auditor_did,
            "auditor_did",
            DATA_LAYER_M2_INVALID_AUDITOR_DID_REASON_CODE,
        )?;
        self.escrow_auditors_by_escrow
            .entry(escrow_id.to_owned())
            .or_default()
            .insert(auditor_did.as_str().to_owned());
        Ok(())
    }

    /// Runs the set escrow dispute active contract operation.
    pub fn set_escrow_dispute_active(&mut self, escrow_id: &str, active: bool) {
        if active {
            self.disputed_escrows.insert(escrow_id.to_owned());
        } else {
            self.disputed_escrows.remove(escrow_id);
        }
    }

    /// Runs the authorize message visibility contract operation.
    pub fn authorize_message_visibility(
        &self,
        requester_did: &str,
        requester_role: DataLayerM2ActorRole,
        scope: &DataLayerM2MessageScope,
    ) -> Result<DataLayerM2AuthorizationDecision, DataLayerM2GatewayError> {
        let requester = validate_requester_did_for_role(requester_did, requester_role)?;
        let scope = DataLayerM2MessageScopeValidated::try_from(scope)?;
        Ok(match requester_role {
            DataLayerM2ActorRole::Agent => agent_decision(requester.as_str(), &scope),
            DataLayerM2ActorRole::Owner => owner_decision(requester.as_str(), &scope),
            DataLayerM2ActorRole::EscrowAuditor => {
                auditor_decision(self, requester.as_str(), &scope)
            }
            DataLayerM2ActorRole::PlatformOperator => deny_decision(),
        })
    }

    /// Runs the evaluate negative authorization matrix contract operation.
    pub fn evaluate_negative_authorization_matrix(
        &self,
        cases: &[DataLayerM2NegativeAuthorizationCase],
    ) -> Result<DataLayerM2NegativeAuthorizationMatrixReport, DataLayerM2GatewayError> {
        validate_cases(cases)?;
        let mut audit_ledger = DataLayerM2AccessAuditLedger::new();
        let fixtures = cases
            .iter()
            .map(|case| self.evaluate_case(case, &mut audit_ledger))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(DataLayerM2NegativeAuthorizationMatrixReport {
            decision: matrix_decision(&fixtures),
            fixtures,
        })
    }

    fn evaluate_case(
        &self,
        case: &DataLayerM2NegativeAuthorizationCase,
        audit_ledger: &mut DataLayerM2AccessAuditLedger,
    ) -> Result<DataLayerM2NegativeAuthorizationAuditFixture, DataLayerM2GatewayError> {
        validate_case(case)?;
        let decision = self.authorize_message_visibility(
            case.requester_did.as_str(),
            case.requester_role,
            &case.scope,
        )?;
        build_fixture(case, decision, audit_ledger)
    }
}
