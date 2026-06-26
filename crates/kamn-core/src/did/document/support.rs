use super::{build_did_document, AgentDid, AgentDidMetadata, DidDocument, DidDocumentError};

pub(super) fn canonical_service_endpoint(raw_endpoint: &str) -> Result<String, DidDocumentError> {
    let trimmed = require_endpoint(raw_endpoint)?;
    let remainder = require_scheme(trimmed)?;
    let path = require_authority_and_path(remainder)?;
    Ok(format!("kamn://messaging/{}", normalize_path(path)?))
}

fn require_endpoint(raw_endpoint: &str) -> Result<&str, DidDocumentError> {
    let trimmed = raw_endpoint.trim();
    if trimmed.is_empty() {
        return Err(endpoint_error("service endpoint must not be empty"));
    }
    if trimmed.contains('?') || trimmed.contains('#') {
        return Err(endpoint_error(
            "service endpoint must not include query or fragment",
        ));
    }
    Ok(trimmed)
}

fn require_scheme(trimmed: &str) -> Result<&str, DidDocumentError> {
    let (scheme, remainder) = trimmed
        .split_once("://")
        .ok_or_else(|| endpoint_error("service endpoint must include scheme://authority/path"))?;
    if !scheme.eq_ignore_ascii_case("kamn") {
        return Err(endpoint_error("service endpoint scheme must be kamn"));
    }
    Ok(remainder)
}

fn require_authority_and_path(remainder: &str) -> Result<&str, DidDocumentError> {
    let (authority, path) = remainder
        .split_once('/')
        .ok_or_else(|| endpoint_error("service endpoint must include authority and path"))?;
    if !authority.eq_ignore_ascii_case("messaging") {
        return Err(endpoint_error(
            "service endpoint authority must be messaging",
        ));
    }
    if path.is_empty() || path.contains('/') {
        return Err(endpoint_error(
            "service endpoint path must be a single segment",
        ));
    }
    Ok(path)
}

fn normalize_path(path: &str) -> Result<String, DidDocumentError> {
    let normalized = path.to_ascii_lowercase();
    if normalized
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_')
    {
        return Ok(normalized);
    }
    Err(endpoint_error(
        "service endpoint path contains invalid characters",
    ))
}

fn endpoint_error(message: &str) -> DidDocumentError {
    DidDocumentError::InvalidServiceEndpoint(message.to_owned())
}

pub(super) fn validate_did_verification_method_algorithms(
    algorithms: &[String],
) -> Result<(), DidDocumentError> {
    let normalized = normalize_algorithms(algorithms)?;
    if normalized
        .iter()
        .all(|algorithm| algorithm == &normalized[0])
    {
        return Ok(());
    }
    Err(DidDocumentError::InvalidVerificationMethodAlgorithm(
        "mixed verification method algorithms are not allowed".to_owned(),
    ))
}

fn normalize_algorithms(algorithms: &[String]) -> Result<Vec<String>, DidDocumentError> {
    if algorithms.is_empty() {
        return Err(algorithm_error(
            "at least one verification method algorithm is required",
        ));
    }
    algorithms
        .iter()
        .map(|algorithm| normalize_algorithm(algorithm))
        .collect()
}

fn normalize_algorithm(algorithm: &str) -> Result<String, DidDocumentError> {
    let normalized = algorithm.trim();
    if normalized.is_empty() {
        return Err(algorithm_error(
            "verification method algorithm entries must not be empty",
        ));
    }
    if normalized != "Multikey" && normalized != "MultikeyV2" {
        return Err(algorithm_error(&format!(
            "unsupported verification method algorithm: {normalized}"
        )));
    }
    Ok(normalized.to_owned())
}

fn algorithm_error(message: &str) -> DidDocumentError {
    DidDocumentError::InvalidVerificationMethodAlgorithm(message.to_owned())
}

pub(super) fn canonical_did_document(
    did: &AgentDid,
    public_key_multibase: &str,
    metadata: AgentDidMetadata,
) -> Result<DidDocument, DidDocumentError> {
    validate_document_inputs(public_key_multibase, &metadata)?;
    let service_endpoint =
        canonical_service_endpoint(&format!("kamn://messaging/{}", did.method_specific_id()))?;
    validate_did_verification_method_algorithms(&["Multikey".to_owned()])?;
    Ok(build_did_document(
        did,
        public_key_multibase,
        metadata,
        service_endpoint,
    ))
}

fn validate_document_inputs(
    public_key_multibase: &str,
    metadata: &AgentDidMetadata,
) -> Result<(), DidDocumentError> {
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
    Ok(())
}
