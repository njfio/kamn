use crate::data_layer_m2_gateway_access::models::{
    parse_agent_did, parse_kamn_did, DataLayerM2GatewayError,
};
use crate::{AgentDid, KamnDid};

/// Actor role used by M2 ABAC authorization checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM2ActorRole {
    Agent,
    Owner,
    EscrowAuditor,
    PlatformOperator,
}

/// Message metadata scope inspected by M2 ABAC checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM2MessageScope {
    pub message_id: String,
    pub sender_did: String,
    pub recipient_did: String,
    pub owner_sender_did: String,
    pub owner_recipient_did: String,
    pub escrow_id: Option<String>,
}

/// Typed validated scope used by internal M2 ABAC authorization paths.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM2MessageScopeValidated {
    pub message_id: String,
    pub sender_did: AgentDid,
    pub recipient_did: AgentDid,
    pub owner_sender_did: KamnDid,
    pub owner_recipient_did: KamnDid,
    pub escrow_id: Option<String>,
}

impl TryFrom<&DataLayerM2MessageScope> for DataLayerM2MessageScopeValidated {
    type Error = DataLayerM2GatewayError;

    fn try_from(scope: &DataLayerM2MessageScope) -> Result<Self, Self::Error> {
        validate_message_id(scope.message_id.as_str())?;
        Ok(Self {
            message_id: scope.message_id.clone(),
            sender_did: parse_agent_did(
                scope.sender_did.as_str(),
                "sender_did",
                super::super::DATA_LAYER_M2_INVALID_SENDER_DID_REASON_CODE,
            )?,
            recipient_did: parse_agent_did(
                scope.recipient_did.as_str(),
                "recipient_did",
                super::super::DATA_LAYER_M2_INVALID_RECIPIENT_DID_REASON_CODE,
            )?,
            owner_sender_did: parse_kamn_did(
                scope.owner_sender_did.as_str(),
                "owner_sender_did",
                super::super::DATA_LAYER_M2_INVALID_OWNER_SENDER_DID_REASON_CODE,
            )?,
            owner_recipient_did: parse_kamn_did(
                scope.owner_recipient_did.as_str(),
                "owner_recipient_did",
                super::super::DATA_LAYER_M2_INVALID_OWNER_RECIPIENT_DID_REASON_CODE,
            )?,
            escrow_id: scope.escrow_id.clone(),
        })
    }
}

fn validate_message_id(message_id: &str) -> Result<(), DataLayerM2GatewayError> {
    if message_id.trim().is_empty() {
        return Err(DataLayerM2GatewayError::EmptyField("message_id"));
    }
    Ok(())
}

/// Authorization decision projected by ABAC evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM2AuthorizationDecision {
    Allow { reason_code: &'static str },
    Deny { reason_code: &'static str },
}

/// One expected-deny authorization case in the M2 negative matrix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM2NegativeAuthorizationCase {
    pub case_id: String,
    pub requester_did: String,
    pub requester_role: DataLayerM2ActorRole,
    pub scope: DataLayerM2MessageScope,
    pub expected_denied: bool,
    pub event_epoch_seconds: u64,
}

/// Per-case audit fixture emitted by negative authorization matrix evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM2NegativeAuthorizationAuditFixture {
    pub case_id: String,
    pub denied: bool,
    pub expected_denied: bool,
    pub mismatch: bool,
    pub decision_reason_code: &'static str,
    pub audit_record: crate::data_layer_m2_gateway_access::DataLayerM2AccessAuditRecord,
}

/// Aggregate matrix decision marker for negative authorization evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM2NegativeAuthorizationMatrixDecision {
    AllDenied { reason_code: &'static str },
    DriftDetected { reason_code: &'static str },
}

/// Aggregate negative authorization matrix evaluation report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM2NegativeAuthorizationMatrixReport {
    pub decision: DataLayerM2NegativeAuthorizationMatrixDecision,
    pub fixtures: Vec<DataLayerM2NegativeAuthorizationAuditFixture>,
}
