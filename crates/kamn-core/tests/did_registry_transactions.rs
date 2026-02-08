use kamn_core::{
    canonical_did_document, AgentDid, AgentDidMetadata, DidChainSubmissionOutcome, DidDocument,
    DidRegistry, DidRegistryError, DidSubmissionFinalityStatus, DidSubmissionRetryClass,
    InMemoryDidRegistrationChainAdapter,
};

fn metadata(model_family: &str) -> AgentDidMetadata {
    AgentDidMetadata {
        agent_type: "autonomous".to_owned(),
        model_family: model_family.to_owned(),
        capabilities: vec!["text".to_owned()],
        operator: None,
    }
}

fn document_for(did: &AgentDid, model_family: &str) -> DidDocument {
    canonical_did_document(did, "z6Mpub", metadata(model_family))
        .expect("did document should build")
}

#[test]
fn register_and_resolve_round_trip() {
    let mut registry = DidRegistry::new();
    let did = AgentDid::parse("kamn:did:agent:agent-1").expect("did should parse");
    let document = document_for(&did, "claude-4");

    registry
        .register(did.clone(), document.clone())
        .expect("register should succeed");
    let resolved = registry.resolve(&did).expect("resolve should succeed");

    assert_eq!(resolved.id, did.as_str().to_owned());
    assert_eq!(resolved.metadata.model_family, "claude-4");
}

#[test]
fn duplicate_register_is_rejected() {
    let mut registry = DidRegistry::new();
    let did = AgentDid::parse("kamn:did:agent:agent-2").expect("did should parse");
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
    let mut registry = DidRegistry::new();
    let did = AgentDid::parse("kamn:did:agent:agent-3").expect("did should parse");
    registry
        .register(did.clone(), document_for(&did, "claude-4"))
        .expect("register should succeed");

    registry
        .update(did.clone(), document_for(&did, "gpt-5"))
        .expect("update should succeed");
    let resolved = registry.resolve(&did).expect("resolve should succeed");
    assert_eq!(resolved.metadata.model_family, "gpt-5");
}

#[test]
fn update_rejects_unknown_did() {
    let mut registry = DidRegistry::new();
    let did = AgentDid::parse("kamn:did:agent:agent-4").expect("did should parse");
    assert_eq!(
        registry.update(did.clone(), document_for(&did, "claude-4")),
        Err(DidRegistryError::NotFound(did.as_str().to_owned()))
    );
}

#[test]
fn revoke_blocks_resolve() {
    let mut registry = DidRegistry::new();
    let did = AgentDid::parse("kamn:did:agent:agent-5").expect("did should parse");
    registry
        .register(did.clone(), document_for(&did, "claude-4"))
        .expect("register should succeed");
    registry.revoke(&did).expect("revoke should succeed");

    assert_eq!(
        registry.resolve(&did),
        Err(DidRegistryError::Revoked(did.as_str().to_owned()))
    );
}

#[test]
fn revoked_did_cannot_be_re_registered() {
    let mut registry = DidRegistry::new();
    let did = AgentDid::parse("kamn:did:agent:agent-6").expect("did should parse");
    let document = document_for(&did, "claude-4");
    registry
        .register(did.clone(), document.clone())
        .expect("register should succeed");
    registry.revoke(&did).expect("revoke should succeed");

    // Regression: #111
    assert_eq!(
        registry.register(did.clone(), document),
        Err(DidRegistryError::Revoked(did.as_str().to_owned()))
    );
}

#[test]
fn retry_classification_is_deterministic_for_duplicate_submission() {
    let mut registry = DidRegistry::new();
    let did = AgentDid::parse("kamn:did:agent:agent-7").expect("did should parse");
    let document = document_for(&did, "claude-4");

    let first = registry
        .register_with_retry_guard(did.clone(), document.clone())
        .expect("initial submission should succeed");
    assert_eq!(first, DidSubmissionRetryClass::NewSubmission);

    let duplicate_before_finality = registry
        .register_with_retry_guard(did.clone(), document.clone())
        .expect("duplicate submission should classify retry state");
    assert_eq!(
        duplicate_before_finality,
        DidSubmissionRetryClass::RetryableInFlight
    );

    let idempotency_key = registry
        .idempotency_key_for_register(&did, &document)
        .expect("idempotency key should derive");
    registry
        .record_register_finality(
            &did,
            &idempotency_key,
            7,
            DidSubmissionFinalityStatus::Confirmed,
            "receipt-7",
        )
        .expect("finality update should succeed");

    let duplicate_after_finality = registry
        .register_with_retry_guard(did.clone(), document)
        .expect("duplicate submission should no-op once finalized");
    assert_eq!(
        duplicate_after_finality,
        DidSubmissionRetryClass::FinalizedNoRetry
    );
}

