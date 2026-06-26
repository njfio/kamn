use std::fmt;

use super::{AgentDid, AgentDidMetadata, DidDocument, DidService, DidVerificationMethod};

mod support;

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
    support::canonical_service_endpoint(raw_endpoint)
}

pub fn validate_did_verification_method_algorithms(
    algorithms: &[String],
) -> Result<(), DidDocumentError> {
    support::validate_did_verification_method_algorithms(algorithms)
}

pub fn canonical_did_document(
    did: &AgentDid,
    public_key_multibase: &str,
    metadata: AgentDidMetadata,
) -> Result<DidDocument, DidDocumentError> {
    support::canonical_did_document(did, public_key_multibase, metadata)
}

pub(crate) fn build_did_document(
    did: &AgentDid,
    public_key_multibase: &str,
    metadata: AgentDidMetadata,
    service_endpoint: String,
) -> DidDocument {
    let key_id = format!("{}#keys-1", did.as_str());
    DidDocument {
        context: did_context(),
        id: did.as_str().to_owned(),
        controller: did.as_str().to_owned(),
        verification_method: build_verification_method(did, &key_id, public_key_multibase),
        authentication: vec![key_id.clone()],
        assertion_method: vec![key_id],
        service: build_service(did, service_endpoint),
        metadata,
    }
}

fn did_context() -> Vec<String> {
    vec![
        "https://www.w3.org/ns/did/v1.1".to_owned(),
        "https://kamn.network/context/v1".to_owned(),
    ]
}

fn build_verification_method(
    did: &AgentDid,
    key_id: &str,
    public_key_multibase: &str,
) -> Vec<DidVerificationMethod> {
    vec![DidVerificationMethod {
        id: key_id.to_owned(),
        type_name: "Multikey".to_owned(),
        controller: did.as_str().to_owned(),
        public_key_multibase: public_key_multibase.to_owned(),
    }]
}

fn build_service(did: &AgentDid, service_endpoint: String) -> Vec<DidService> {
    vec![DidService {
        id: format!("{}#messaging", did.as_str()),
        type_name: "KAMNMessaging".to_owned(),
        service_endpoint,
    }]
}
