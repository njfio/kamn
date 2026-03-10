use kamn_core::{
    canonical_did_document, AgentDid, AgentDidMetadata, DidLifecycleMutationAction,
    DidLifecycleMutationRequest, DidRegistry, DidRegistryError, DidSubmissionFinalityStatus,
    InMemoryDidRegistrationChainAdapter, InMemoryKolmeRuntimeCommitClient,
    KolmeDidLifecycleChainAdapter,
};
pub(crate) use kamn_core::{DidChainSubmissionOutcome, DidSubmissionRetryClass};
pub(crate) use std::time::Instant;

pub(crate) fn metadata(model_family: &str) -> AgentDidMetadata {
    AgentDidMetadata {
        agent_type: "autonomous".to_owned(),
        model_family: model_family.to_owned(),
        capabilities: vec!["text".to_owned()],
        operator: None,
    }
}

pub(crate) fn document_for(did: &AgentDid, model_family: &str) -> kamn_core::DidDocument {
    canonical_did_document(did, "z6Mpub", metadata(model_family))
        .expect("did document should build")
}

pub(crate) fn parse_did(value: &str) -> AgentDid {
    AgentDid::parse(value).expect("did should parse")
}

pub(crate) fn registry() -> DidRegistry {
    DidRegistry::new()
}

pub(crate) fn register_document(registry: &mut DidRegistry, did: &AgentDid, family: &str) {
    registry
        .register(did.clone(), document_for(did, family))
        .expect("register should succeed");
}

pub(crate) fn set_operator(document: &mut kamn_core::DidDocument, operator: &str) {
    document.metadata.operator = Some(operator.to_owned());
}

pub(crate) fn apply_mutation(
    registry: &mut DidRegistry,
    did: &AgentDid,
    actor_did: &str,
    nonce: u64,
    action: DidLifecycleMutationAction,
) -> kamn_core::DidLifecycleMutationEvidence {
    registry
        .apply_lifecycle_mutation(DidLifecycleMutationRequest {
            did: did.clone(),
            actor_did: actor_did.to_owned(),
            nonce,
            action,
        })
        .expect("lifecycle mutation should succeed")
}

pub(crate) fn registration_adapter() -> InMemoryDidRegistrationChainAdapter {
    InMemoryDidRegistrationChainAdapter::new("ledger-stub")
}

pub(crate) fn lifecycle_adapter() -> KolmeDidLifecycleChainAdapter<InMemoryKolmeRuntimeCommitClient> {
    let client = InMemoryKolmeRuntimeCommitClient::new("kolme-live").expect("client should build");
    KolmeDidLifecycleChainAdapter::new(client, "did-lifecycle-state").expect("adapter")
}

pub(crate) fn confirmed_register_finality(
    registry: &mut DidRegistry,
    did: &AgentDid,
    document: &kamn_core::DidDocument,
    sequence: u64,
    receipt: &str,
) {
    let idempotency_key = registry
        .idempotency_key_for_register(did, document)
        .expect("idempotency key should derive");
    registry
        .record_register_finality(
            did,
            &idempotency_key,
            sequence,
            DidSubmissionFinalityStatus::Confirmed,
            receipt,
        )
        .expect("finality update should succeed");
}

pub(crate) fn assert_replay_conflict(error: DidRegistryError) {
    assert_eq!(error.reason_code(), "did_lifecycle_mutation_nonce_replay");
}