#[test]
fn integration_register_retry_and_finality_boundary_is_idempotent() {
    let mut registry = DidRegistry::new();
    let did = AgentDid::parse("kamn:did:agent:agent-8").expect("did should parse");
    let document = document_for(&did, "gpt-5");
    let idempotency_key = registry
        .idempotency_key_for_register(&did, &document)
        .expect("idempotency key should derive");

    registry
        .register_with_retry_guard(did.clone(), document.clone())
        .expect("first submit should succeed");

    registry
        .record_register_finality(
            &did,
            &idempotency_key,
            9,
            DidSubmissionFinalityStatus::Confirmed,
            "receipt-9",
        )
        .expect("first finality should succeed");

    registry
        .record_register_finality(
            &did,
            &idempotency_key,
            9,
            DidSubmissionFinalityStatus::Confirmed,
            "receipt-9",
        )
        .expect("duplicate finality should remain idempotent");

    assert_eq!(
        registry.register_with_retry_guard(did.clone(), document),
        Ok(DidSubmissionRetryClass::FinalizedNoRetry)
    );
    assert_eq!(
        registry
            .resolve(&did)
            .expect("did should stay resolvable")
            .id,
        did.as_str().to_owned()
    );
}

#[test]
fn regression_register_finality_rejects_stale_or_conflicting_updates() {
    // Regression: #678
    let mut registry = DidRegistry::new();
    let did = AgentDid::parse("kamn:did:agent:agent-9").expect("did should parse");
    let document = document_for(&did, "gpt-5");
    let idempotency_key = registry
        .idempotency_key_for_register(&did, &document)
        .expect("idempotency key should derive");

    registry
        .register_with_retry_guard(did.clone(), document.clone())
        .expect("submit should succeed");
    registry
        .record_register_finality(
            &did,
            &idempotency_key,
            11,
            DidSubmissionFinalityStatus::Confirmed,
            "receipt-11",
        )
        .expect("initial finality should succeed");

    assert_eq!(
        registry.record_register_finality(
            &did,
            &idempotency_key,
            10,
            DidSubmissionFinalityStatus::Confirmed,
            "receipt-10",
        ),
        Err(DidRegistryError::StaleFinalityUpdate {
            did: did.as_str().to_owned(),
            current_sequence: 11,
            attempted_sequence: 10,
        })
    );

    assert_eq!(
        registry.record_register_finality(
            &did,
            &idempotency_key,
            11,
            DidSubmissionFinalityStatus::Rejected,
            "receipt-11-conflict",
        ),
        Err(DidRegistryError::ConflictingFinalityUpdate {
            did: did.as_str().to_owned(),
            sequence: 11,
        })
    );
}

#[test]
fn functional_chain_submission_adapter_returns_typed_submitted_outcome() {
    let mut registry = DidRegistry::new();
    let mut adapter = InMemoryDidRegistrationChainAdapter::new("ledger-stub");
    let did = AgentDid::parse("kamn:did:agent:agent-10").expect("did should parse");
    let document = document_for(&did, "claude-4");

    let result = registry
        .submit_registration_via_chain_adapter(&mut adapter, did.clone(), document)
        .expect("submit should succeed");

    assert_eq!(result.retry_class, DidSubmissionRetryClass::NewSubmission);
    assert!(matches!(
        result.outcome,
        DidChainSubmissionOutcome::Submitted(_)
    ));
}

#[test]
fn integration_chain_submission_adapter_deduplicates_retry_outcomes() {
    let mut registry = DidRegistry::new();
    let mut adapter = InMemoryDidRegistrationChainAdapter::new("ledger-stub");
    let did = AgentDid::parse("kamn:did:agent:agent-11").expect("did should parse");
    let document = document_for(&did, "gpt-5");

    let first = registry
        .submit_registration_via_chain_adapter(&mut adapter, did.clone(), document.clone())
        .expect("first submit should succeed");
    let second = registry
        .submit_registration_via_chain_adapter(&mut adapter, did.clone(), document)
        .expect("retry submit should succeed");

    assert_eq!(first.retry_class, DidSubmissionRetryClass::NewSubmission);
    assert_eq!(
        second.retry_class,
        DidSubmissionRetryClass::RetryableInFlight
    );
    assert!(matches!(
        first.outcome,
        DidChainSubmissionOutcome::Submitted(_)
    ));
    assert!(matches!(
        second.outcome,
        DidChainSubmissionOutcome::Duplicate(_)
    ));
}

#[test]
fn regression_chain_submission_adapter_exposes_rejected_outcome_without_panicking() {
    // Regression: #678
    let mut registry = DidRegistry::new();
    let mut adapter = InMemoryDidRegistrationChainAdapter::new("ledger-stub");
    let did = AgentDid::parse("kamn:did:agent:agent-12").expect("did should parse");
    let document = document_for(&did, "gpt-5");
    let idempotency_key = registry
        .idempotency_key_for_register(&did, &document)
        .expect("idempotency key should derive");

    adapter.reject_idempotency_key(&idempotency_key, "simulated-ledger-reject");

    let rejected = registry
        .submit_registration_via_chain_adapter(&mut adapter, did.clone(), document)
        .expect("submission result should remain typed");

    assert!(matches!(
        rejected.outcome,
        DidChainSubmissionOutcome::Rejected { .. }
    ));
}
