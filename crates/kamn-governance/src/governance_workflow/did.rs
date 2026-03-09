use super::error::GovernanceWorkflowError;

const AGENT_DID_PREFIX: &str = "kamn:did:agent:";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ParsedAgentDid(String);

impl ParsedAgentDid {
    pub(super) fn parse(value: &str) -> Result<Self, String> {
        if !value.starts_with(AGENT_DID_PREFIX) {
            return Err(format!("invalid agent did prefix: {value}"));
        }
        let method_specific_id = &value[AGENT_DID_PREFIX.len()..];
        if method_specific_id.is_empty() {
            return Err("agent did method-specific id must not be empty".to_owned());
        }
        if !method_specific_id
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_')
        {
            return Err(format!(
                "agent did has invalid characters: {method_specific_id}"
            ));
        }
        Ok(Self(value.to_owned()))
    }

    pub(super) fn as_str(&self) -> &str {
        &self.0
    }
}

pub(super) fn parse_agent_did(
    value: &str,
    field: &'static str,
    reason_code: &'static str,
) -> Result<ParsedAgentDid, GovernanceWorkflowError> {
    ParsedAgentDid::parse(value).map_err(|detail| GovernanceWorkflowError::InvalidDid {
        field,
        reason_code,
        detail,
    })
}
