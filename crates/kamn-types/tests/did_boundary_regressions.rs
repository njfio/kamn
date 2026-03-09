use kamn_types::did;
use kamn_types::{
    parse_agent_did_canonical, parse_kamn_did_canonical, AgentDid, AgentDidKeyBindingError,
    AgentDidError, KamnDidError, SharedDidParseError,
};

const PUBLIC_KEY_HEX: &str = "025f6ceceac37540cf6ef5f09d4f62c05f0b8f57fe6d8ae32a8f13f4a2eb6e940d";
const PUBLIC_KEY_HEX_ALT: &str =
    "02dbf4fcb77ef6a9f2d0f5f0d7c7faaf02f53b724f4cfe6fe1d95ff5a6d4bf8132";

#[test]
fn integration_plain_agent_did_reports_missing_key_binding() {
    let did = AgentDid::parse("kamn:did:agent:plain-boundary").expect("plain agent did should parse");
    assert_eq!(
        did.ensure_public_key_hex_binding(PUBLIC_KEY_HEX),
        Err(AgentDidKeyBindingError::MissingKeyBinding)
    );
}

#[test]
fn integration_bound_agent_did_rejects_mismatched_public_key_with_typed_error() {
    let did = did::AgentDid::with_public_key_hex_binding("typed-boundary", PUBLIC_KEY_HEX)
        .expect("bound did should render");

    let error = did
        .ensure_public_key_hex_binding(PUBLIC_KEY_HEX_ALT)
        .expect_err("mismatched public key should fail");

    let AgentDidKeyBindingError::KeyBindingMismatch { expected, actual } = error else {
        panic!("expected key binding mismatch error");
    };
    assert_eq!(expected.len(), 32);
    assert_eq!(actual.len(), 32);
}

#[test]
fn integration_parse_helpers_preserve_displayable_typed_errors() {
    let empty = parse_agent_did_canonical(" ").expect_err("empty input should fail");
    assert_eq!(empty, SharedDidParseError::EmptyInput);
    assert_eq!(empty.to_string(), "did input must not be empty");

    let agent = parse_agent_did_canonical("did:example:agent").expect_err("invalid agent did");
    assert_eq!(
        agent,
        SharedDidParseError::Agent(AgentDidError::InvalidPrefix("did:example:agent".to_owned()))
    );
    assert!(agent.to_string().contains("agent did parse failed"));

    let kamn = parse_kamn_did_canonical("did:example:operator").expect_err("invalid kamn did");
    assert_eq!(
        kamn,
        SharedDidParseError::Kamn(KamnDidError::InvalidPrefix(
            "did:example:operator".to_owned()
        ))
    );
    assert!(kamn.to_string().contains("kamn did parse failed"));
}

#[test]
fn integration_bound_agent_did_round_trips_after_parse() {
    let rendered = AgentDid::with_public_key_hex_binding("roundtrip-boundary", PUBLIC_KEY_HEX)
        .expect("bound did should render")
        .to_string();

    let parsed = AgentDid::parse(rendered.as_str()).expect("rendered bound did should parse");
    assert_eq!(parsed.as_str(), rendered);
    parsed
        .ensure_public_key_hex_binding(PUBLIC_KEY_HEX)
        .expect("parsed bound did should preserve key binding");
}
