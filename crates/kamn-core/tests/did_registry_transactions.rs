use kamn_core::{
    canonical_did_document, AgentDid, AgentDidMetadata, DidChainSubmissionOutcome, DidDocument,
    DidLifecycleMutationAction, DidLifecycleMutationRequest, DidRegistry, DidRegistryError,
    DidSubmissionFinalityStatus, DidSubmissionRetryClass, InMemoryDidRegistrationChainAdapter,
    InMemoryKolmeRuntimeCommitClient, KolmeDidLifecycleChainAdapter,
};
use std::time::Instant;

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

#[test]
fn unit_lifecycle_mutation_nonce_guards_emit_deterministic_reason_codes() {
    let mut registry = DidRegistry::new();
    let did = AgentDid::parse("kamn:did:agent:lifecycle-unit-1").expect("did should parse");
    let mut initial_document = document_for(&did, "claude-4");
    initial_document.metadata.operator = Some("kamn:did:human:ops-1".to_owned());
    registry
        .register(did.clone(), initial_document.clone())
        .expect("register should succeed");

    let zero_nonce_error = registry
        .apply_lifecycle_mutation(DidLifecycleMutationRequest {
            did: did.clone(),
            actor_did: "kamn:did:human:ops-1".to_owned(),
            nonce: 0,
            action: DidLifecycleMutationAction::Revoke,
        })
        .expect_err("zero nonce mutation should fail");
    assert_eq!(
        zero_nonce_error.reason_code(),
        "did_lifecycle_mutation_nonce_invalid"
    );

    registry
        .apply_lifecycle_mutation(DidLifecycleMutationRequest {
            did: did.clone(),
            actor_did: "kamn:did:human:ops-1".to_owned(),
            nonce: 1,
            action: DidLifecycleMutationAction::Rotate {
                document: initial_document,
            },
        })
        .expect("first mutation should succeed");

    let replay_error = registry
        .apply_lifecycle_mutation(DidLifecycleMutationRequest {
            did: did.clone(),
            actor_did: "kamn:did:human:ops-1".to_owned(),
            nonce: 1,
            action: DidLifecycleMutationAction::Revoke,
        })
        .expect_err("replayed nonce should fail");
    assert_eq!(
        replay_error.reason_code(),
        "did_lifecycle_mutation_nonce_replay"
    );
}

#[test]
fn functional_lifecycle_rotate_mutation_updates_document_and_emits_allowed_reason_code() {
    let mut registry = DidRegistry::new();
    let did = AgentDid::parse("kamn:did:agent:lifecycle-func-1").expect("did should parse");
    let mut initial_document = document_for(&did, "claude-4");
    initial_document.metadata.operator = Some("kamn:did:human:ops-2".to_owned());
    registry
        .register(did.clone(), initial_document)
        .expect("register should succeed");

    let mut rotated_document = document_for(&did, "gpt-5");
    rotated_document.metadata.operator = Some("kamn:did:human:ops-2".to_owned());
    let evidence = registry
        .apply_lifecycle_mutation(DidLifecycleMutationRequest {
            did: did.clone(),
            actor_did: "kamn:did:human:ops-2".to_owned(),
            nonce: 7,
            action: DidLifecycleMutationAction::Rotate {
                document: rotated_document,
            },
        })
        .expect("rotate mutation should succeed");

    assert_eq!(evidence.reason_code, "did_lifecycle_mutation_allowed");
    assert_eq!(
        registry
            .resolve(&did)
            .expect("resolve should succeed")
            .metadata
            .model_family,
        "gpt-5"
    );
}

#[test]
fn integration_lifecycle_revoke_then_recover_restores_active_resolution() {
    let mut registry = DidRegistry::new();
    let did = AgentDid::parse("kamn:did:agent:lifecycle-int-1").expect("did should parse");
    let mut initial_document = document_for(&did, "claude-4");
    initial_document.metadata.operator = Some("kamn:did:human:ops-3".to_owned());
    registry
        .register(did.clone(), initial_document)
        .expect("register should succeed");

    registry
        .apply_lifecycle_mutation(DidLifecycleMutationRequest {
            did: did.clone(),
            actor_did: "kamn:did:human:ops-3".to_owned(),
            nonce: 1,
            action: DidLifecycleMutationAction::Revoke,
        })
        .expect("revoke mutation should succeed");
    assert_eq!(
        registry.resolve(&did),
        Err(DidRegistryError::Revoked(did.as_str().to_owned()))
    );

    let mut recovered_document = document_for(&did, "claude-4.1");
    recovered_document.metadata.operator = Some("kamn:did:human:ops-3".to_owned());
    let recovery_evidence = registry
        .apply_lifecycle_mutation(DidLifecycleMutationRequest {
            did: did.clone(),
            actor_did: "kamn:did:human:ops-3".to_owned(),
            nonce: 2,
            action: DidLifecycleMutationAction::Recover {
                document: recovered_document,
            },
        })
        .expect("recover mutation should succeed");

    assert_eq!(
        recovery_evidence.reason_code,
        "did_lifecycle_mutation_allowed"
    );
    assert_eq!(
        registry
            .resolve(&did)
            .expect("resolve should recover")
            .metadata
            .model_family,
        "claude-4.1"
    );
}

