use super::constants::{
    DATA_LAYER_M2_HASH_ALGORITHM, DATA_LAYER_M2_INVALID_REQUESTER_DID_REASON_CODE,
};
use super::error::DataLayerM2GatewayError;
use crate::data_layer_m2_gateway_access::audit::DataLayerM2AccessAuditInput;
use crate::data_layer_m2_gateway_access::authorization::DataLayerM2ActorRole;
use crate::{data_layer_hashing::tagged_sha256, AgentDid, AgentDidError, KamnDid, KamnDidError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DataLayerM2RequesterDidValidated {
    Agent(AgentDid),
    KamnDid(KamnDid),
}

impl DataLayerM2RequesterDidValidated {
    pub(crate) fn as_str(&self) -> &str {
        match self {
            Self::Agent(agent_did) => agent_did.as_str(),
            Self::KamnDid(value) => value.as_str(),
        }
    }
}

pub(crate) fn validate_requester_did_for_role(
    requester_did: &str,
    requester_role: DataLayerM2ActorRole,
) -> Result<DataLayerM2RequesterDidValidated, DataLayerM2GatewayError> {
    match requester_role {
        DataLayerM2ActorRole::Agent => validate_agent_requester(requester_did),
        DataLayerM2ActorRole::Owner
        | DataLayerM2ActorRole::EscrowAuditor
        | DataLayerM2ActorRole::PlatformOperator => validate_kamn_requester(requester_did),
    }
}

fn validate_agent_requester(
    requester_did: &str,
) -> Result<DataLayerM2RequesterDidValidated, DataLayerM2GatewayError> {
    let did = parse_agent_did(
        requester_did,
        "requester_did",
        DATA_LAYER_M2_INVALID_REQUESTER_DID_REASON_CODE,
    )?;
    Ok(DataLayerM2RequesterDidValidated::Agent(did))
}

fn validate_kamn_requester(
    requester_did: &str,
) -> Result<DataLayerM2RequesterDidValidated, DataLayerM2GatewayError> {
    let did = parse_kamn_did(
        requester_did,
        "requester_did",
        DATA_LAYER_M2_INVALID_REQUESTER_DID_REASON_CODE,
    )?;
    Ok(DataLayerM2RequesterDidValidated::KamnDid(did))
}

pub(crate) fn validate_audit_input(
    input: &DataLayerM2AccessAuditInput,
) -> Result<(), DataLayerM2GatewayError> {
    validate_non_empty(input.action.as_str(), "action")?;
    validate_non_empty(input.resource_id.as_str(), "resource_id")?;
    validate_non_empty(input.reason_code.as_str(), "reason_code")?;
    if input.event_epoch_seconds == 0 {
        return Err(DataLayerM2GatewayError::EmptyField("event_epoch_seconds"));
    }
    parse_kamn_did(
        input.requester_did.as_str(),
        "requester_did",
        DATA_LAYER_M2_INVALID_REQUESTER_DID_REASON_CODE,
    )?;
    Ok(())
}

fn validate_non_empty(value: &str, field: &'static str) -> Result<(), DataLayerM2GatewayError> {
    if value.trim().is_empty() {
        return Err(DataLayerM2GatewayError::EmptyField(field));
    }
    Ok(())
}

fn map_kamn_did_error(
    error: KamnDidError,
    field: &'static str,
    reason_code: &'static str,
) -> DataLayerM2GatewayError {
    DataLayerM2GatewayError::InvalidDid {
        field,
        reason_code,
        detail: error.to_string(),
    }
}

fn map_agent_did_error(
    error: AgentDidError,
    field: &'static str,
    reason_code: &'static str,
) -> DataLayerM2GatewayError {
    DataLayerM2GatewayError::InvalidDid {
        field,
        reason_code,
        detail: error.to_string(),
    }
}

pub(crate) fn parse_kamn_did(
    value: &str,
    field: &'static str,
    reason_code: &'static str,
) -> Result<KamnDid, DataLayerM2GatewayError> {
    KamnDid::parse(value).map_err(|error| map_kamn_did_error(error, field, reason_code))
}

pub(crate) fn parse_agent_did(
    value: &str,
    field: &'static str,
    reason_code: &'static str,
) -> Result<AgentDid, DataLayerM2GatewayError> {
    AgentDid::parse(value).map_err(|error| map_agent_did_error(error, field, reason_code))
}

pub(crate) fn compute_audit_record_hash(
    sequence: u64,
    input: &DataLayerM2AccessAuditInput,
    hash_chain_prev: &str,
) -> String {
    tagged_digest(
        format!(
            "audit|seq:{sequence}|requester:{}|action:{}|resource:{}|reason:{}|event:{}|prev:{}",
            input.requester_did,
            input.action,
            input.resource_id,
            input.reason_code,
            input.event_epoch_seconds,
            hash_chain_prev
        )
        .as_str(),
    )
}

pub(crate) fn tagged_digest(value: &str) -> String {
    tagged_sha256(value, DATA_LAYER_M2_HASH_ALGORITHM)
}
