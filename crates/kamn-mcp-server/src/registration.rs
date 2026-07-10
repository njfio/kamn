use kamn_agent_lib::{AgentLibError, KamnAgentHandle};
use kamn_sdk::AgentMetadata;

pub(crate) fn register_service_backed_mcp_agent(
    handle: &KamnAgentHandle,
) -> Result<String, AgentLibError> {
    let profile = handle.register_agent(&mcp_agent_metadata())?;
    Ok(serde_json::json!({
        "did": profile.did,
        "reputation_score": profile.reputation_score,
        "agent_type": profile.agent_type,
        "model_family": profile.model_family,
        "capabilities": profile.capabilities,
    })
    .to_string())
}

fn mcp_agent_metadata() -> AgentMetadata {
    AgentMetadata {
        agent_type: "autonomous".to_owned(),
        model_family: "mcp-agent".to_owned(),
        capabilities: vec!["mcp".to_owned()],
    }
}
