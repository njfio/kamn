use kamn_core::{
    AgentDid, AgentDidError, AttachmentRef, CanonicalMessageEnvelope, EnvelopeEncryption,
    EnvelopeHeader, EnvelopeMetadata, EnvelopeProof, MessageEnvelopeError,
};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum EnvelopeCoverageClass {
    Accept,
    InvalidSenderDid,
    InvalidRecipientDid,
    InvalidMessageType,
    InvalidEncryptionAlgorithm,
    EmptyBody,
    InvalidProofPurpose,
    ProofVerificationMethodMismatch,
    OtherRejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum DidCoverageClass {
    Accept,
    InvalidPrefix,
    MissingMethodSpecificId,
    InvalidCharacter,
}

fn base_body() -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    map.insert("task.type".to_owned(), "analysis".to_owned());
    map.insert(
        "task.description".to_owned(),
        "coverage-guided seed payload".to_owned(),
    );
    map
}

fn valid_envelope() -> CanonicalMessageEnvelope {
    CanonicalMessageEnvelope {
        envelope: EnvelopeMetadata {
            id: "urn:uuid:coverage-guided".to_owned(),
            type_name: "kamn:message:v1".to_owned(),
            from: "kamn:did:agent:sender-1".to_owned(),
            to: vec!["kamn:did:agent:recipient-1".to_owned()],
            created: "2026-02-09T05:00:00.000Z".to_owned(),
            expires: "2026-02-09T05:10:00.000Z".to_owned(),
            thread_id: Some("urn:uuid:coverage-guided-thread".to_owned()),
            parent_id: None,
            nonce: 11,
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
        body: base_body(),
        attachments: vec![AttachmentRef {
            id: "attachment-1".to_owned(),
            media_type: "application/json".to_owned(),
            uri: "ipfs://bafybeicoverageguided".to_owned(),
        }],
        proof: EnvelopeProof {
            type_name: "Ed25519Signature2020".to_owned(),
            created: "2026-02-09T05:00:00.000Z".to_owned(),
            verification_method: "kamn:did:agent:sender-1#keys-1".to_owned(),
            proof_purpose: "authentication".to_owned(),
            proof_value: "z58DAdFfa9SkqZMVPxAQp".to_owned(),
        },
    }
}

fn mutation_slot(seed: u64, slots: usize) -> usize {
    let mixed = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    (mixed % slots as u64) as usize
}

fn envelope_from_seed(seed: u64) -> CanonicalMessageEnvelope {
    let mut envelope = valid_envelope();
    match mutation_slot(seed, 8) {
        0 => {}
        1 => {
            envelope.envelope.from = "did:example:sender".to_owned();
        }
        2 => {
            envelope.envelope.to = vec!["did:example:recipient".to_owned()];
        }
        3 => {
            envelope.header.message_type = "UnknownType".to_owned();
        }
        4 => {
            envelope.header.encryption.algorithm = "AES-GCM".to_owned();
        }
        5 => {
            envelope.body.clear();
        }
        6 => {
            envelope.proof.proof_purpose = "assertionMethod".to_owned();
        }
        7 => {
            envelope.proof.verification_method = "kamn:did:agent:attacker-1#keys-9".to_owned();
        }
        _ => unreachable!("envelope mutation slot is bounded"),
    }
    envelope
}

fn did_from_seed(seed: u64) -> String {
    match mutation_slot(seed.rotate_left(17), 5) {
        0 => format!("kamn:did:agent:coverage-{:04}", seed % 10_000),
        1 => format!("did:example:coverage-{:04}", seed % 10_000),
        2 => "kamn:did:agent:".to_owned(),
        3 => format!("kamn:did:agent:coverage {:04}", seed % 10_000),
        4 => format!("kamn:did:agent:coverage+{:04}", seed % 10_000),
        _ => unreachable!("did mutation slot is bounded"),
    }
}

fn classify_envelope(result: &Result<(), MessageEnvelopeError>) -> EnvelopeCoverageClass {
    match result {
        Ok(()) => EnvelopeCoverageClass::Accept,
        Err(MessageEnvelopeError::InvalidSenderDid(_)) => EnvelopeCoverageClass::InvalidSenderDid,
        Err(MessageEnvelopeError::InvalidRecipientDid(_)) => {
            EnvelopeCoverageClass::InvalidRecipientDid
        }
        Err(MessageEnvelopeError::InvalidMessageType(_)) => {
            EnvelopeCoverageClass::InvalidMessageType
        }
        Err(MessageEnvelopeError::InvalidEncryptionAlgorithm(_)) => {
            EnvelopeCoverageClass::InvalidEncryptionAlgorithm
        }
        Err(MessageEnvelopeError::EmptyBody) => EnvelopeCoverageClass::EmptyBody,
        Err(MessageEnvelopeError::InvalidProofPurpose(_)) => {
            EnvelopeCoverageClass::InvalidProofPurpose
        }
        Err(MessageEnvelopeError::ProofVerificationMethodMismatch { .. }) => {
            EnvelopeCoverageClass::ProofVerificationMethodMismatch
        }
        Err(_) => EnvelopeCoverageClass::OtherRejected,
    }
}

fn classify_did(result: &Result<AgentDid, AgentDidError>) -> DidCoverageClass {
    match result {
        Ok(_) => DidCoverageClass::Accept,
        Err(AgentDidError::InvalidPrefix(_)) => DidCoverageClass::InvalidPrefix,
        Err(AgentDidError::MissingMethodSpecificId) => DidCoverageClass::MissingMethodSpecificId,
        Err(AgentDidError::InvalidCharacter(_)) => DidCoverageClass::InvalidCharacter,
    }
}

fn expected_envelope_classes() -> BTreeSet<EnvelopeCoverageClass> {
    [
        EnvelopeCoverageClass::Accept,
        EnvelopeCoverageClass::InvalidSenderDid,
        EnvelopeCoverageClass::InvalidRecipientDid,
        EnvelopeCoverageClass::InvalidMessageType,
        EnvelopeCoverageClass::InvalidEncryptionAlgorithm,
        EnvelopeCoverageClass::EmptyBody,
        EnvelopeCoverageClass::InvalidProofPurpose,
        EnvelopeCoverageClass::ProofVerificationMethodMismatch,
    ]
    .into_iter()
    .collect()
}

fn expected_did_classes() -> BTreeSet<DidCoverageClass> {
    [
        DidCoverageClass::Accept,
        DidCoverageClass::InvalidPrefix,
        DidCoverageClass::MissingMethodSpecificId,
        DidCoverageClass::InvalidCharacter,
    ]
    .into_iter()
    .collect()
}

fn envelope_coverage_for_seed_slice(seeds: &[u64]) -> BTreeSet<EnvelopeCoverageClass> {
    let mut covered = BTreeSet::new();
    for seed in seeds {
        let result = envelope_from_seed(*seed).validate();
        covered.insert(classify_envelope(&result));
    }
    covered
}

fn did_coverage_for_seed_slice(seeds: &[u64]) -> BTreeSet<DidCoverageClass> {
    let mut covered = BTreeSet::new();
    for seed in seeds {
        let did = did_from_seed(*seed);
        let result = AgentDid::parse(&did);
        covered.insert(classify_did(&result));
    }
    covered
}

fn discover_envelope_frontier(max_seed: u64) -> (Vec<u64>, BTreeSet<EnvelopeCoverageClass>) {
    let mut frontier = Vec::new();
    let mut covered = BTreeSet::new();
    let expected = expected_envelope_classes();
    for seed in 0_u64..max_seed {
        let class = classify_envelope(&envelope_from_seed(seed).validate());
        if covered.insert(class) {
            frontier.push(seed);
            if covered == expected {
                break;
            }
        }
    }
    (frontier, covered)
}

fn discover_did_frontier(max_seed: u64) -> (Vec<u64>, BTreeSet<DidCoverageClass>) {
    let mut frontier = Vec::new();
    let mut covered = BTreeSet::new();
    let expected = expected_did_classes();
    for seed in 0_u64..max_seed {
        let did = did_from_seed(seed);
        let class = classify_did(&AgentDid::parse(&did));
        if covered.insert(class) {
            frontier.push(seed);
            if covered == expected {
                break;
            }
        }
    }
    (frontier, covered)
}

fn minimize_envelope_seed_prefix(
    seeds: &[u64],
    expected: &BTreeSet<EnvelopeCoverageClass>,
) -> Vec<u64> {
    for end in 1..=seeds.len() {
        let prefix = &seeds[..end];
        if envelope_coverage_for_seed_slice(prefix) == *expected {
            return prefix.to_vec();
        }
    }
    seeds.to_vec()
}

fn minimize_did_seed_prefix(seeds: &[u64], expected: &BTreeSet<DidCoverageClass>) -> Vec<u64> {
    for end in 1..=seeds.len() {
        let prefix = &seeds[..end];
        if did_coverage_for_seed_slice(prefix) == *expected {
            return prefix.to_vec();
        }
    }
    seeds.to_vec()
}

#[test]
fn unit_input_mutation_coverage_guided_envelope_seed_corpus_covers_boundary_classes() {
    let expected = expected_envelope_classes();
    let (frontier, covered) = discover_envelope_frontier(256);
    assert_eq!(
        covered, expected,
        "envelope coverage frontier is incomplete"
    );
    assert!(
        !frontier.is_empty(),
        "coverage-guided envelope frontier must not be empty"
    );
}

#[test]
fn unit_input_mutation_coverage_guided_did_seed_corpus_covers_boundary_classes() {
    let expected = expected_did_classes();
    let (frontier, covered) = discover_did_frontier(256);
    assert_eq!(covered, expected, "did coverage frontier is incomplete");
    assert!(
        !frontier.is_empty(),
        "coverage-guided did frontier must not be empty"
    );
}

#[test]
fn functional_input_mutation_coverage_guided_targets_are_deterministic() {
    let (first_envelope_frontier, first_envelope_covered) = discover_envelope_frontier(256);
    let (second_envelope_frontier, second_envelope_covered) = discover_envelope_frontier(256);
    assert_eq!(first_envelope_frontier, second_envelope_frontier);
    assert_eq!(first_envelope_covered, second_envelope_covered);

    let (first_did_frontier, first_did_covered) = discover_did_frontier(256);
    let (second_did_frontier, second_did_covered) = discover_did_frontier(256);
    assert_eq!(first_did_frontier, second_did_frontier);
    assert_eq!(first_did_covered, second_did_covered);

    let minimized_envelope =
        minimize_envelope_seed_prefix(&first_envelope_frontier, &expected_envelope_classes());
    let minimized_did = minimize_did_seed_prefix(&first_did_frontier, &expected_did_classes());
    assert!(minimized_envelope.len() <= first_envelope_frontier.len());
    assert!(minimized_did.len() <= first_did_frontier.len());
}

#[test]
fn integration_input_mutation_coverage_guided_reason_taxonomy_stable() {
    let (envelope_frontier, _) = discover_envelope_frontier(256);
    let (did_frontier, _) = discover_did_frontier(256);
    let minimized_envelope =
        minimize_envelope_seed_prefix(&envelope_frontier, &expected_envelope_classes());
    let minimized_did = minimize_did_seed_prefix(&did_frontier, &expected_did_classes());

    for seed in minimized_envelope {
        let first = envelope_from_seed(seed).validate();
        let second = envelope_from_seed(seed).validate();
        assert_eq!(
            first, second,
            "envelope validation reason drifted for seed {seed}"
        );
        if let Err(error) = first {
            let reason = error.to_string();
            assert!(
                !reason.trim().is_empty(),
                "envelope fail-closed reason must remain explicit for seed {seed}"
            );
        }
    }

    for seed in minimized_did {
        let did = did_from_seed(seed);
        let first = AgentDid::parse(&did);
        let second = AgentDid::parse(&did);
        assert_eq!(first, second, "did parse reason drifted for seed {seed}");
        if let Err(error) = first {
            let reason = error.to_string();
            assert!(
                !reason.trim().is_empty(),
                "did fail-closed reason must remain explicit for seed {seed}"
            );
        }
    }
}

#[test]
fn regression_input_mutation_coverage_guided_seed_corpus_contains_known_malformed_classes() {
    // Regression: #2693
    let (envelope_frontier, envelope_covered) = discover_envelope_frontier(256);
    let (did_frontier, did_covered) = discover_did_frontier(256);

    let minimized_envelope =
        minimize_envelope_seed_prefix(&envelope_frontier, &expected_envelope_classes());
    let minimized_did = minimize_did_seed_prefix(&did_frontier, &expected_did_classes());

    assert!(
        envelope_covered.contains(&EnvelopeCoverageClass::InvalidSenderDid),
        "envelope coverage-guided corpus must include malformed sender boundary class"
    );
    assert!(
        envelope_covered.contains(&EnvelopeCoverageClass::ProofVerificationMethodMismatch),
        "envelope coverage-guided corpus must include tampered proof-binding boundary class"
    );
    assert!(
        did_covered.contains(&DidCoverageClass::InvalidPrefix),
        "did coverage-guided corpus must include method mismatch prefix boundary class"
    );
    assert!(
        did_covered.contains(&DidCoverageClass::InvalidCharacter),
        "did coverage-guided corpus must include encoding/character boundary class"
    );
    assert!(
        !minimized_envelope.is_empty(),
        "envelope minimizer must return a bounded replay prefix"
    );
    assert!(
        !minimized_did.is_empty(),
        "did minimizer must return a bounded replay prefix"
    );
}

#[test]
fn performance_input_mutation_coverage_guided_contract_lane_stays_within_budget() {
    let started = Instant::now();
    let (envelope_frontier, _) = discover_envelope_frontier(1024);
    let (did_frontier, _) = discover_did_frontier(1024);
    let minimized_envelope =
        minimize_envelope_seed_prefix(&envelope_frontier, &expected_envelope_classes());
    let minimized_did = minimize_did_seed_prefix(&did_frontier, &expected_did_classes());

    assert!(!envelope_frontier.is_empty());
    assert!(!did_frontier.is_empty());
    assert!(!minimized_envelope.is_empty());
    assert!(!minimized_did.is_empty());

    let elapsed_millis = started.elapsed().as_millis();
    assert!(
        elapsed_millis < 450,
        "coverage-guided contract lane exceeded budget: {elapsed_millis}ms"
    );
}

#[test]
fn performance_input_mutation_coverage_guided_deep_lane_stress() {
    if env::var("KAMN_KOLME_LOCAL_HEAVY").ok().as_deref() != Some("1") {
        eprintln!(
            "skipping deep-lane coverage-guided stress test; set KAMN_KOLME_LOCAL_HEAVY=1 to run"
        );
        return;
    }

    let started = Instant::now();
    let (envelope_frontier, _) = discover_envelope_frontier(16_384);
    let (did_frontier, _) = discover_did_frontier(16_384);
    let minimized_envelope =
        minimize_envelope_seed_prefix(&envelope_frontier, &expected_envelope_classes());
    let minimized_did = minimize_did_seed_prefix(&did_frontier, &expected_did_classes());

    assert!(!envelope_frontier.is_empty());
    assert!(!did_frontier.is_empty());
    assert!(!minimized_envelope.is_empty());
    assert!(!minimized_did.is_empty());

    let elapsed_millis = started.elapsed().as_millis();
    assert!(
        elapsed_millis < 2_500,
        "coverage-guided deep lane exceeded budget: {elapsed_millis}ms"
    );
}
