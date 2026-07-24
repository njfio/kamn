use kamn_e2e_harness::{
    verify_peer_receipt_authority, PeerReceiptAuthorityAttempt, PeerReceiptAuthorityVerdict,
};

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

    for mutate in missing_field_mutations() {
        let mut attempt = complete_attempt();
        mutate(&mut attempt);
        assert_code(attempt, "PEER_AUTHORITY_FIELD_MISSING");
    }
}

#[test]
fn malformed_and_mismatched_digests_fail_closed() {
    let mut malformed = complete_attempt();
    malformed.request.as_mut().unwrap().request_digest = "sha256:nope".into();
    assert_code(malformed, "PEER_AUTHORITY_DIGEST_INVALID");

    for mutate in digest_mutations() {
        let mut attempt = complete_attempt();
        mutate(&mut attempt);
        assert_code(attempt, "PEER_AUTHORITY_DIGEST_MISMATCH");
    }
}

#[test]
fn identity_and_economic_mutations_fail_closed() {
    for mutate in binding_mutations() {
        let mut attempt = complete_attempt();
        mutate(&mut attempt);
        recompute_digests(&mut attempt);
        assert_code(attempt, "PEER_AUTHORITY_BINDING_MISMATCH");
    }
}

#[test]
fn expiry_and_stage_order_mutations_fail_closed() {
    for mutate in time_mutations() {
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

fn missing_field_mutations() -> Vec<fn(&mut PeerReceiptAuthorityAttempt)> {
    vec![
        |value| value.challenge.as_mut().unwrap().challenge_id.clear(),
        |value| value.challenge.as_mut().unwrap().nonce.clear(),
        |value| value.approval.as_mut().unwrap().payer.clear(),
        |value| value.settlement.as_mut().unwrap().receipt_id.clear(),
        |value| value.settlement.as_mut().unwrap().transaction_id.clear(),
        |value| {
            value
                .service_result
                .as_mut()
                .unwrap()
                .canonical_result
                .clear()
        },
    ]
}

fn digest_mutations() -> Vec<fn(&mut PeerReceiptAuthorityAttempt)> {
    vec![
        |value| {
            value
                .request
                .as_mut()
                .unwrap()
                .request_digest
                .replace_range(7..8, "0")
        },
        |value| {
            value
                .challenge
                .as_mut()
                .unwrap()
                .challenge_digest
                .replace_range(7..8, "0")
        },
        |value| {
            value
                .approval
                .as_mut()
                .unwrap()
                .approval_digest
                .replace_range(7..8, "0")
        },
        |value| {
            value
                .settlement
                .as_mut()
                .unwrap()
                .settlement_digest
                .replace_range(7..8, "0")
        },
        |value| {
            value
                .service_result
                .as_mut()
                .unwrap()
                .result_digest
                .replace_range(7..8, "0")
        },
    ]
}

fn binding_mutations() -> Vec<fn(&mut PeerReceiptAuthorityAttempt)> {
    vec![
        |value| value.approval.as_mut().unwrap().request_digest = digest('f'),
        |value| value.approval.as_mut().unwrap().challenge_id = "challenge-other".into(),
        |value| value.approval.as_mut().unwrap().nonce = "nonce-other".into(),
        |value| value.approval.as_mut().unwrap().payer = "did:peer:other".into(),
        |value| value.settlement.as_mut().unwrap().payee = "wallet-other".into(),
        |value| value.settlement.as_mut().unwrap().asset = "USDC".into(),
        |value| value.settlement.as_mut().unwrap().network = "solana:mainnet".into(),
        |value| value.settlement.as_mut().unwrap().amount_minor += 1,
        |value| value.service_result.as_mut().unwrap().settlement_digest = digest('e'),
    ]
}

fn time_mutations() -> Vec<fn(&mut PeerReceiptAuthorityAttempt)> {
    vec![
        |value| value.approval.as_mut().unwrap().approved_at_unix = 1_800_000_001,
        |value| value.settlement.as_mut().unwrap().finalized_at_unix = 1_700_000_099,
        |value| value.service_result.as_mut().unwrap().produced_at_unix = 1_700_000_199,
    ]
}

fn digest(character: char) -> String {
    format!("sha256:{}", character.to_string().repeat(64))
}
