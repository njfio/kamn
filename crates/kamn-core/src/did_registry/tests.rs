use super::{DidRegistry, DidRegistryError, DidSubmissionRetryClass};
use crate::{canonical_did_document, AgentDid, AgentDidMetadata};

fn metadata() -> AgentDidMetadata {
    AgentDidMetadata {
        agent_type: "autonomous".to_owned(),
        model_family: "claude-4".to_owned(),
        capabilities: vec!["text".to_owned()],
        operator: None,
    }
}

fn document_for(did: &AgentDid) -> crate::DidDocument {
    canonical_did_document(did, "z6Mpubkey", metadata()).expect("document should build")
}

#[test]
fn rejects_document_mismatch() {
    let mut registry = DidRegistry::new();
    let did = AgentDid::parse("kamn:did:agent:agent-1").expect("did should parse");
    let other = AgentDid::parse("kamn:did:agent:agent-2").expect("did should parse");
    assert_eq!(
        registry.register(did.clone(), document_for(&other)),
        Err(DidRegistryError::DocumentDidMismatch {
            expected: did.as_str().to_owned(),
            actual: other.as_str().to_owned(),
        })
    );
}

#[test]
fn update_rejects_revoked_did() {
    let mut registry = DidRegistry::new();
    let did = AgentDid::parse("kamn:did:agent:agent-3").expect("did should parse");
    registry
        .register(did.clone(), document_for(&did))
        .expect("register should succeed");
    registry.revoke(&did).expect("revoke should succeed");
    assert_eq!(
        registry.update(did.clone(), document_for(&did)),
        Err(DidRegistryError::Revoked(did.as_str().to_owned()))
    );
}

#[test]
fn idempotency_key_generation_is_deterministic() {
    let registry = DidRegistry::new();
    let did = AgentDid::parse("kamn:did:agent:agent-4").expect("did should parse");
    let document = document_for(&did);
    let key_a = registry
        .idempotency_key_for_register(&did, &document)
        .expect("first key should derive");
    let key_b = registry
        .idempotency_key_for_register(&did, &document)
        .expect("second key should derive");
    assert_eq!(key_a, key_b);
}

#[test]
fn retry_classification_rejects_conflicting_document_key() {
    let mut registry = DidRegistry::new();
    let did = AgentDid::parse("kamn:did:agent:agent-5").expect("did should parse");
    let original = document_for(&did);
    let mut changed = document_for(&did);
    changed.metadata.model_family = "gpt-5".to_owned();
    assert_eq!(
        registry
            .register_with_retry_guard(did.clone(), original.clone())
            .expect("first submit should succeed"),
        DidSubmissionRetryClass::NewSubmission
    );
    assert_eq!(
        registry
            .classify_register_retry(&did, &original)
            .expect("duplicate should classify"),
        DidSubmissionRetryClass::RetryableInFlight
    );
    assert_eq!(
        registry
            .classify_register_retry(&did, &changed)
            .expect("changed document should classify"),
        DidSubmissionRetryClass::ConflictNoRetry
    );
}
