use crate::AgentDid;

use super::{ApproverQuorumError, ListenerQuorumError};

pub(crate) const RUNTIME_LISTENER_QUORUM_INVALID_LISTENER_DID_REASON_CODE: &str =
    "runtime_listener_quorum_invalid_listener_did";
pub(crate) const RUNTIME_APPROVER_QUORUM_INVALID_APPROVER_DID_REASON_CODE: &str =
    "runtime_approver_quorum_invalid_approver_did";

pub(crate) fn parse_listener_did(
    value: &str,
    field: &'static str,
) -> Result<AgentDid, ListenerQuorumError> {
    AgentDid::parse(value).map_err(|error| ListenerQuorumError::InvalidListenerDid {
        field,
        reason_code: RUNTIME_LISTENER_QUORUM_INVALID_LISTENER_DID_REASON_CODE,
        detail: error.to_string(),
    })
}

pub(crate) fn parse_approver_did(
    value: &str,
    field: &'static str,
) -> Result<AgentDid, ApproverQuorumError> {
    AgentDid::parse(value).map_err(|error| ApproverQuorumError::InvalidApproverDid {
        field,
        reason_code: RUNTIME_APPROVER_QUORUM_INVALID_APPROVER_DID_REASON_CODE,
        detail: error.to_string(),
    })
}