#[test]
fn regression_lifecycle_replayed_or_unauthorized_mutation_fails_closed() {
    // Regression: #889
    let mut registry = DidRegistry::new();
    let did = AgentDid::parse("kamn:did:agent:lifecycle-reg-1").expect("did should parse");
    let mut document = document_for(&did, "claude-4");
    document.metadata.operator = Some("kamn:did:human:ops-4".to_owned());
    registry
        .register(did.clone(), document)
        .expect("register should succeed");

    let unauthorized_error = registry
        .apply_lifecycle_mutation(DidLifecycleMutationRequest {
            did: did.clone(),
            actor_did: "kamn:did:human:intruder-4".to_owned(),
            nonce: 1,
            action: DidLifecycleMutationAction::Revoke,
        })
        .expect_err("unauthorized mutation must fail");
    assert_eq!(
        unauthorized_error.reason_code(),
        "did_lifecycle_mutation_unauthorized_actor"
    );

    registry
        .apply_lifecycle_mutation(DidLifecycleMutationRequest {
            did: did.clone(),
            actor_did: "kamn:did:human:ops-4".to_owned(),
            nonce: 1,
            action: DidLifecycleMutationAction::Revoke,
        })
        .expect("authorized revoke should succeed");

    let replay_error = registry
        .apply_lifecycle_mutation(DidLifecycleMutationRequest {
            did: did.clone(),
            actor_did: "kamn:did:human:ops-4".to_owned(),
            nonce: 1,
            action: DidLifecycleMutationAction::Recover {
                document: {
                    let mut recovered = document_for(&did, "claude-4.2");
                    recovered.metadata.operator = Some("kamn:did:human:ops-4".to_owned());
                    recovered
                },
            },
        })
        .expect_err("replayed mutation nonce must fail");
    assert_eq!(
        replay_error.reason_code(),
        "did_lifecycle_mutation_nonce_replay"
    );
}

#[test]
fn performance_lifecycle_mutation_contract_lane_stays_within_budget() {
    let started = Instant::now();

    for round in 0..64 {
        let mut registry = DidRegistry::new();
        let did = AgentDid::parse(format!("kamn:did:agent:lifecycle-perf-{round}").as_str())
            .expect("did should parse");
        let mut document = document_for(&did, "claude-4");
        document.metadata.operator = Some("kamn:did:human:ops-perf".to_owned());
        registry
            .register(did.clone(), document)
            .expect("register should succeed");

        registry
            .apply_lifecycle_mutation(DidLifecycleMutationRequest {
                did: did.clone(),
                actor_did: "kamn:did:human:ops-perf".to_owned(),
                nonce: 1,
                action: DidLifecycleMutationAction::Revoke,
            })
            .expect("revoke should succeed");

        let mut recovered_document = document_for(&did, "claude-4.1");
        recovered_document.metadata.operator = Some("kamn:did:human:ops-perf".to_owned());
        registry
            .apply_lifecycle_mutation(DidLifecycleMutationRequest {
                did,
                actor_did: "kamn:did:human:ops-perf".to_owned(),
                nonce: 2,
                action: DidLifecycleMutationAction::Recover {
                    document: recovered_document,
                },
            })
            .expect("recover should succeed");
    }

    let elapsed_millis = started.elapsed().as_millis();
    assert!(
        elapsed_millis < 800,
        "did lifecycle mutation contract lane exceeded budget: {elapsed_millis}ms"
    );
}

