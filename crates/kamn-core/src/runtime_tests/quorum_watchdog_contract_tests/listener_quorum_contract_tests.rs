use super::super::*;
use super::support::{listener_attestation, listener_input, listener_quorum_evaluator};

#[test]
fn functional_listener_quorum_accepts_canonical_sufficient_attestations() {
    let mut evaluator = listener_quorum_evaluator(2);
    let input = listener_input(
        "bridge-event-1",
        1,
        vec![
            listener_attestation("kamn:did:agent:listener-b", "att-2"),
            listener_attestation("kamn:did:agent:listener-a", "att-1"),
        ],
    );
    let decision = evaluator
        .evaluate(input)
        .expect("quorum should accept canonical listener attestations");
    assert!(decision.accepted);
    assert_eq!(decision.required_confirmations, 2);
    assert_eq!(decision.confirmed_listeners.len(), 2);
    assert_eq!(
        decision.confirmed_listeners,
        vec![
            "kamn:did:agent:listener-a".to_owned(),
            "kamn:did:agent:listener-b".to_owned()
        ]
    );
}

#[test]
fn unit_listener_quorum_rejects_zero_required_confirmations() {
    let error =
        ListenerQuorumEvaluator::new(0).expect_err("zero quorum threshold must be rejected");
    assert_eq!(
        error,
        ListenerQuorumError::InvalidRequiredConfirmations { required: 0 }
    );
}

#[test]
fn integration_daemon_listener_quorum_rejects_replayed_event_sequence() {
    let mut evaluator = listener_quorum_evaluator(1);
    let first = listener_input(
        "bridge-event-1",
        3,
        vec![listener_attestation("kamn:did:agent:listener-a", "att-1")],
    );
    assert!(super::super::evaluate_daemon_listener_quorum(&mut evaluator, first).is_ok());

    let replay = listener_input(
        "bridge-event-1",
        3,
        vec![listener_attestation("kamn:did:agent:listener-a", "att-2")],
    );
    let error = super::super::evaluate_daemon_listener_quorum(&mut evaluator, replay)
        .expect_err("replayed sequence should be rejected");
    assert_eq!(
        error,
        ListenerQuorumError::ReplayedEventSequence {
            event_id: "bridge-event-1".to_owned(),
            previous_sequence: 3,
            received_sequence: 3
        }
    );
}

#[test]
fn regression_duplicate_listener_attestation_replay_is_rejected() {
    let mut evaluator = listener_quorum_evaluator(2);
    let input = listener_input(
        "bridge-event-dup",
        1,
        vec![
            listener_attestation("kamn:did:agent:listener-a", "att-1"),
            listener_attestation("kamn:did:agent:listener-a", "att-2"),
        ],
    );
    let error = evaluator
        .evaluate(input)
        .expect_err("duplicate listener attestations must be rejected");
    assert_eq!(
        error,
        ListenerQuorumError::DuplicateListenerAttestation {
            listener_did: "kamn:did:agent:listener-a".to_owned()
        }
    );
}

#[test]
fn regression_replayed_listener_event_sequence_is_rejected() {
    let mut evaluator = listener_quorum_evaluator(1);
    let first = listener_input(
        "bridge-event-regression",
        7,
        vec![listener_attestation("kamn:did:agent:listener-a", "att-1")],
    );
    assert!(evaluator.evaluate(first).is_ok());

    let replay = listener_input(
        "bridge-event-regression",
        6,
        vec![listener_attestation("kamn:did:agent:listener-a", "att-2")],
    );
    let error = evaluator
        .evaluate(replay)
        .expect_err("stale/replayed sequence must be rejected");
    assert_eq!(
        error,
        replayed_listener_sequence_error("bridge-event-regression", 7, 6)
    );
}

fn replayed_listener_sequence_error(
    event_id: &str,
    previous_sequence: u64,
    received_sequence: u64,
) -> ListenerQuorumError {
    ListenerQuorumError::ReplayedEventSequence {
        event_id: event_id.to_owned(),
        previous_sequence,
        received_sequence,
    }
}
