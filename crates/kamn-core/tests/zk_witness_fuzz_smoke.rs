use kamn_core::{
    build_message_witness, AttachmentRef, CanonicalMessageEnvelope, EnvelopeEncryption,
    EnvelopeHeader, EnvelopeMetadata, EnvelopeProof, MessageEnvelopeError, ZkDesignError,
};
use std::collections::BTreeMap;
use std::panic::catch_unwind;
use std::time::Instant;

fn base_body() -> BTreeMap<String, String> {
    let mut body = BTreeMap::new();
    body.insert("task.type".to_owned(), "analysis".to_owned());
    body.insert(
        "task.description".to_owned(),
        "generated witness payload".to_owned(),
    );
    body.insert("task.customer".to_owned(), "acme".to_owned());
    body
}

fn valid_envelope() -> CanonicalMessageEnvelope {
    CanonicalMessageEnvelope {
        envelope: EnvelopeMetadata {
            id: "urn:uuid:zk-fuzz-smoke".to_owned(),
            type_name: "kamn:message:v1".to_owned(),
            from: "kamn:did:agent:sender-1".to_owned(),
            to: vec!["kamn:did:agent:recipient-1".to_owned()],
            created: "2026-02-10T00:00:00.000Z".to_owned(),
            expires: "2026-02-10T00:10:00.000Z".to_owned(),
            thread_id: Some("urn:uuid:zk-fuzz-thread".to_owned()),
            parent_id: None,
            nonce: 17,
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
            uri: "ipfs://bafybeizkwitness".to_owned(),
        }],
        proof: EnvelopeProof {
            type_name: "Ed25519Signature2020".to_owned(),
            created: "2026-02-10T00:00:00.000Z".to_owned(),
            verification_method: "kamn:did:agent:sender-1#keys-1".to_owned(),
            proof_purpose: "authentication".to_owned(),
            proof_value: "z58DAdFfa9SkqZMVPxAQp".to_owned(),
        },
    }
}

fn mutation_slot(seed: u64, slots: usize) -> usize {
    let mixed = seed
        .wrapping_mul(11400714819323198485)
        .wrapping_add(14029467366897019727);
    (mixed % slots as u64) as usize
}

fn mutate_selector(seed: u64) -> String {
    match mutation_slot(seed, 8) {
        0 => "task.description".to_owned(),
        1 => "task.type".to_owned(),
        2 => "task.customer".to_owned(),
        3 => "task..description".to_owned(),
        4 => "task description".to_owned(),
        5 => ".task.description".to_owned(),
        6 => "task.description.".to_owned(),
        7 => "task.unknown".to_owned(),
        _ => unreachable!("selector slot is bounded"),
    }
}

