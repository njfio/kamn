use std::fmt;

const AGENT_DID_PREFIX: &str = "kamn:did:agent:";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AgentDid(String);

impl AgentDid {
    pub fn parse(value: &str) -> Result<Self, AgentDidError> {
        if !value.starts_with(AGENT_DID_PREFIX) {
            return Err(AgentDidError::InvalidPrefix(value.to_owned()));
        }
        let method_specific_id = &value[AGENT_DID_PREFIX.len()..];
        if method_specific_id.is_empty() {
            return Err(AgentDidError::MissingMethodSpecificId);
        }
        if !method_specific_id
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_')
        {
            return Err(AgentDidError::InvalidCharacter(
                method_specific_id.to_owned(),
            ));
        }

        Ok(Self(value.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn method_specific_id(&self) -> &str {
        &self.0[AGENT_DID_PREFIX.len()..]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentDidError {
    InvalidPrefix(String),
    MissingMethodSpecificId,
    InvalidCharacter(String),
}

impl fmt::Display for AgentDidError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPrefix(value) => write!(f, "invalid agent did prefix: {value}"),
            Self::MissingMethodSpecificId => {
                write!(f, "agent did method-specific id must not be empty")
            }
            Self::InvalidCharacter(value) => {
                write!(f, "agent did has invalid characters: {value}")
            }
        }
    }
}

impl std::error::Error for AgentDidError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentDidMetadata {
    pub agent_type: String,
    pub model_family: String,
    pub capabilities: Vec<String>,
    pub operator: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DidVerificationMethod {
    pub id: String,
    pub type_name: String,
    pub controller: String,
    pub public_key_multibase: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DidService {
    pub id: String,
    pub type_name: String,
    pub service_endpoint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DidDocument {
    pub context: Vec<String>,
    pub id: String,
    pub controller: String,
    pub verification_method: Vec<DidVerificationMethod>,
    pub authentication: Vec<String>,
    pub assertion_method: Vec<String>,
    pub service: Vec<DidService>,
    pub metadata: AgentDidMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DidDocumentError {
    EmptyPublicKey,
    EmptyAgentType,
    EmptyModelFamily,
    MissingCapabilities,
    InvalidCapability,
    InvalidServiceEndpoint(String),
}

impl fmt::Display for DidDocumentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPublicKey => write!(f, "public key must not be empty"),
            Self::EmptyAgentType => write!(f, "agent_type must not be empty"),
            Self::EmptyModelFamily => write!(f, "model_family must not be empty"),
            Self::MissingCapabilities => write!(f, "at least one capability is required"),
            Self::InvalidCapability => write!(f, "capability entries must not be empty"),
            Self::InvalidServiceEndpoint(message) => {
                write!(f, "invalid service endpoint: {message}")
            }
        }
    }
}

impl std::error::Error for DidDocumentError {}

pub fn canonical_service_endpoint(raw_endpoint: &str) -> Result<String, DidDocumentError> {
    let trimmed = raw_endpoint.trim();
    if trimmed.is_empty() {
        return Err(DidDocumentError::InvalidServiceEndpoint(
            "service endpoint must not be empty".to_owned(),
        ));
    }
    if trimmed.contains('?') || trimmed.contains('#') {
        return Err(DidDocumentError::InvalidServiceEndpoint(
            "service endpoint must not include query or fragment".to_owned(),
        ));
    }

    let (scheme, remainder) = trimmed.split_once("://").ok_or_else(|| {
        DidDocumentError::InvalidServiceEndpoint(
            "service endpoint must include scheme://authority/path".to_owned(),
        )
    })?;
    if !scheme.eq_ignore_ascii_case("kamn") {
        return Err(DidDocumentError::InvalidServiceEndpoint(
            "service endpoint scheme must be kamn".to_owned(),
        ));
    }

    let (authority, path) = remainder.split_once('/').ok_or_else(|| {
        DidDocumentError::InvalidServiceEndpoint(
            "service endpoint must include authority and path".to_owned(),
        )
    })?;
    if !authority.eq_ignore_ascii_case("messaging") {
        return Err(DidDocumentError::InvalidServiceEndpoint(
            "service endpoint authority must be messaging".to_owned(),
        ));
    }
    if path.is_empty() || path.contains('/') {
        return Err(DidDocumentError::InvalidServiceEndpoint(
            "service endpoint path must be a single segment".to_owned(),
        ));
    }

    let normalized_path = path.to_ascii_lowercase();
    if !normalized_path
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_')
    {
        return Err(DidDocumentError::InvalidServiceEndpoint(
            "service endpoint path contains invalid characters".to_owned(),
        ));
    }

    Ok(format!("kamn://messaging/{normalized_path}"))
}

pub fn canonical_did_document(
    did: &AgentDid,
    public_key_multibase: &str,
    metadata: AgentDidMetadata,
) -> Result<DidDocument, DidDocumentError> {
    if public_key_multibase.trim().is_empty() {
        return Err(DidDocumentError::EmptyPublicKey);
    }
    if metadata.agent_type.trim().is_empty() {
        return Err(DidDocumentError::EmptyAgentType);
    }
    if metadata.model_family.trim().is_empty() {
        return Err(DidDocumentError::EmptyModelFamily);
    }
    if metadata.capabilities.is_empty() {
        return Err(DidDocumentError::MissingCapabilities);
    }
    if metadata
        .capabilities
        .iter()
        .any(|capability| capability.trim().is_empty())
    {
        return Err(DidDocumentError::InvalidCapability);
    }

    let key_id = format!("{}#keys-1", did.as_str());
    let service_id = format!("{}#messaging", did.as_str());
    let service_endpoint =
        canonical_service_endpoint(&format!("kamn://messaging/{}", did.method_specific_id()))?;

    Ok(DidDocument {
        context: vec![
            "https://www.w3.org/ns/did/v1.1".to_owned(),
            "https://kamn.network/context/v1".to_owned(),
        ],
        id: did.as_str().to_owned(),
        controller: did.as_str().to_owned(),
        verification_method: vec![DidVerificationMethod {
            id: key_id.clone(),
            type_name: "Multikey".to_owned(),
            controller: did.as_str().to_owned(),
            public_key_multibase: public_key_multibase.to_owned(),
        }],
        authentication: vec![key_id.clone()],
        assertion_method: vec![key_id],
        service: vec![DidService {
            id: service_id,
            type_name: "KAMNMessaging".to_owned(),
            service_endpoint,
        }],
        metadata,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        canonical_did_document, canonical_service_endpoint, AgentDid, AgentDidError,
        AgentDidMetadata, DidDocumentError,
    };

    fn metadata() -> AgentDidMetadata {
        AgentDidMetadata {
            agent_type: "autonomous".to_owned(),
            model_family: "claude-4".to_owned(),
            capabilities: vec!["text".to_owned()],
            operator: None,
        }
    }

    #[test]
    fn parse_rejects_invalid_characters() {
        assert_eq!(
            AgentDid::parse("kamn:did:agent:Agent_1"),
            Err(AgentDidError::InvalidCharacter("Agent_1".to_owned()))
        );
    }

    #[test]
    fn canonical_document_requires_capabilities() {
        let did = match AgentDid::parse("kamn:did:agent:agent-1") {
            Ok(value) => value,
            Err(error) => panic!("did parse failed: {error}"),
        };
        let mut invalid = metadata();
        invalid.capabilities.clear();
        assert_eq!(
            canonical_did_document(&did, "z6Mkey", invalid),
            Err(DidDocumentError::MissingCapabilities)
        );
    }

    #[test]
    fn canonical_service_endpoint_normalizes_scheme_authority_and_path() {
        assert_eq!(
            canonical_service_endpoint("  KAMN://MESSAGING/Agent_1  "),
            Ok("kamn://messaging/agent_1".to_owned())
        );
    }

    #[test]
    fn canonical_service_endpoint_rejects_query_and_fragment() {
        assert_eq!(
            canonical_service_endpoint("kamn://messaging/agent-1?channel=dm"),
            Err(DidDocumentError::InvalidServiceEndpoint(
                "service endpoint must not include query or fragment".to_owned()
            ))
        );
    }
}
