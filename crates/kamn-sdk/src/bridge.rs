use crate::{MessageId, SdkError};

/// Bridge identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BridgeId(pub u64);

/// Lifecycle view for a submitted or forwarded bridge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeStatus {
    /// SDK bridge identifier.
    pub bridge_id: BridgeId,
    /// Bridge lifecycle state.
    pub bridge_status: String,
    /// Forwarded target message identifier when available.
    pub target_message_id: Option<MessageId>,
    /// Forward transaction hash when available.
    pub forward_tx_hash: Option<String>,
}

impl BridgeStatus {
    pub(crate) fn submitted(bridge_id: &BridgeId) -> Self {
        Self {
            bridge_id: bridge_id.clone(),
            bridge_status: "submitted".to_owned(),
            target_message_id: None,
            forward_tx_hash: None,
        }
    }

    pub(crate) fn forwarded(
        bridge_id: &BridgeId,
        target_message_id: MessageId,
        forward_tx_hash: String,
    ) -> Self {
        Self {
            bridge_id: bridge_id.clone(),
            bridge_status: "forwarded".to_owned(),
            target_message_id: Some(target_message_id),
            forward_tx_hash: Some(forward_tx_hash),
        }
    }
}

pub(crate) fn target_network(target_network: &str) -> Result<&str, SdkError> {
    let normalized = target_network.trim();
    if normalized.is_empty() {
        return Err(SdkError::InvalidInput {
            field: "target_network",
            reason: "must not be empty",
        });
    }
    Ok(normalized)
}
