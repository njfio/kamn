use crate::operator_binding::constants::{
    AGENT_DID_PREFIX, HUMAN_DID_PREFIX, OPERATOR_BINDING_INVALID_AGENT_DID_REASON_CODE,
    OPERATOR_BINDING_INVALID_OPERATOR_DID_REASON_CODE,
};
use crate::operator_binding::OperatorBindingError;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct ParsedAgentDid(String);

impl ParsedAgentDid {
    fn parse(value: &str) -> Result<Self, String> {
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct OperatorHumanDid(String);

impl OperatorHumanDid {
    fn parse(
        value: &str,
        field: &'static str,
        reason_code: &'static str,
    ) -> Result<Self, OperatorBindingError> {
        if !value.starts_with(HUMAN_DID_PREFIX) {
            return Err(OperatorBindingError::InvalidOperatorDid {
                field,
                reason_code,
                detail: format!("invalid human did prefix: {value}"),
            });
        }
        let method_specific_id = &value[HUMAN_DID_PREFIX.len()..];
        if method_specific_id.is_empty() {
            return Err(OperatorBindingError::InvalidOperatorDid {
                field,
                reason_code,
                detail: "human did method-specific id must not be empty".to_owned(),
            });
        }
        if !method_specific_id
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_')
        {
            return Err(OperatorBindingError::InvalidOperatorDid {
                field,
                reason_code,
                detail: format!("human did has invalid characters: {method_specific_id}"),
            });
        }
        Ok(Self(value.to_owned()))
    }

    pub(super) fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct OperatorBindingPrincipals {
    pub(super) agent_did: ParsedAgentDid,
    pub(super) operator_did: OperatorHumanDid,
}

impl OperatorBindingPrincipals {
    pub(super) fn parse(agent_did: &str, operator_did: &str) -> Result<Self, OperatorBindingError> {
        Ok(Self {
            agent_did: parse_agent_did(
                agent_did,
                "agent_did",
                OPERATOR_BINDING_INVALID_AGENT_DID_REASON_CODE,
            )?,
            operator_did: parse_operator_did(
                operator_did,
                "operator_did",
                OPERATOR_BINDING_INVALID_OPERATOR_DID_REASON_CODE,
            )?,
        })
    }
}

fn parse_agent_did(
    value: &str,
    field: &'static str,
    reason_code: &'static str,
) -> Result<ParsedAgentDid, OperatorBindingError> {
    ParsedAgentDid::parse(value).map_err(|detail| OperatorBindingError::InvalidAgentDid {
        field,
        reason_code,
        detail,
    })
}

fn parse_operator_did(
    value: &str,
    field: &'static str,
    reason_code: &'static str,
) -> Result<OperatorHumanDid, OperatorBindingError> {
    OperatorHumanDid::parse(value, field, reason_code)
}
