use super::super::*;

pub(super) fn peer_authenticator() -> PeerFrameAuthenticator {
    PeerFrameAuthenticator::new(
        "kamn:did:agent:peer-b",
        vec!["kamn:did:agent:peer-a".to_owned()],
    )
    .expect("authenticator should build")
}

pub(super) fn signed_peer_frame(
    frame_id: &str,
    nonce: u64,
    payload: &str,
) -> AuthenticatedPeerFrame {
    AuthenticatedPeerFrame::signed(
        frame_id,
        "kamn:did:agent:peer-a",
        "kamn:did:agent:peer-b",
        nonce,
        payload,
    )
    .expect("frame should build")
}

pub(super) fn peer_frame_budget() -> u128 {
    std::env::var("KAMN_RUNTIME_PEER_FRAME_VALIDATION_BUDGET_MS")
        .ok()
        .and_then(|raw| raw.parse::<u128>().ok())
        .unwrap_or(2_500)
}
