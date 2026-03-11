#[derive(Debug, Clone, PartialEq, Eq)]
/// Metadata extension embedded in KAMN DID documents.
pub struct AgentDidMetadata {
    /// Agent type classification (for example `autonomous`).
    pub agent_type: String,
    /// Model family identifier for the agent.
    pub model_family: String,
    /// Declared capabilities supported by the agent.
    pub capabilities: Vec<String>,
    /// Optional operator DID or name associated with the agent.
    pub operator: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// DID verification method entry.
pub struct DidVerificationMethod {
    /// Verification method identifier.
    pub id: String,
    /// Verification method type name.
    pub type_name: String,
    /// DID controller for this method.
    pub controller: String,
    /// Multibase-encoded public key.
    pub public_key_multibase: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// DID service endpoint entry.
pub struct DidService {
    /// Service identifier.
    pub id: String,
    /// Service type name.
    pub type_name: String,
    /// Canonical service endpoint URI.
    pub service_endpoint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Canonical DID document representation used by KAMN.
pub struct DidDocument {
    /// Ordered JSON-LD context list.
    pub context: Vec<String>,
    /// DID identifier.
    pub id: String,
    /// DID controller identifier.
    pub controller: String,
    /// Verification methods published by the DID.
    pub verification_method: Vec<DidVerificationMethod>,
    /// Authentication method references.
    pub authentication: Vec<String>,
    /// Assertion method references.
    pub assertion_method: Vec<String>,
    /// Service endpoints attached to the DID.
    pub service: Vec<DidService>,
    /// KAMN-specific metadata extension.
    pub metadata: AgentDidMetadata,
}
