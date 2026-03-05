use super::*;

const OWNER_ALPHA: &str = "kamn:did:owner:alpha";
const SENDER_ALPHA: &str = "kamn:did:agent:alpha-sender";
const RECIPIENT_ALPHA: &str = "kamn:did:agent:alpha-recipient";

fn alpha_dispatch_request(
    sender_agent_did: &str,
    recipient_agent_did: &str,
    message_id: &str,
    dispatched_at_epoch_seconds: u64,
) -> DataLayerM9DispatchRequest {
    dispatch_request(
        OWNER_ALPHA,
        OWNER_ALPHA,
        sender_agent_did,
        recipient_agent_did,
        message_id,
        dispatched_at_epoch_seconds,
    )
}

pub(super) fn run_spec_c14_invalid_requester_owner_did_fails_closed_with_field_taxonomy() {
    let mut registry = DataLayerM9RealtimeDeliveryRegistry::new();
    let invalid_requester_owner = registry.dispatch_message(dispatch_request(
        "did:example:intruder",
        "kamn:did:owner:alpha",
        "kamn:did:agent:alpha-sender",
        "kamn:did:agent:alpha-recipient",
        "m9-invalid-owner-requester",
        1_708_560_900,
    ));
    assert!(matches!(
        invalid_requester_owner,
        Err(DataLayerM9RealtimeDeliveryError::InvalidDid {
            field: "requester_owner_did",
            reason_code: DATA_LAYER_M9_INVALID_REQUESTER_OWNER_DID_REASON_CODE,
            ..
        })
    ));
}

pub(super) fn run_spec_c15_invalid_sender_and_recipient_agent_dids_fail_closed_with_field_taxonomy()
{
    let mut registry = DataLayerM9RealtimeDeliveryRegistry::new();

    let invalid_sender = registry.dispatch_message(alpha_dispatch_request(
        "kamn:did:owner:not-an-agent",
        RECIPIENT_ALPHA,
        "m9-invalid-sender",
        1_708_560_901,
    ));
    assert!(matches!(
        invalid_sender,
        Err(DataLayerM9RealtimeDeliveryError::InvalidDid {
            field: "sender_agent_did",
            reason_code: DATA_LAYER_M9_INVALID_SENDER_AGENT_DID_REASON_CODE,
            ..
        })
    ));

    let invalid_recipient = registry.dispatch_message(alpha_dispatch_request(
        SENDER_ALPHA,
        "kamn:did:agent:Recipient",
        "m9-invalid-recipient",
        1_708_560_902,
    ));
    assert!(matches!(
        invalid_recipient,
        Err(DataLayerM9RealtimeDeliveryError::InvalidDid {
            field: "recipient_agent_did",
            reason_code: DATA_LAYER_M9_INVALID_RECIPIENT_AGENT_DID_REASON_CODE,
            ..
        })
    ));
}

pub(super) fn run_spec_c16_invalid_presence_requester_agent_did_fails_closed_with_field_taxonomy() {
    let registry = DataLayerM9RealtimeDeliveryRegistry::new();
    let invalid_presence_requester = registry.query_presence(DataLayerM9PresenceQuery {
        requester_owner_did: OWNER_ALPHA.to_owned(),
        owner_did: OWNER_ALPHA.to_owned(),
        requester_agent_did: OWNER_ALPHA.to_owned(),
        target_agent_did: "kamn:did:agent:alpha-target".to_owned(),
    });
    assert!(matches!(
        invalid_presence_requester,
        Err(DataLayerM9RealtimeDeliveryError::InvalidDid {
            field: "requester_agent_did",
            reason_code: DATA_LAYER_M9_INVALID_REQUESTER_AGENT_DID_REASON_CODE,
            ..
        })
    ));
}
