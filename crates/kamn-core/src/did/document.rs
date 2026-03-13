use std::fmt;

use super::{AgentDid, AgentDidMetadata, DidDocument, DidService, DidVerificationMethod};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DidDocumentError {
    EmptyPublicKey,
    EmptyAgentType,
    EmptyModelFamily,
    MissingCapabilities,
    InvalidCapability,
    InvalidServiceEndpoint(String),
    InvalidVerificationMethodAlgorithm(String),
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
            Self::InvalidVerificationMethodAlgorithm(message) => {
                write!(f, "invalid verification method algorithm: {message}")
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

pub fn validate_did_verification_method_algorithms(
    algorithms: &[String],
) -> Result<(), DidDocumentError> {
    if algorithms.is_empty() {
        return Err(DidDocumentError::InvalidVerificationMethodAlgorithm(
            "at least one verification method algorithm is required".to_owned(),
        ));
    }
    let mut normalized_algorithms: Vec<String> = Vec::with_capacity(algorithms.len());
    for algorithm in algorithms {
        let normalized = algorithm.trim();
        if normalized.is_empty() {
            return Err(DidDocumentError::InvalidVerificationMethodAlgorithm(
                "verification method algorithm entries must not be empty".to_owned(),
            ));
        }
        if normalized != "Multikey" && normalized != "MultikeyV2" {
            return Err(DidDocumentError::InvalidVerificationMethodAlgorithm(format!(
                "unsupported verification method algorithm: {normalized}"
            )));
        }
        normalized_algorithms.push(normalized.to_owned());
    }
    let first_algorithm = &normalized_algorithms[0];
    if normalized_algorithms.iter().any(|algorithm| algorithm != first_algorithm) {
        return Err(DidDocumentError::InvalidVerificationMethodAlgorithm(
            "mixed verification method algorithms are not allowed".to_owned(),
        ));
    }
    Ok(())
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
    if metadata.capabilities.iter().any(|capability| capability.trim().is_empty()) {
        return Err(DidDocumentError::InvalidCapability);
    }
    let key_id = format!("{}#keys-1", did.as_str());
    let service_id = format!("{}#messaging", did.as_str());
    let service_endpoint =
        canonical_service_endpoint(&format!("kamn://messaging/{}", did.method_specific_id()))?;
    validate_did_verification_method_algorithms(&["Multikey".to_owned()])?;
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
