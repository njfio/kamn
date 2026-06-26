use crate::data_layer_m2_gateway_access::models::{
    parse_agent_did, parse_kamn_did, DataLayerM2GatewayError,
};
use crate::{AgentDid, KamnDid};

/// Actor role used by M2 ABAC authorization checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM2ActorRole {
    /// Agent variant for this public contract enum.
    Agent,
    /// Owner variant for this public contract enum.
    Owner,
    /// Escrow auditor variant for this public contract enum.
    EscrowAuditor,
    /// Platform operator variant for this public contract enum.
    PlatformOperator,
}

/// Message metadata scope inspected by M2 ABAC checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM2MessageScope {
    /// Message id carried by this public contract model.
    pub message_id: String,
    /// Sender did carried by this public contract model.
    pub sender_did: String,
    /// Recipient did carried by this public contract model.
    pub recipient_did: String,
    /// Owner sender did carried by this public contract model.
    pub owner_sender_did: String,
    /// Owner recipient did carried by this public contract model.
    pub owner_recipient_did: String,
    /// Escrow id carried by this public contract model.
    pub escrow_id: Option<String>,
}

/// Typed validated scope used by internal M2 ABAC authorization paths.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM2MessageScopeValidated {
    /// Message id carried by this public contract model.
    pub message_id: String,
    /// Sender did carried by this public contract model.
    pub sender_did: AgentDid,
    /// Recipient did carried by this public contract model.
    pub recipient_did: AgentDid,
    /// Owner sender did carried by this public contract model.
    pub owner_sender_did: KamnDid,
    /// Owner recipient did carried by this public contract model.
    pub owner_recipient_did: KamnDid,
    /// Escrow id carried by this public contract model.
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
    /// Allow variant for this public contract enum.
    Allow {
        /// Reason code carried by this enum variant.
        reason_code: &'static str,
    },
    /// Deny variant for this public contract enum.
    Deny {
        /// Reason code carried by this enum variant.
        reason_code: &'static str,
    },
}

/// One expected-deny authorization case in the M2 negative matrix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM2NegativeAuthorizationCase {
    /// Case id carried by this public contract model.
    pub case_id: String,
    /// Requester did carried by this public contract model.
    pub requester_did: String,
    /// Requester role carried by this public contract model.
    pub requester_role: DataLayerM2ActorRole,
    /// Scope carried by this public contract model.
    pub scope: DataLayerM2MessageScope,
    /// Expected denied carried by this public contract model.
    pub expected_denied: bool,
    /// Event epoch seconds carried by this public contract model.
    pub event_epoch_seconds: u64,
}

/// Per-case audit fixture emitted by negative authorization matrix evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM2NegativeAuthorizationAuditFixture {
    /// Case id carried by this public contract model.
    pub case_id: String,
    /// Denied carried by this public contract model.
    pub denied: bool,
    /// Expected denied carried by this public contract model.
    pub expected_denied: bool,
    /// Mismatch carried by this public contract model.
    pub mismatch: bool,
    /// Decision reason code carried by this public contract model.
    pub decision_reason_code: &'static str,
    /// Audit record carried by this public contract model.
    pub audit_record: crate::data_layer_m2_gateway_access::DataLayerM2AccessAuditRecord,
}

/// Aggregate matrix decision marker for negative authorization evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM2NegativeAuthorizationMatrixDecision {
    /// All denied variant for this public contract enum.
    AllDenied {
        /// Reason code carried by this enum variant.
        reason_code: &'static str,
    },
    /// Drift detected variant for this public contract enum.
    DriftDetected {
        /// Reason code carried by this enum variant.
        reason_code: &'static str,
    },
}

/// Aggregate negative authorization matrix evaluation report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataLayerM2NegativeAuthorizationMatrixReport {
    /// Decision carried by this public contract model.
    pub decision: DataLayerM2NegativeAuthorizationMatrixDecision,
    /// Fixtures carried by this public contract model.
    pub fixtures: Vec<DataLayerM2NegativeAuthorizationAuditFixture>,
}
