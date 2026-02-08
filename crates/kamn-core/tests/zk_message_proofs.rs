use kamn_core::{
    build_message_witness, phase4_baseline_options, recommend_phase4_plan, AttachmentRef,
    CanonicalMessageEnvelope, EnvelopeEncryption, EnvelopeHeader, EnvelopeMetadata, EnvelopeProof,
    MessageEnvelopeError, ProcessorProofAdmissionEvaluator, ProcessorProofAdmissionInput,
    ProcessorProofArtifact, ProofWatchdogProjectionKind, ProofWatchdogProjector,
    ProofWatchdogSeverity, ValidatorProofAttestation, ValidatorProofConsensusError,
    ValidatorProofConsensusEvaluator, ValidatorProofConsensusInput, ValidatorProofConsensusStatus,
    ValidatorProofVerdict, ZkArchitectureOption, ZkDesignError, ZkEvaluationPolicy, ZkProofSystem,
    ZkVerificationTopology,
};
use std::collections::BTreeMap;

fn valid_envelope() -> CanonicalMessageEnvelope {
    let mut body = BTreeMap::new();
    body.insert(
        "task.description".to_owned(),
        "Classify customer ticket".to_owned(),
    );
    body.insert("task.type".to_owned(), "support".to_owned());

    CanonicalMessageEnvelope {
        envelope: EnvelopeMetadata {
            id: "urn:uuid:420e8400-e29b-41d4-a716-446655440000".to_owned(),
            type_name: "kamn:message:v1".to_owned(),
            from: "kamn:did:agent:sender-1".to_owned(),
            to: vec!["kamn:did:agent:recipient-1".to_owned()],
            created: "2026-02-08T00:10:00.000Z".to_owned(),
            expires: "2026-02-08T00:40:00.000Z".to_owned(),
            thread_id: Some("urn:uuid:thread-9001".to_owned()),
            parent_id: None,
            nonce: 7,
        },
        header: EnvelopeHeader {
            message_type: "Request".to_owned(),
            priority: "Normal".to_owned(),
            content_type: "application/json".to_owned(),
            encryption: EnvelopeEncryption {
                algorithm: "X25519-XChaCha20-Poly1305".to_owned(),
                recipient_keys: vec!["kamn:did:agent:recipient-1#key-agreement-1".to_owned()],
            },
        },
        body,
        attachments: vec![AttachmentRef {
            id: "attachment-1".to_owned(),
            media_type: "text/plain".to_owned(),
            uri: "ipfs://QmWitness".to_owned(),
        }],
        proof: EnvelopeProof {
            type_name: "Ed25519Signature2020".to_owned(),
            created: "2026-02-08T00:10:00.000Z".to_owned(),
            verification_method: "kamn:did:agent:sender-1#keys-1".to_owned(),
            proof_purpose: "authentication".to_owned(),
            proof_value: "z58DAdFfa9SkqZ".to_owned(),
        },
    }
}

#[test]
fn zk_message_proofs_recommend_plan_under_phase4_policy() {
    let plan = recommend_phase4_plan(
        &phase4_baseline_options(),
        ZkEvaluationPolicy {
            max_verifier_latency_ms: 25,
            max_proof_size_bytes: 2_048,
            max_engineering_weeks: 12,
            require_transparent_setup: true,
        },
    )
    .expect("phase 4 plan should evaluate");

    assert_eq!(plan.recommended_option, "plonkish-batched-envelope");
    assert!(plan.rationale.contains("transparent setup"));
    assert_eq!(plan.milestones.len(), 3);
}

#[test]
fn zk_message_proofs_reject_invalid_policy_boundaries() {
    let result = recommend_phase4_plan(
        &phase4_baseline_options(),
        ZkEvaluationPolicy {
            max_verifier_latency_ms: 0,
            max_proof_size_bytes: 2_048,
            max_engineering_weeks: 12,
            require_transparent_setup: true,
        },
    );

    assert_eq!(
        result,
        Err(ZkDesignError::InvalidPolicy(
            "max_verifier_latency_ms must be greater than zero".to_owned()
        ))
    );
}

#[test]
fn zk_message_proofs_integration_builds_witness_for_hidden_fields() {
    let witness = build_message_witness(&valid_envelope(), &["task.description"])
        .expect("witness generation should succeed");

    assert!(witness.public_commitment.starts_with("fnv1a64:"));
    assert_eq!(witness.hidden_field_count, 1);
    assert!(witness.revealed_fields.contains(&"task.type".to_owned()));
    assert!(!witness
        .revealed_fields
        .contains(&"task.description".to_owned()));
    assert!(witness.payload_bytes > 0);
}

#[test]
fn zk_message_proofs_reject_missing_hidden_field() {
    let result = build_message_witness(&valid_envelope(), &["task.unknown"]);
    assert_eq!(
        result,
        Err(ZkDesignError::MissingPrivateField(
            "task.unknown".to_owned()
        ))
    );
}

