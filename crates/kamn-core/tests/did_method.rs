use kamn_core::{
    canonical_did_document, AgentDid, AgentDidError, AgentDidMetadata, DidDocumentError,
};

fn metadata() -> AgentDidMetadata {
    AgentDidMetadata {
        agent_type: "autonomous".to_owned(),
        model_family: "claude-4".to_owned(),
        capabilities: vec!["text".to_owned(), "code".to_owned()],
        operator: Some("kamn:did:human:operator_1".to_owned()),
    }
}

#[test]
fn parses_valid_kamn_agent_did() {
    let did = AgentDid::parse("kamn:did:agent:agent-1").expect("valid did should parse");
    assert_eq!(did.as_str(), "kamn:did:agent:agent-1");
}

#[test]
fn rejects_invalid_did_prefix() {
    assert_eq!(
        AgentDid::parse("did:example:123"),
        Err(AgentDidError::InvalidPrefix("did:example:123".to_owned()))
    );
}

#[test]
fn rejects_missing_method_specific_id() {
    assert_eq!(
        AgentDid::parse("kamn:did:agent:"),
        Err(AgentDidError::MissingMethodSpecificId)
    );
}

#[test]
fn canonical_document_contains_expected_context_and_service_endpoint() {
    let did = AgentDid::parse("kamn:did:agent:agent-77").expect("valid did should parse");
    let document = canonical_did_document(&did, "z6Mkey", metadata())
        .expect("canonical did document should build");

    assert_eq!(
        document.context,
        vec![
            "https://www.w3.org/ns/did/v1.1".to_owned(),
            "https://kamn.network/context/v1".to_owned()
        ]
    );
    assert_eq!(document.id, did.as_str());
    assert_eq!(document.controller, did.as_str());
    assert_eq!(
        document.service[0].service_endpoint,
        "kamn://messaging/agent-77".to_owned()
    );
}

#[test]
fn canonical_document_rejects_empty_public_key() {
    let did = AgentDid::parse("kamn:did:agent:agent-88").expect("valid did should parse");
    assert_eq!(
        canonical_did_document(&did, "", metadata()),
        Err(DidDocumentError::EmptyPublicKey)
    );
}

#[test]
fn canonical_document_rejects_empty_capability_entry() {
    let did = AgentDid::parse("kamn:did:agent:agent-99").expect("valid did should parse");
    let mut invalid = metadata();
    invalid.capabilities.push(String::new());
    // Regression: #109
    assert_eq!(
        canonical_did_document(&did, "z6Mkey", invalid),
        Err(DidDocumentError::InvalidCapability)
    );
}
