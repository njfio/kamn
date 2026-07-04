use super::{
    canonical_did_document, canonical_service_endpoint,
    validate_did_verification_method_algorithms, AgentDid, AgentDidError, AgentDidKeyBindingError,
    AgentDidMetadata, DidDocumentError, KamnDid, KamnDidError,
};

const SOURCE: &str = include_str!("../../../kamn-types/src/did/ids.rs");
const TEST_PUBLIC_KEY_HEX: &str =
    "025f6ceceac37540cf6ef5f09d4f62c05f0b8f57fe6d8ae32a8f13f4a2eb6e940d";
const TEST_PUBLIC_KEY_HEX_ALT: &str =
    "02dbf4fcb77ef6a9f2d0f5f0d7c7faaf02f53b724f4cfe6fe1d95ff5a6d4bf8132";

fn ensure_public_key_hex_binding_source() -> &'static str {
    let function_start = SOURCE
        .find("pub fn ensure_public_key_hex_binding(")
        .expect("function must exist");
    let function_end = SOURCE[function_start..]
        .find("\n    /// Builds an agent DID with deterministic key-binding fingerprint suffix.")
        .map(|offset| function_start + offset)
        .expect("function boundary must exist");
    &SOURCE[function_start..function_end]
}

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
fn parse_kamn_did_accepts_owner_and_agent_dids() {
    let owner = KamnDid::parse("kamn:did:owner:sender-1").expect("owner did should parse");
    let agent = KamnDid::parse("kamn:did:agent:agent-1").expect("agent did should parse");
    assert_eq!(owner.as_str(), "kamn:did:owner:sender-1");
    assert_eq!(agent.as_str(), "kamn:did:agent:agent-1");
}

#[test]
fn parse_kamn_did_rejects_invalid_prefix_and_shape() {
    assert_eq!(
        KamnDid::parse("did:example:alice"),
        Err(KamnDidError::InvalidPrefix("did:example:alice".to_owned()))
    );
    assert_eq!(
        KamnDid::parse("kamn:did:"),
        Err(KamnDidError::InvalidShape("kamn:did:".to_owned()))
    );
}

#[test]
fn canonical_document_requires_capabilities() {
    let did = AgentDid::parse("kamn:did:agent:agent-1").expect("did parse failed");
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

#[test]
fn validate_did_verification_method_algorithms_accepts_uniform_multikey_set() {
    let algorithms = vec!["Multikey".to_owned(), "Multikey".to_owned()];
    assert_eq!(
        validate_did_verification_method_algorithms(&algorithms),
        Ok(())
    );
}

#[test]
fn validate_did_verification_method_algorithms_rejects_mixed_algorithms() {
    let algorithms = vec!["Multikey".to_owned(), "MultikeyV2".to_owned()];
    assert_eq!(
        validate_did_verification_method_algorithms(&algorithms),
        Err(DidDocumentError::InvalidVerificationMethodAlgorithm(
            "mixed verification method algorithms are not allowed".to_owned()
        ))
    );
}

#[test]
fn unit_agent_did_with_public_key_hex_binding_embeds_fingerprint_suffix() {
    let did = AgentDid::with_public_key_hex_binding("agent-1", TEST_PUBLIC_KEY_HEX)
        .expect("bound did should render");
    assert!(did.as_str().starts_with("kamn:did:agent:agent-1--keyh-"));
    assert_eq!(
        did.key_binding_fingerprint().expect("fingerprint").len(),
        32
    );
}

#[test]
fn regression_agent_did_key_binding_verification_rejects_missing_binding() {
    let did = AgentDid::parse("kamn:did:agent:agent-1").expect("did should parse");
    assert_eq!(
        did.ensure_public_key_hex_binding(TEST_PUBLIC_KEY_HEX),
        Err(AgentDidKeyBindingError::MissingKeyBinding)
    );
}

#[test]
fn regression_agent_did_key_binding_verification_rejects_mismatched_public_key() {
    let did = AgentDid::with_public_key_hex_binding("agent-2", TEST_PUBLIC_KEY_HEX)
        .expect("bound did should render");
    let error = did
        .ensure_public_key_hex_binding(TEST_PUBLIC_KEY_HEX_ALT)
        .expect_err("mismatched public key should fail binding verification");
    let AgentDidKeyBindingError::KeyBindingMismatch { expected, actual } = error else {
        panic!("expected key binding mismatch error");
    };
    assert!(expected.len() == 32 && actual.len() == 32);
}

#[test]
fn regression_agent_did_key_binding_verification_accepts_matching_public_key() {
    let did = AgentDid::with_public_key_hex_binding("agent-3", TEST_PUBLIC_KEY_HEX)
        .expect("bound did should render");
    did.ensure_public_key_hex_binding(TEST_PUBLIC_KEY_HEX)
        .expect("matching public key should satisfy did binding");
}

#[test]
fn regression_agent_did_key_binding_verification_accepts_parsed_bound_did() {
    let rendered = AgentDid::with_public_key_hex_binding("agent-5", TEST_PUBLIC_KEY_HEX)
        .expect("bound did should render")
        .to_string();
    let parsed = AgentDid::parse(rendered.as_str()).expect("rendered bound did should parse");
    parsed
        .ensure_public_key_hex_binding(TEST_PUBLIC_KEY_HEX)
        .expect("parsed bound did should preserve key-binding verification");
}

#[test]
fn regression_agent_did_key_binding_verification_rejects_malformed_public_key_hex() {
    let did = AgentDid::with_public_key_hex_binding("agent-4", TEST_PUBLIC_KEY_HEX)
        .expect("bound did should render");
    assert_eq!(
        did.ensure_public_key_hex_binding("zz-not-hex"),
        Err(AgentDidKeyBindingError::InvalidPublicKeyHex)
    );
}

#[test]
fn regression_requires_constant_time_agent_did_key_binding_compare() {
    let function_source = ensure_public_key_hex_binding_source();
    let direct_pattern = ["if actual", "!=", " expected {"].concat();
    assert!(function_source.contains("constant_time_eq_bytes("));
    assert!(!function_source.contains(direct_pattern.as_str()));
}
