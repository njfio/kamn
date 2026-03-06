use kamn_types::{parse_agent_did_canonical, parse_kamn_did_canonical};

const README: &str = include_str!("../README.md");
const ARCH_DOC: &str = include_str!("../../../docs/architecture/kamn-types.md");
const DID_FORMAT_DOC: &str =
    include_str!("../../../docs/architecture/did-format-standardization.md");

#[test]
fn docs_contain_identity_boundary_and_migration_markers() {
    assert!(README.contains("kamn_types_identity_boundary=did-helpers"));
    assert!(README.contains("kamn_types_primary_module=kamn_types::did"));
    assert!(README.contains("kamn_types_migration_import=use kamn_types::did::AgentDid"));
    assert!(ARCH_DOC.contains("kamn_types_identity_boundary=did-helpers"));
    assert!(ARCH_DOC.contains("kamn_types_import_ownership=explicit"));
    assert!(DID_FORMAT_DOC.contains("did_format_current_canonical=kamn:did:{role}:{id}"));
    assert!(DID_FORMAT_DOC.contains("did_format_divergent_shape=did:kamn:{role}:{id}"));
    assert!(DID_FORMAT_DOC.contains("did_format_target_standard=kamn:did:{role}:{id}"));
    assert!(DID_FORMAT_DOC.contains("did_format_divergent_consumer_count=0"));
    assert!(DID_FORMAT_DOC.contains("did_format_followup_scope=parser-compatibility-decision-only"));
    assert!(DID_FORMAT_DOC.contains("did_format_public_contract_gate=approval-required"));
    assert!(!DID_FORMAT_DOC.contains("did_format_divergent_consumer=docs/"));
}

#[test]
fn did_module_parse_helpers_preserve_top_level_compatibility() {
    let top_agent = parse_agent_did_canonical("  kamn:did:agent:identity-1  ")
        .expect("top-level helper should parse canonical agent did");
    let module_agent = kamn_types::did::parse_agent_did_canonical("  kamn:did:agent:identity-1  ")
        .expect("did module helper should parse canonical agent did");
    assert_eq!(top_agent, module_agent);

    let top_kamn = parse_kamn_did_canonical("  kamn:did:operator:identity-2  ")
        .expect("top-level helper should parse canonical kamn did");
    let module_kamn = kamn_types::did::parse_kamn_did_canonical("  kamn:did:operator:identity-2  ")
        .expect("did module helper should parse canonical kamn did");
    assert_eq!(top_kamn, module_kamn);
}

#[test]
fn did_module_exposes_canonical_did_types() {
    let agent = kamn_types::did::AgentDid::parse("kamn:did:agent:identity-module")
        .expect("did module should expose AgentDid");
    assert_eq!(agent.as_str(), "kamn:did:agent:identity-module");

    let did = kamn_types::did::KamnDid::parse("kamn:did:operator:identity-module")
        .expect("did module should expose KamnDid");
    assert_eq!(did.as_str(), "kamn:did:operator:identity-module");
}