#[test]
fn zk_message_proofs_reports_envelope_validation_failures() {
    let mut envelope = valid_envelope();
    envelope.envelope.type_name = "kamn:message:v2".to_owned();

    let result = build_message_witness(&envelope, &[]);
    assert_eq!(
        result,
        Err(ZkDesignError::EnvelopeError(
            MessageEnvelopeError::InvalidEnvelopeType("kamn:message:v2".to_owned())
        ))
    );
}

#[test]
fn zk_message_proofs_regression_threshold_boundaries_are_inclusive() {
    // Regression: #62
    let option = ZkArchitectureOption {
        name: "boundary".to_owned(),
        proof_system: ZkProofSystem::Plonkish,
        verification_topology: ZkVerificationTopology::ValidatorQuorum,
        trusted_setup_required: false,
        deterministic_witness_inputs: true,
        prover_latency_ms: 200,
        verifier_latency_ms: 25,
        proof_size_bytes: 2_048,
        supports_batching: true,
        estimated_engineering_weeks: 12,
    };

    let plan = recommend_phase4_plan(
        &[option],
        ZkEvaluationPolicy {
            max_verifier_latency_ms: 25,
            max_proof_size_bytes: 2_048,
            max_engineering_weeks: 12,
            require_transparent_setup: true,
        },
    )
    .expect("boundary option should remain feasible");

    assert_eq!(plan.recommended_option, "boundary");
}

#[test]
fn zk_message_proofs_regression_rejects_tampered_processor_proof_artifact() {
    // Regression: #509
    let envelope = valid_envelope();
    let witness = build_message_witness(&envelope, &["task.description"])
        .expect("witness generation should succeed");
    let mut evaluator = ProcessorProofAdmissionEvaluator::new();

    let artifact = ProcessorProofArtifact::new(
        "artifact-1",
        &envelope.envelope.id,
        "fnv1a64:tampered",
        "proof:ok:artifact-1",
    )
    .expect("artifact should parse");
    let input = ProcessorProofAdmissionInput::new(
        &envelope.envelope.id,
        &witness.public_commitment,
        artifact,
    )
    .expect("input should parse");

    assert_eq!(
        evaluator.evaluate(input),
        Err(ZkDesignError::ProofArtifactCommitmentMismatch {
            expected: witness.public_commitment,
            found: "fnv1a64:tampered".to_owned(),
        })
    );
}

#[test]
fn zk_message_proofs_functional_processor_admission_accepts_valid_artifact() {
    let envelope = valid_envelope();
    let witness = build_message_witness(&envelope, &["task.description"])
        .expect("witness generation should succeed");
    let mut evaluator = ProcessorProofAdmissionEvaluator::new();

    let artifact = ProcessorProofArtifact::new(
        "artifact-2",
        &envelope.envelope.id,
        &witness.public_commitment,
        "proof:ok:artifact-2",
    )
    .expect("artifact should parse");
    let input = ProcessorProofAdmissionInput::new(
        &envelope.envelope.id,
        &witness.public_commitment,
        artifact,
    )
    .expect("input should parse");

    let decision = evaluator
        .evaluate(input)
        .expect("valid artifact should be admitted");
    assert_eq!(decision.message_id, envelope.envelope.id);
    assert_eq!(decision.artifact_id, "artifact-2".to_owned());
}

#[test]
fn zk_message_proofs_integration_rejects_replayed_processor_artifact() {
    let envelope = valid_envelope();
    let witness = build_message_witness(&envelope, &["task.description"])
        .expect("witness generation should succeed");
    let mut evaluator = ProcessorProofAdmissionEvaluator::new();

    let first_input = ProcessorProofAdmissionInput::new(
        &envelope.envelope.id,
        &witness.public_commitment,
        ProcessorProofArtifact::new(
            "artifact-3",
            &envelope.envelope.id,
            &witness.public_commitment,
            "proof:ok:artifact-3",
        )
        .expect("artifact should parse"),
    )
    .expect("input should parse");
    assert!(evaluator.evaluate(first_input).is_ok());

    let replay_input = ProcessorProofAdmissionInput::new(
        &envelope.envelope.id,
        &witness.public_commitment,
        ProcessorProofArtifact::new(
            "artifact-3",
            &envelope.envelope.id,
            &witness.public_commitment,
            "proof:ok:artifact-3",
        )
        .expect("artifact should parse"),
    )
    .expect("input should parse");

    assert_eq!(
        evaluator.evaluate(replay_input),
        Err(ZkDesignError::ProofArtifactReplay("artifact-3".to_owned()))
    );
}

