use super::support::*;
use kamn_core::DidRegistryError;

#[test]
fn register_and_resolve_round_trip() {
    let mut registry = registry();
    let did = parse_did("kamn:did:agent:agent-1");
    let document = document_for(&did, "claude-4");

    registry
        .register(did.clone(), document)
        .expect("register should succeed");
    let resolved = registry.resolve(&did).expect("resolve should succeed");

    assert_eq!(resolved.id, did.as_str().to_owned());
    assert_eq!(resolved.metadata.model_family, "claude-4");
}

#[test]
fn duplicate_register_is_rejected() {
    let mut registry = registry();
    let did = parse_did("kamn:did:agent:agent-2");
    let document = document_for(&did, "claude-4");

    registry
        .register(did.clone(), document.clone())
        .expect("first register should succeed");
    assert_eq!(
        registry.register(did.clone(), document),
        Err(DidRegistryError::AlreadyRegistered(did.as_str().to_owned()))
    );
}

#[test]
fn update_existing_document_succeeds() {
    let mut registry = registry();
    let did = parse_did("kamn:did:agent:agent-3");
    register_document(&mut registry, &did, "claude-4");

    registry
        .update(did.clone(), document_for(&did, "gpt-5"))
        .expect("update should succeed");
    let resolved = registry.resolve(&did).expect("resolve should succeed");
    assert_eq!(resolved.metadata.model_family, "gpt-5");
}

#[test]
fn update_rejects_unknown_did() {
    let mut registry = registry();
    let did = parse_did("kamn:did:agent:agent-4");
    assert_eq!(
        registry.update(did.clone(), document_for(&did, "claude-4")),
        Err(DidRegistryError::NotFound(did.as_str().to_owned()))
    );
}

#[test]
fn revoke_blocks_resolve() {
    let mut registry = registry();
    let did = parse_did("kamn:did:agent:agent-5");
    register_document(&mut registry, &did, "claude-4");
    registry.revoke(&did).expect("revoke should succeed");

    assert_eq!(
        registry.resolve(&did),
        Err(DidRegistryError::Revoked(did.as_str().to_owned()))
    );
}

#[test]
fn revoked_did_cannot_be_re_registered() {
    let mut registry = registry();
    let did = parse_did("kamn:did:agent:agent-6");
    let document = document_for(&did, "claude-4");
    registry
        .register(did.clone(), document.clone())
        .expect("register should succeed");
    registry.revoke(&did).expect("revoke should succeed");

    assert_eq!(
        registry.register(did.clone(), document),
        Err(DidRegistryError::Revoked(did.as_str().to_owned()))
    );
}
