use super::super::*;

#[test]
fn performance_authenticated_peer_frame_validation_stays_within_ci_budget() {
    let budget_millis = std::env::var("KAMN_RUNTIME_PEER_FRAME_VALIDATION_BUDGET_MS")
        .ok()
        .and_then(|raw| raw.parse::<u128>().ok())
        .unwrap_or(2_500);
    let mut authenticator = PeerFrameAuthenticator::new(
        "kamn:did:agent:peer-b",
        vec!["kamn:did:agent:peer-a".to_owned()],
    )
    .expect("authenticator should build");
    let started = Instant::now();
    for nonce in 1..=256 {
        let frame = AuthenticatedPeerFrame::signed(
            &format!("frame-{nonce}"),
            "kamn:did:agent:peer-a",
            "kamn:did:agent:peer-b",
            nonce,
            "payload-bounded",
        )
        .expect("frame should build");
        authenticator
            .validate_inbound(&frame)
            .expect("frame should be accepted");
    }
    let elapsed_millis = started.elapsed().as_millis();
    assert!(
        elapsed_millis <= budget_millis,
        "authenticated peer frame validation exceeded CI budget: {elapsed_millis}ms (budget={budget_millis}ms)"
    );
}
