use kamn_types::did;
use kamn_types::{
    parse_agent_did_canonical, parse_kamn_did_canonical, AgentDid, AgentDidError,
    AgentDidKeyBindingError, AgentDidMetadata, DidDocument, DidService, DidVerificationMethod,
    KamnDidError, SharedDidParseError,
};

const PUBLIC_KEY_HEX: &str =
    "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

#[test]
fn integration_top_level_and_module_key_binding_generation_match() {
    let top_level =
        AgentDid::with_public_key_hex_binding("boundary_agent", PUBLIC_KEY_HEX).expect("top-level");
    let module =
        did::AgentDid::with_public_key_hex_binding("boundary_agent", PUBLIC_KEY_HEX).expect("did");

    assert_eq!(top_level, module);
    assert_eq!(top_level.as_str(), module.as_str());
}

#[test]
fn integration_generated_agent_did_exposes_and_verifies_key_binding_fingerprint() {
    let generated =
        AgentDid::with_public_key_hex_binding("boundary_agent", PUBLIC_KEY_HEX).expect("generate");

    let fingerprint = generated
        .key_binding_fingerprint()
        .expect("generated did should carry fingerprint");
    assert_eq!(fingerprint.len(), 32);
    assert!(fingerprint.chars().all(|ch| ch.is_ascii_hexdigit()));
    generated
        .ensure_public_key_hex_binding(PUBLIC_KEY_HEX)
        .expect("original public key should validate");
}

#[test]
fn integration_invalid_public_key_hex_surfaces_shared_boundary_error() {
    assert_eq!(
        AgentDid::with_public_key_hex_binding("boundary_agent", "zz"),
        Err(AgentDidKeyBindingError::InvalidPublicKeyHex)
    );
    assert_eq!(
        did::AgentDid::with_public_key_hex_binding("boundary_agent", "zz"),
        Err(AgentDidKeyBindingError::InvalidPublicKeyHex)
    );
}

#[test]
fn integration_canonical_parse_helpers_preserve_missing_id_and_invalid_shape_errors() {
    assert_eq!(
        parse_agent_did_canonical("kamn:did:agent:"),
        Err(SharedDidParseError::Agent(
            AgentDidError::MissingMethodSpecificId
        ))
    );
    assert_eq!(
        parse_kamn_did_canonical("kamn:did:operator:"),
        Err(SharedDidParseError::Kamn(KamnDidError::InvalidShape(
            "kamn:did:operator:".to_owned()
        )))
    );
}

#[test]
fn integration_shared_did_boundary_types_are_constructible_via_kamn_types() {
    let metadata = AgentDidMetadata {
        agent_type: "autonomous".to_owned(),
        model_family: "boundary-v1".to_owned(),
        capabilities: vec!["route".to_owned(), "sign".to_owned()],
        operator: Some("kamn:did:operator:node-1".to_owned()),
    };
    let verification_method = DidVerificationMethod {
        id: "kamn:did:agent:boundary#key-1".to_owned(),
        type_name: "Multikey".to_owned(),
        controller: "kamn:did:agent:boundary".to_owned(),
        public_key_multibase: "z6MkwBoundaryKey".to_owned(),
    };
    let service = DidService {
        id: "kamn:did:agent:boundary#service-1".to_owned(),
        type_name: "KamnApi".to_owned(),
        service_endpoint: "https://example.invalid/agents/boundary".to_owned(),
    };
    let document = DidDocument {
        context: vec!["https://www.w3.org/ns/did/v1".to_owned()],
        id: "kamn:did:agent:boundary".to_owned(),
        controller: "kamn:did:agent:boundary".to_owned(),
        verification_method: vec![verification_method.clone()],
        authentication: vec![verification_method.id.clone()],
        assertion_method: vec![verification_method.id.clone()],
        service: vec![service.clone()],
        metadata: metadata.clone(),
    };

    assert_eq!(document.metadata, metadata);
    assert_eq!(document.verification_method, vec![verification_method]);
    assert_eq!(document.service, vec![service]);
}
