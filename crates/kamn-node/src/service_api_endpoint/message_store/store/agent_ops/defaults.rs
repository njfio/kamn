use super::super::super::*;

pub(super) fn default_agent_record(agent_did: &str) -> ServiceApiPersistedAgentRecord {
    ServiceApiPersistedAgentRecord {
        did: agent_did.to_owned(),
        reputation_score: INITIAL_SERVICE_API_AGENT_REPUTATION_SCORE,
        balance: Some(INITIAL_SERVICE_API_AGENT_BALANCE),
        registered: false,
        agent_type: default_agent_type(),
        model_family: default_model_family(),
        capabilities: default_capabilities(),
    }
}

pub(super) fn agent_profile_body(
    record: &ServiceApiPersistedAgentRecord,
) -> ServiceApiAgentGetBody {
    ServiceApiAgentGetBody {
        did: record.did.clone(),
        reputation_score: record.reputation_score,
        agent_type: record_agent_type(record),
        model_family: record_model_family(record),
        capabilities: record_capabilities(record),
    }
}

pub(super) fn record_agent_type(record: &ServiceApiPersistedAgentRecord) -> String {
    if record.agent_type.trim().is_empty() {
        return default_agent_type();
    }
    record.agent_type.clone()
}

pub(super) fn record_model_family(record: &ServiceApiPersistedAgentRecord) -> String {
    if record.model_family.trim().is_empty() {
        return default_model_family();
    }
    record.model_family.clone()
}

pub(super) fn record_capabilities(record: &ServiceApiPersistedAgentRecord) -> Vec<String> {
    if record.capabilities.is_empty() {
        return default_capabilities();
    }
    record.capabilities.clone()
}

pub(super) fn default_agent_type() -> String {
    "service-agent".to_owned()
}

pub(super) fn default_model_family() -> String {
    "service-api".to_owned()
}

pub(super) fn default_capabilities() -> Vec<String> {
    vec!["profile:read".to_owned()]
}

pub(crate) fn normalize_agent_did(candidate: Option<&str>, fallback: &str) -> String {
    match candidate {
        Some(value) if AgentDid::parse(value).is_ok() => value.to_owned(),
        _ => fallback.to_owned(),
    }
}