fn mutate_envelope_case(
    mut envelope: CanonicalMessageEnvelope,
    seed: u64,
) -> CanonicalMessageEnvelope {
    match mutation_slot(seed.rotate_left(9), 6) {
        0 => envelope,
        1 => {
            envelope.envelope.type_name = "kamn:message:v2".to_owned();
            envelope
        }
        2 => {
            envelope.envelope.from = "did:example:sender".to_owned();
            envelope
        }
        3 => {
            envelope.body.remove("task.description");
            envelope
        }
        4 => {
            envelope.header.message_type = "UnknownType".to_owned();
            envelope
        }
        5 => {
            envelope.proof.verification_method = "kamn:did:agent:attacker-1#keys-9".to_owned();
            envelope
        }
        _ => unreachable!("envelope slot is bounded"),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ZkWitnessMutationClass {
    MalformedSelector,
    MissingField,
    TamperedEnvelope,
}

#[derive(Debug, Clone)]
struct ZkWitnessMutationCase {
    id: &'static str,
    class: ZkWitnessMutationClass,
    envelope: CanonicalMessageEnvelope,
    selector: &'static str,
    expected_error: ZkDesignError,
}

fn deterministic_zk_witness_mutation_cases() -> Vec<ZkWitnessMutationCase> {
    let mut tampered_envelope = valid_envelope();
    tampered_envelope.envelope.type_name = "kamn:message:v2".to_owned();

    vec![
        ZkWitnessMutationCase {
            id: "malformed-selector-whitespace",
            class: ZkWitnessMutationClass::MalformedSelector,
            envelope: valid_envelope(),
            selector: "task description",
            expected_error: ZkDesignError::InvalidPrivateField(
                "private field selector `task description` must contain only [A-Za-z0-9_.-] and no empty path segments".to_owned(),
            ),
        },
        ZkWitnessMutationCase {
            id: "missing-selector-field",
            class: ZkWitnessMutationClass::MissingField,
            envelope: valid_envelope(),
            selector: "task.unknown",
            expected_error: ZkDesignError::MissingPrivateField("task.unknown".to_owned()),
        },
        ZkWitnessMutationCase {
            id: "tampered-envelope-type",
            class: ZkWitnessMutationClass::TamperedEnvelope,
            envelope: tampered_envelope,
            selector: "task.description",
            expected_error: ZkDesignError::EnvelopeError(MessageEnvelopeError::InvalidEnvelopeType(
                "kamn:message:v2".to_owned(),
            )),
        },
    ]
}

#[test]
fn fuzz_smoke_zk_witness_mutation_lane_is_panic_free_and_deterministic() {
    for seed in 0_u64..1024 {
        let envelope = mutate_envelope_case(valid_envelope(), seed);
        let selector = mutate_selector(seed);

        let first = catch_unwind(|| build_message_witness(&envelope, &[selector.as_str()]));
        assert!(first.is_ok(), "witness generation panicked for seed {seed}");

        let first = first.expect("panic-free witness result should unwrap");
        let second = build_message_witness(&envelope, &[selector.as_str()]);
        assert_eq!(
            first, second,
            "witness result should remain deterministic for mutation seed {seed}"
        );
    }
}

#[test]
fn fuzz_smoke_zk_witness_error_corpus_covers_expected_rejection_classes() {
    assert_eq!(
        build_message_witness(&valid_envelope(), &["task..description"]),
        Err(ZkDesignError::InvalidPrivateField(
            "private field selector `task..description` must contain only [A-Za-z0-9_.-] and no empty path segments".to_owned(),
        ))
    );
    assert_eq!(
        build_message_witness(&valid_envelope(), &["task description"]),
        Err(ZkDesignError::InvalidPrivateField(
            "private field selector `task description` must contain only [A-Za-z0-9_.-] and no empty path segments".to_owned(),
        ))
    );
    assert_eq!(
        build_message_witness(&valid_envelope(), &["task.unknown"]),
        Err(ZkDesignError::MissingPrivateField(
            "task.unknown".to_owned()
        ))
    );
}

#[test]
fn functional_zk_witness_mutation_suite_covers_malformed_missing_and_tampered_classes() {
    let cases = deterministic_zk_witness_mutation_cases();
    let mut saw_malformed = false;
    let mut saw_missing = false;
    let mut saw_tampered = false;

    for case in &cases {
        let actual = build_message_witness(&case.envelope, &[case.selector])
            .expect_err("mutation corpus entries must fail closed");
        assert_eq!(
            actual, case.expected_error,
            "unexpected fail-closed reason for case {}",
            case.id
        );

        match case.class {
            ZkWitnessMutationClass::MalformedSelector => saw_malformed = true,
            ZkWitnessMutationClass::MissingField => saw_missing = true,
            ZkWitnessMutationClass::TamperedEnvelope => saw_tampered = true,
        }
    }

    assert!(
        saw_malformed,
        "malformed selector mutation class must be covered"
    );
    assert!(
        saw_missing,
        "missing selector field mutation class must be covered"
    );
    assert!(
        saw_tampered,
        "tampered envelope mutation class must be covered"
    );
}

#[test]
fn integration_zk_witness_mutation_fail_closed_reasons_are_explicit_and_deterministic() {
    for case in deterministic_zk_witness_mutation_cases() {
        let first =
            build_message_witness(&case.envelope, &[case.selector]).expect_err("must fail closed");
        let second =
            build_message_witness(&case.envelope, &[case.selector]).expect_err("must fail closed");
        assert_eq!(first, second, "reason drifted for case {}", case.id);
        let reason = first.to_string();
        assert!(
            !reason.trim().is_empty(),
            "fail-closed reason must be explicit for case {}",
            case.id
        );
    }
}

#[test]
fn regression_zk_witness_mutation_reason_signatures_remain_stable() {
    // Regression: #994
    let error = build_message_witness(&valid_envelope(), &["task description"])
        .expect_err("whitespace selector must fail closed");
    assert_eq!(
        error,
        ZkDesignError::InvalidPrivateField(
            "private field selector `task description` must contain only [A-Za-z0-9_.-] and no empty path segments".to_owned(),
        )
    );
    assert_eq!(
        error.to_string(),
        "invalid private field: private field selector `task description` must contain only [A-Za-z0-9_.-] and no empty path segments"
    );
}

#[test]
fn performance_zk_witness_mutation_contract_lane_stays_within_budget() {
    let started = Instant::now();
    let mut accepted = 0_u64;
    let mut rejected = 0_u64;

    for seed in 0_u64..1536 {
        let envelope = mutate_envelope_case(valid_envelope(), seed);
        let selector = mutate_selector(seed);
        if build_message_witness(&envelope, &[selector.as_str()]).is_ok() {
            accepted += 1;
        } else {
            rejected += 1;
        }
    }

    assert!(
        accepted > 0,
        "mutation lane should retain valid witness samples"
    );
    assert!(
        rejected > 0,
        "mutation lane should reject invalid witness samples"
    );

    let elapsed_millis = started.elapsed().as_millis();
    assert!(
        elapsed_millis < 450,
        "zk witness mutation contract lane exceeded budget: {elapsed_millis}ms"
    );
}

#[test]
#[ignore = "scheduled deep mutation stress lane"]
fn performance_zk_witness_mutation_deep_lane_stress() {
    let started = Instant::now();
    for seed in 0_u64..40_000 {
        let envelope = mutate_envelope_case(valid_envelope(), seed);
        let selector = mutate_selector(seed);
        let _ = build_message_witness(&envelope, &[selector.as_str()]);
    }

    let elapsed_millis = started.elapsed().as_millis();
    assert!(
        elapsed_millis < 4_000,
        "zk witness deep mutation lane exceeded budget: {elapsed_millis}ms"
    );
}