#[test]
fn functional_lifecycle_chain_submission_through_kolme_adapter_returns_typed_outcome() {
    let mut registry = DidRegistry::new();
    let did =
        AgentDid::parse("kamn:did:agent:lifecycle-chain-functional").expect("did should parse");
    let mut document = document_for(&did, "claude-4");
    document.metadata.operator = Some("kamn:did:human:ops-chain-functional".to_owned());
    registry
        .register(did.clone(), document)
        .expect("register should succeed");

    let client = InMemoryKolmeRuntimeCommitClient::new("kolme-live").expect("client should build");
    let mut adapter =
        KolmeDidLifecycleChainAdapter::new(client, "did-lifecycle-state").expect("adapter");
    let result = registry
        .submit_lifecycle_mutation_via_chain_adapter(
            &mut adapter,
            DidLifecycleMutationRequest {
                did: did.clone(),
                actor_did: "kamn:did:human:ops-chain-functional".to_owned(),
                nonce: 1,
                action: DidLifecycleMutationAction::Revoke,
            },
        )
        .expect("lifecycle chain submission should succeed");

    assert_eq!(result.retry_class, DidSubmissionRetryClass::NewSubmission);
    assert_eq!(
        result.evidence.reason_code,
        "did_lifecycle_mutation_allowed"
    );
    assert!(matches!(
        result.outcome,
        DidChainSubmissionOutcome::Submitted(_)
    ));
}

#[test]
fn integration_lifecycle_chain_submission_allows_retry_without_reapplying_mutation() {
    let mut registry = DidRegistry::new();
    let did =
        AgentDid::parse("kamn:did:agent:lifecycle-chain-integration").expect("did should parse");
    let mut document = document_for(&did, "claude-4");
    document.metadata.operator = Some("kamn:did:human:ops-chain-integration".to_owned());
    registry
        .register(did.clone(), document)
        .expect("register should succeed");

    let rotated_document = {
        let mut value = document_for(&did, "gpt-5");
        value.metadata.operator = Some("kamn:did:human:ops-chain-integration".to_owned());
        value
    };
    let request = DidLifecycleMutationRequest {
        did: did.clone(),
        actor_did: "kamn:did:human:ops-chain-integration".to_owned(),
        nonce: 8,
        action: DidLifecycleMutationAction::Rotate {
            document: rotated_document,
        },
    };

    let client = InMemoryKolmeRuntimeCommitClient::new("kolme-live").expect("client should build");
    let mut adapter =
        KolmeDidLifecycleChainAdapter::new(client, "did-lifecycle-state").expect("adapter");
    let first = registry
        .submit_lifecycle_mutation_via_chain_adapter(&mut adapter, request.clone())
        .expect("first lifecycle submission should succeed");
    let second = registry
        .submit_lifecycle_mutation_via_chain_adapter(&mut adapter, request)
        .expect("retry lifecycle submission should succeed");

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
    assert_eq!(
        registry
            .resolve(&did)
            .expect("rotate should remain applied")
            .metadata
            .model_family,
        "gpt-5"
    );
}

#[test]
fn regression_lifecycle_chain_submission_rejects_conflicting_same_nonce_payload() {
    // Regression: #2936
    let mut registry = DidRegistry::new();
    let did =
        AgentDid::parse("kamn:did:agent:lifecycle-chain-regression").expect("did should parse");
    let mut document = document_for(&did, "claude-4");
    document.metadata.operator = Some("kamn:did:human:ops-chain-regression".to_owned());
    registry
        .register(did.clone(), document)
        .expect("register should succeed");

    let client = InMemoryKolmeRuntimeCommitClient::new("kolme-live").expect("client should build");
    let mut adapter =
        KolmeDidLifecycleChainAdapter::new(client, "did-lifecycle-state").expect("adapter");

    registry
        .submit_lifecycle_mutation_via_chain_adapter(
            &mut adapter,
            DidLifecycleMutationRequest {
                did: did.clone(),
                actor_did: "kamn:did:human:ops-chain-regression".to_owned(),
                nonce: 4,
                action: DidLifecycleMutationAction::Revoke,
            },
        )
        .expect("first lifecycle submission should succeed");

    let conflicting = registry.submit_lifecycle_mutation_via_chain_adapter(
        &mut adapter,
        DidLifecycleMutationRequest {
            did: did.clone(),
            actor_did: "kamn:did:human:ops-chain-regression".to_owned(),
            nonce: 4,
            action: DidLifecycleMutationAction::Recover {
                document: {
                    let mut recovered = document_for(&did, "gpt-5");
                    recovered.metadata.operator =
                        Some("kamn:did:human:ops-chain-regression".to_owned());
                    recovered
                },
            },
        },
    );

    assert!(matches!(
        conflicting,
        Err(DidRegistryError::ConflictingSubmissionIdempotencyKey { .. })
    ));
}
