use super::super::*;

pub(super) fn listener_quorum_evaluator(required: usize) -> ListenerQuorumEvaluator {
    ListenerQuorumEvaluator::new(required).expect("listener quorum evaluator should build")
}

pub(super) fn listener_attestation(
    listener_did: &str,
    attestation_id: &str,
) -> ListenerAttestation {
    ListenerAttestation::new(listener_did, attestation_id).expect("valid attestation")
}

pub(super) fn listener_input(
    event_id: &str,
    sequence: u64,
    attestations: Vec<ListenerAttestation>,
) -> ListenerQuorumInput {
    ListenerQuorumInput::new(event_id, sequence, attestations).expect("valid listener quorum input")
}
