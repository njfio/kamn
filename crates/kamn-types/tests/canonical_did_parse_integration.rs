use kamn_types::{
    parse_agent_did_canonical, parse_kamn_did_canonical, AgentDidError, KamnDidError,
    SharedDidParseError,
};

#[test]
fn integration_canonical_agent_did_parse_trims_surrounding_whitespace() {
    let parsed = parse_agent_did_canonical("  kamn:did:agent:alpha_1  ")
        .expect("trimmed agent did should parse");
    assert_eq!(parsed.as_str(), "kamn:did:agent:alpha_1");
}

#[test]
fn integration_canonical_kamn_did_parse_trims_surrounding_whitespace() {
    let parsed = parse_kamn_did_canonical("  kamn:did:operator:node-1  ")
        .expect("trimmed kamn did should parse");
    assert_eq!(parsed.as_str(), "kamn:did:operator:node-1");
}

#[test]
fn integration_canonical_parse_rejects_empty_input() {
    assert_eq!(
        parse_agent_did_canonical(" \n\t "),
        Err(SharedDidParseError::EmptyInput)
    );
    assert_eq!(
        parse_kamn_did_canonical(""),
        Err(SharedDidParseError::EmptyInput)
    );
}

#[test]
fn integration_canonical_parse_preserves_underlying_error_types() {
    assert_eq!(
        parse_agent_did_canonical("did:example:agent"),
        Err(SharedDidParseError::Agent(AgentDidError::InvalidPrefix(
            "did:example:agent".to_owned()
        )))
    );
    assert_eq!(
        parse_kamn_did_canonical("did:example:operator"),
        Err(SharedDidParseError::Kamn(KamnDidError::InvalidPrefix(
            "did:example:operator".to_owned()
        )))
    );
}
