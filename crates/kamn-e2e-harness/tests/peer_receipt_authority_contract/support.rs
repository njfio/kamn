use kamn_e2e_harness::{
    peer_approval_digest, peer_challenge_digest, peer_request_digest, peer_result_digest,
    peer_settlement_digest, PeerApprovalAuthority, PeerChallengeAuthority,
    PeerReceiptAuthorityAttempt, PeerRequestAuthority, PeerResultAuthority,
    PeerSettlementAuthority, PeerSettlementVisibility,
};

pub fn request_mut(value: &mut PeerReceiptAuthorityAttempt) -> &mut PeerRequestAuthority {
    value.request.as_mut().expect("complete fixture request")
}

pub fn challenge_mut(value: &mut PeerReceiptAuthorityAttempt) -> &mut PeerChallengeAuthority {
    value
        .challenge
        .as_mut()
        .expect("complete fixture challenge")
}

pub fn approval_mut(value: &mut PeerReceiptAuthorityAttempt) -> &mut PeerApprovalAuthority {
    value.approval.as_mut().expect("complete fixture approval")
}

pub fn settlement_mut(value: &mut PeerReceiptAuthorityAttempt) -> &mut PeerSettlementAuthority {
    value
        .settlement
        .as_mut()
        .expect("complete fixture settlement")
}

pub fn result_mut(value: &mut PeerReceiptAuthorityAttempt) -> &mut PeerResultAuthority {
    value
        .service_result
        .as_mut()
        .expect("complete fixture service result")
}

pub fn complete_attempt() -> PeerReceiptAuthorityAttempt {
    let request = request();
    let challenge = challenge(request.request_digest.as_str());
    let approval = approval(&challenge);
    let settlement = settlement(&approval);
    let service_result = service_result(&request, &settlement);
    PeerReceiptAuthorityAttempt {
        request: Some(request),
        challenge: Some(challenge),
        approval: Some(approval),
        settlement: Some(settlement),
        service_result: Some(service_result),
        settlement_visibility: PeerSettlementVisibility::Observed,
    }
}

pub fn blocked_attempt() -> PeerReceiptAuthorityAttempt {
    let mut attempt = complete_attempt();
    let challenge = attempt.challenge.as_mut().expect("complete fixture stage");
    challenge.request_digest.clear();
    challenge.challenge_id.clear();
    challenge.nonce.clear();
    challenge.expires_at_unix = 0;
    attempt.approval = None;
    attempt.settlement = None;
    attempt.service_result = None;
    attempt.settlement_visibility = PeerSettlementVisibility::Blocked;
    attempt
}

pub fn recompute_digests(attempt: &mut PeerReceiptAuthorityAttempt) {
    let request = attempt.request.as_mut().expect("complete fixture stage");
    request.request_digest = peer_request_digest(request.canonical_body.as_str());
    let challenge = attempt.challenge.as_mut().expect("complete fixture stage");
    challenge.challenge_digest = peer_challenge_digest(challenge);
    let approval = attempt.approval.as_mut().expect("complete fixture stage");
    approval.approval_digest = peer_approval_digest(approval);
    let settlement = attempt.settlement.as_mut().expect("complete fixture stage");
    settlement.settlement_digest = peer_settlement_digest(settlement);
    let result = attempt
        .service_result
        .as_mut()
        .expect("complete fixture stage");
    result.result_digest = peer_result_digest(result);
}

fn request() -> PeerRequestAuthority {
    let canonical_body = r#"{"prompt":"deliver artifact"}"#.to_owned();
    let request_digest = peer_request_digest(canonical_body.as_str());
    PeerRequestAuthority {
        canonical_body,
        request_digest,
    }
}

fn challenge(request_digest: &str) -> PeerChallengeAuthority {
    let mut value = PeerChallengeAuthority {
        request_digest: request_digest.into(),
        challenge_id: "challenge-7171".into(),
        nonce: "nonce-7171".into(),
        expires_at_unix: 1_800_000_000,
        payer: "did:peer:payer".into(),
        payee: "wallet-payee".into(),
        asset: "SOL".into(),
        network: "solana:devnet".into(),
        amount_minor: 1_000,
        challenge_digest: String::new(),
    };
    value.challenge_digest = peer_challenge_digest(&value);
    value
}

fn approval(challenge: &PeerChallengeAuthority) -> PeerApprovalAuthority {
    let mut value = PeerApprovalAuthority {
        request_digest: challenge.request_digest.clone(),
        challenge_digest: challenge.challenge_digest.clone(),
        challenge_id: challenge.challenge_id.clone(),
        nonce: challenge.nonce.clone(),
        approved_at_unix: 1_700_000_100,
        payer: challenge.payer.clone(),
        payee: challenge.payee.clone(),
        asset: challenge.asset.clone(),
        network: challenge.network.clone(),
        amount_minor: challenge.amount_minor,
        approval_digest: String::new(),
    };
    value.approval_digest = peer_approval_digest(&value);
    value
}

fn settlement(approval: &PeerApprovalAuthority) -> PeerSettlementAuthority {
    let mut value = PeerSettlementAuthority {
        request_digest: approval.request_digest.clone(),
        challenge_digest: approval.challenge_digest.clone(),
        approval_digest: approval.approval_digest.clone(),
        receipt_id: "receipt-7171".into(),
        transaction_id: "transaction-7171".into(),
        finalized_at_unix: 1_700_000_200,
        payer: approval.payer.clone(),
        payee: approval.payee.clone(),
        asset: approval.asset.clone(),
        network: approval.network.clone(),
        amount_minor: approval.amount_minor,
        settlement_digest: String::new(),
    };
    value.settlement_digest = peer_settlement_digest(&value);
    value
}

fn service_result(
    request: &PeerRequestAuthority,
    settlement: &PeerSettlementAuthority,
) -> PeerResultAuthority {
    let mut value = PeerResultAuthority {
        request_digest: request.request_digest.clone(),
        settlement_digest: settlement.settlement_digest.clone(),
        produced_at_unix: 1_700_000_300,
        canonical_result: r#"{"status":"delivered"}"#.into(),
        result_digest: String::new(),
    };
    value.result_digest = peer_result_digest(&value);
    value
}
