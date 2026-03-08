use super::task_escrow::escape_json;
use crate::channel_create::payload as channel_create_json_payload;
use crate::service::ServiceMessageDelivery;
use crate::{
    AgentDid, AgentMetadata, AgentReputation, AgentSummary, DidDocument, Message, MessageId,
    MessageRecord, SdkError, ServiceAgentProfile,
};

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

pub(crate) fn agent_registration_payload(metadata: &AgentMetadata) -> Result<String, SdkError> {
    validate_agent_metadata(metadata)?;
    Ok(serde_json::json!({
        "agent_type": metadata.agent_type,
        "model_family": metadata.model_family,
        "capabilities": metadata.capabilities,
    })
    .to_string())
}

pub(crate) fn channel_create_payload(name: &str) -> Result<String, SdkError> {
    channel_create_json_payload(name)
}

pub(crate) fn recipient_mailbox_channel_id(recipient: &AgentDid) -> String {
    format!("recipient:{}", recipient.as_str())
}

pub(crate) fn service_message_to_record(
    delivery: ServiceMessageDelivery,
    message_id: MessageId,
) -> Result<MessageRecord, SdkError> {
    let sender =
        parse_service_agent_did(&delivery.sender_did, "service returned invalid sender did")?;
    let recipient = parse_service_agent_did(
        &delivery.recipient_did,
        "service returned invalid recipient did",
    )?;
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
    let resolved_did = parse_service_agent_did(
        &profile.did,
        "service returned invalid did in agent profile response",
    )?;
    Ok(DidDocument {
        id: resolved_did,
        metadata: AgentMetadata {
            agent_type: profile.agent_type,
            model_family: profile.model_family,
            capabilities: profile.capabilities,
        },
        service_endpoint: endpoint.to_owned(),
    })
}

pub(crate) fn agent_profile_to_reputation(
    profile: ServiceAgentProfile,
) -> Result<AgentReputation, SdkError> {
    let profile_did = parse_service_agent_did(
        &profile.did,
        "service returned invalid did in agent profile response",
    )?;
    let score = u32::try_from(profile.reputation_score).map_err(|_| {
        SdkError::TransportFailure("service returned reputation score outside u32 range")
    })?;
    Ok(AgentReputation {
        did: profile_did,
        score,
    })
}

pub(crate) fn agent_profile_to_summary(
    profile: ServiceAgentProfile,
) -> Result<AgentSummary, SdkError> {
    let did = parse_service_agent_did(
        &profile.did,
        "service returned invalid did in agent profile response",
    )?;
    Ok(AgentSummary {
        did,
        agent_type: profile.agent_type,
        model_family: profile.model_family,
        capabilities: profile.capabilities,
    })
}

fn parse_service_agent_did(raw: &str, error_message: &'static str) -> Result<AgentDid, SdkError> {
    AgentDid::parse(raw).map_err(|_| SdkError::TransportFailure(error_message))
}

fn validate_agent_metadata(metadata: &AgentMetadata) -> Result<(), SdkError> {
    if metadata.agent_type.trim().is_empty() {
        return Err(SdkError::InvalidInput {
            field: "agent_type",
            reason: "must not be empty",
        });
    }
    if metadata.model_family.trim().is_empty() {
        return Err(SdkError::InvalidInput {
            field: "model_family",
            reason: "must not be empty",
        });
    }
    if metadata.capabilities.is_empty() {
        return Err(SdkError::InvalidInput {
            field: "capabilities",
            reason: "must include at least one capability",
        });
    }
    if metadata
        .capabilities
        .iter()
        .any(|value| value.trim().is_empty())
    {
        return Err(SdkError::InvalidInput {
            field: "capabilities",
            reason: "must not include empty capability entries",
        });
    }
    Ok(())
}
