use super::task_escrow::escape_json;
use crate::{
    AgentDid, AgentMetadata, AgentReputation, DidDocument, Message, MessageId, MessageRecord,
    SdkError, ServiceAgentProfile,
};
use crate::service::ServiceMessageDelivery;

pub(crate) fn service_message_payload(message: &Message) -> String {
    let channel_segment = match &message.channel {
        Some(channel_id) => {
            format!(",\"channel_id\":\"{}\"", escape_json(channel_id.0.as_str()))
        }
        None => String::new(),
    };

    format!(
        "{{\"from\":\"{}\",\"to\":\"{}\",\"body\":\"{}\"{}}}",
        escape_json(message.from.as_str()),
        escape_json(message.to.as_str()),
        escape_json(message.body.as_str()),
        channel_segment,
    )
}

pub(crate) fn recipient_mailbox_channel_id(recipient: &AgentDid) -> String {
    format!("recipient:{}", recipient.as_str())
}

pub(crate) fn service_message_to_record(
    delivery: ServiceMessageDelivery,
    message_id: MessageId,
) -> Result<MessageRecord, SdkError> {
    let sender = AgentDid::parse(&delivery.sender_did)
        .map_err(|_| SdkError::TransportFailure("service returned invalid sender did"))?;
    let recipient = AgentDid::parse(&delivery.recipient_did)
        .map_err(|_| SdkError::TransportFailure("service returned invalid recipient did"))?;
    Ok(MessageRecord {
        id: message_id,
        message: Message {
            from: sender,
            to: recipient,
            body: delivery.body,
            channel: None,
        },
    })
}

pub(crate) fn agent_profile_to_document(
    profile: ServiceAgentProfile,
    endpoint: &str,
) -> Result<DidDocument, SdkError> {
    let resolved_did = crate::AgentDid::parse(&profile.did).map_err(|_| {
        SdkError::TransportFailure("service returned invalid did in agent profile response")
    })?;
    Ok(DidDocument {
        id: resolved_did,
        metadata: AgentMetadata {
            agent_type: "service-agent".to_owned(),
            model_family: "service-api".to_owned(),
            capabilities: vec!["profile:read".to_owned()],
        },
        service_endpoint: endpoint.to_owned(),
    })
}

pub(crate) fn agent_profile_to_reputation(
    profile: ServiceAgentProfile,
) -> Result<AgentReputation, SdkError> {
    let profile_did = crate::AgentDid::parse(&profile.did).map_err(|_| {
        SdkError::TransportFailure("service returned invalid did in agent profile response")
    })?;
    let score = u32::try_from(profile.reputation_score).map_err(|_| {
        SdkError::TransportFailure("service returned reputation score outside u32 range")
    })?;
    Ok(AgentReputation {
        did: profile_did,
        score,
    })
}
