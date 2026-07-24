use kamn_e2e_harness::{
    verify_peer_receipt_authority, PeerReceiptAuthorityAttempt, PeerReceiptAuthorityVerdict,
};

#[path = "peer_receipt_authority_contract/binding_mutations.rs"]
mod binding_mutations;
#[path = "peer_receipt_authority_contract/mutations.rs"]
mod mutations;
#[path = "peer_receipt_authority_contract/support.rs"]
mod support;

use support::{blocked_attempt, complete_attempt, recompute_digests};

#[test]
fn complete_receipt_authority_chain_passes() {
    assert_eq!(
        verify_peer_receipt_authority(&complete_attempt()),
        PeerReceiptAuthorityVerdict::Pass
    );
}

#[test]
fn missing_stages_and_fields_fail_closed() {
    let mut missing_stage = complete_attempt();
    missing_stage.approval = None;
    assert_code(missing_stage, "PEER_AUTHORITY_STAGE_MISSING");

    for mutate in mutations::missing_fields() {
        let mut attempt = complete_attempt();
        mutate(&mut attempt);
        assert_code(attempt, "PEER_AUTHORITY_FIELD_MISSING");
    }
}

#[test]
fn malformed_and_mismatched_digests_fail_closed() {
    let mut malformed = complete_attempt();
    malformed
        .request
        .as_mut()
        .expect("complete fixture stage")
        .request_digest = "sha256:nope".into();
    assert_code(malformed, "PEER_AUTHORITY_DIGEST_INVALID");

    for mutate in mutations::digests() {
        let mut attempt = complete_attempt();
        mutate(&mut attempt);
        assert_code(attempt, "PEER_AUTHORITY_DIGEST_MISMATCH");
    }
}

#[test]
fn identity_and_economic_mutations_fail_closed() {
    for mutate in binding_mutations::bindings() {
        let mut attempt = complete_attempt();
        mutate(&mut attempt);
        recompute_digests(&mut attempt);
        assert_code(attempt, "PEER_AUTHORITY_BINDING_MISMATCH");
    }
}

#[test]
fn expiry_and_stage_order_mutations_fail_closed() {
    for mutate in binding_mutations::times() {
        let mut attempt = complete_attempt();
        mutate(&mut attempt);
        recompute_digests(&mut attempt);
        assert_code(attempt, "PEER_AUTHORITY_TIME_INVALID");
    }
}

#[test]
fn partial_no_funds_observation_is_blocked_and_never_passes() {
    let verdict = verify_peer_receipt_authority(&blocked_attempt());
    match verdict {
        PeerReceiptAuthorityVerdict::Blocked(error) => {
            assert_eq!(error.code, "PEER_AUTHORITY_FIELD_MISSING");
            assert_eq!(error.stage, "challenge");
        }
        other => panic!("expected blocked verdict, got {other:?}"),
    }
}

fn assert_code(attempt: PeerReceiptAuthorityAttempt, expected: &str) {
    match verify_peer_receipt_authority(&attempt) {
        PeerReceiptAuthorityVerdict::Fail(error) | PeerReceiptAuthorityVerdict::Blocked(error) => {
            assert_eq!(error.code, expected)
        }
        PeerReceiptAuthorityVerdict::Pass => panic!("expected {expected}, got pass"),
    }
}
