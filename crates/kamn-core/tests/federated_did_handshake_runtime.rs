use std::time::Instant;

use kamn_core::{
    FederatedDidHandshakeError, FederatedDidHandshakeEvaluator, FederatedDidHandshakeInput,
    InMemoryFederatedDidTrustStore,
};

fn input(subject_did: &str) -> FederatedDidHandshakeInput {
    FederatedDidHandshakeInput::new(
        "handshake-001",
        subject_did,
        "kolme-mainnet-a",
        "kolme-mainnet-b",
        "resolver-v1",
        true,
        true,
        false,
        true,
        2,
        2,
    )
    .expect("input should be valid")
}

#[test]
fn unit_federated_trust_store_resolver_rejects_unknown_subject() {
    let trust_store = InMemoryFederatedDidTrustStore::from_entries([(
        "kolme-mainnet-b",
        "kamn:did:agent:federated-worker-1",
    )]);
    let mut evaluator = FederatedDidHandshakeEvaluator::new(trust_store);

    let err = evaluator
        .evaluate(input("kamn:did:agent:federated-worker-2"))
        .expect_err("unknown DID should be rejected");

    assert_eq!(
        err,
        FederatedDidHandshakeError::TrustStoreMiss {
            subject_did: "kamn:did:agent:federated-worker-2".to_owned(),
            network: "kolme-mainnet-b".to_owned(),
        }
    );
    assert_eq!(
        err.reason_code(),
        "federated_did_handshake_trust_store_miss"
    );
}

#[test]
fn functional_federated_runtime_handshake_accepts_trusted_subject_with_quorum() {
    let trust_store = InMemoryFederatedDidTrustStore::from_entries([(
        "kolme-mainnet-b",
        "kamn:did:agent:federated-worker-1",
    )]);
    let mut evaluator = FederatedDidHandshakeEvaluator::new(trust_store);

    let result = evaluator
        .evaluate(input("kamn:did:agent:federated-worker-1"))
        .expect("trusted subject with satisfied quorum should pass");
    assert_eq!(result.reason_code(), "federated_did_handshake_ok");
}

#[test]
fn functional_federated_runtime_handshake_rejects_signature_policy_failure() {
    let trust_store = InMemoryFederatedDidTrustStore::from_entries([(
        "kolme-mainnet-b",
        "kamn:did:agent:federated-worker-1",
    )]);
    let mut evaluator = FederatedDidHandshakeEvaluator::new(trust_store);
    let input = FederatedDidHandshakeInput::new(
        "handshake-002",
        "kamn:did:agent:federated-worker-1",
        "kolme-mainnet-a",
        "kolme-mainnet-b",
        "resolver-v1",
        false,
        true,
        false,
        true,
        2,
        2,
    )
    .expect("input should be valid");

    let err = evaluator
        .evaluate(input)
        .expect_err("signature policy failure must reject handshake");
    assert_eq!(
        err,
        FederatedDidHandshakeError::SignaturePolicyFailed {
            handshake_id: "handshake-002".to_owned(),
        }
    );
    assert_eq!(
        err.reason_code(),
        "federated_did_handshake_signature_policy_failed"
    );
}

#[test]
fn integration_federated_runtime_handshake_rejects_quorum_shortfall() {
    let trust_store = InMemoryFederatedDidTrustStore::from_entries([(
        "kolme-mainnet-b",
        "kamn:did:agent:federated-worker-1",
    )]);
    let mut evaluator = FederatedDidHandshakeEvaluator::new(trust_store);
    let input = FederatedDidHandshakeInput::new(
        "handshake-003",
        "kamn:did:agent:federated-worker-1",
        "kolme-mainnet-a",
        "kolme-mainnet-b",
        "resolver-v1",
        true,
        true,
        false,
        true,
        3,
        2,
    )
    .expect("input should be valid");

    let err = evaluator
        .evaluate(input)
        .expect_err("quorum shortfall should fail closed");
    assert_eq!(
        err,
        FederatedDidHandshakeError::QuorumShortfall {
            required: 3,
            received: 2,
        }
    );
    assert_eq!(
        err.reason_code(),
        "federated_did_handshake_quorum_shortfall"
    );
}

#[test]
fn regression_federated_runtime_handshake_rejects_untrusted_subject_even_with_quorum() {
    let trust_store = InMemoryFederatedDidTrustStore::from_entries([(
        "kolme-mainnet-b",
        "kamn:did:agent:another-worker",
    )]);
    let mut evaluator = FederatedDidHandshakeEvaluator::new(trust_store);

    let err = evaluator
        .evaluate(input("kamn:did:agent:federated-worker-1"))
        .expect_err("trust-store bypass must remain rejected");

    // Regression: #1002
    assert_eq!(
        err.reason_code(),
        "federated_did_handshake_trust_store_miss"
    );
}

#[test]
fn performance_federated_runtime_handshake_contract_lane_stays_within_budget() {
    let trust_store = InMemoryFederatedDidTrustStore::from_entries([(
        "kolme-mainnet-b",
        "kamn:did:agent:federated-worker-1",
    )]);
    let mut evaluator = FederatedDidHandshakeEvaluator::new(trust_store);
    let start = Instant::now();

    for _ in 0..5_000 {
        evaluator
            .evaluate(input("kamn:did:agent:federated-worker-1"))
            .expect("trusted handshake should pass");
    }

    // Keep runtime deterministic and cheap in PR lanes.
    assert!(
        start.elapsed().as_millis() < 1_000,
        "runtime handshake evaluator exceeded contract lane budget"
    );
}