#[test]
fn zk_message_proofs_functional_validator_quorum_consensus_aligned_valid_is_nominal() {
    let envelope = valid_envelope();
    let mut evaluator =
        ValidatorProofConsensusEvaluator::new(2).expect("consensus evaluator should build");

    let input = ValidatorProofConsensusInput::new(
        &envelope.envelope.id,
        "artifact-quorum-valid",
        vec![
            ValidatorProofAttestation::new(
                "attestation-valid-1",
                "kamn:did:agent:validator-z",
                &envelope.envelope.id,
                "artifact-quorum-valid",
                ValidatorProofVerdict::Valid,
            )
            .expect("first attestation should parse"),
            ValidatorProofAttestation::new(
                "attestation-valid-2",
                "kamn:did:agent:validator-a",
                &envelope.envelope.id,
                "artifact-quorum-valid",
                ValidatorProofVerdict::Valid,
            )
            .expect("second attestation should parse"),
        ],
    )
    .expect("consensus input should parse");

    let decision = evaluator
        .evaluate(input)
        .expect("aligned consensus should evaluate");
    let projection = ProofWatchdogProjector::new().project(&decision);

    assert_eq!(
        decision.status,
        ValidatorProofConsensusStatus::ConsensusValid
    );
    assert_eq!(
        decision.validator_dids,
        vec![
            "kamn:did:agent:validator-a".to_owned(),
            "kamn:did:agent:validator-z".to_owned()
        ],
        "validator did ordering should be deterministic"
    );
    assert_eq!(
        projection.kind,
        ProofWatchdogProjectionKind::ConsensusAligned
    );
    assert_eq!(projection.severity, ProofWatchdogSeverity::Info);
}

#[test]
fn zk_message_proofs_integration_rejects_replayed_validator_attestation_id() {
    let envelope = valid_envelope();
    let mut evaluator =
        ValidatorProofConsensusEvaluator::new(1).expect("consensus evaluator should build");

    let first = ValidatorProofConsensusInput::new(
        &envelope.envelope.id,
        "artifact-quorum-replay",
        vec![ValidatorProofAttestation::new(
            "attestation-replay-1",
            "kamn:did:agent:validator-a",
            &envelope.envelope.id,
            "artifact-quorum-replay",
            ValidatorProofVerdict::Valid,
        )
        .expect("first attestation should parse")],
    )
    .expect("first input should parse");
    assert!(evaluator.evaluate(first).is_ok());

    let replayed = ValidatorProofConsensusInput::new(
        &envelope.envelope.id,
        "artifact-quorum-replay",
        vec![ValidatorProofAttestation::new(
            "attestation-replay-1",
            "kamn:did:agent:validator-b",
            &envelope.envelope.id,
            "artifact-quorum-replay",
            ValidatorProofVerdict::Replay,
        )
        .expect("second attestation should parse")],
    )
    .expect("second input should parse");

    assert_eq!(
        evaluator.evaluate(replayed),
        Err(ValidatorProofConsensusError::AttestationReplay(
            "attestation-replay-1".to_owned()
        ))
    );
}

#[test]
fn zk_message_proofs_regression_projects_validator_invalid_mismatch_to_watchdog_signal() {
    // Regression: #509
    let envelope = valid_envelope();
    let witness = build_message_witness(&envelope, &["task.description"])
        .expect("witness generation should succeed");
    let mut evaluator =
        ValidatorProofConsensusEvaluator::new(2).expect("consensus evaluator should build");

    let input = ValidatorProofConsensusInput::new(
        &envelope.envelope.id,
        "artifact-quorum-1",
        vec![
            ValidatorProofAttestation::new(
                "attestation-1",
                "kamn:did:agent:validator-a",
                &envelope.envelope.id,
                "artifact-quorum-1",
                ValidatorProofVerdict::Valid,
            )
            .expect("first attestation should parse"),
            ValidatorProofAttestation::new(
                "attestation-2",
                "kamn:did:agent:validator-b",
                &envelope.envelope.id,
                "artifact-quorum-1",
                ValidatorProofVerdict::Invalid,
            )
            .expect("second attestation should parse"),
        ],
    )
    .expect("consensus input should parse");

    let decision = evaluator
        .evaluate(input)
        .expect("mismatch consensus decision should evaluate");
    let projection = ProofWatchdogProjector::new().project(&decision);

    assert_eq!(
        decision.message_id, envelope.envelope.id,
        "consensus should preserve message id"
    );
    assert_eq!(
        projection.kind,
        ProofWatchdogProjectionKind::ValidatorMismatch
    );
    assert_eq!(projection.severity, ProofWatchdogSeverity::Critical);
    assert_eq!(projection.valid_attestation_count, 1);
    assert_eq!(projection.invalid_attestation_count, 1);
    assert_eq!(projection.replay_attestation_count, 0);
    assert_eq!(
        projection.artifact_id,
        "artifact-quorum-1".to_owned(),
        "projection should preserve artifact id"
    );
    assert_eq!(
        projection.required_quorum, 2,
        "projection should preserve required quorum"
    );
    assert_ne!(
        witness.public_commitment, "fnv1a64:tampered",
        "regression setup must use valid witness commitment baseline"
    );
}
