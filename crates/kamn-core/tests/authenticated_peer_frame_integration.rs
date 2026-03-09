use kamn_core::{AuthenticatedPeerFrame, AuthenticatedPeerFrameError, PeerFrameAuthenticator};

fn valid_sender() -> &'static str {
    "kamn:did:agent:peer-a"
}

fn valid_recipient() -> &'static str {
    "kamn:did:agent:peer-b"
}

fn signed_frame(
    frame_id: &str,
    recipient_did: &str,
    nonce: u64,
    payload: &str,
) -> AuthenticatedPeerFrame {
    AuthenticatedPeerFrame::signed(frame_id, valid_sender(), recipient_did, nonce, payload)
        .expect("signed frame should construct")
}

fn authenticator(local_peer_did: &str, allowed_sender_dids: Vec<String>) -> PeerFrameAuthenticator {
    PeerFrameAuthenticator::new(local_peer_did, allowed_sender_dids)
        .expect("authenticator should construct")
}

fn assert_invalid_new(expected: AuthenticatedPeerFrameError, payload: &str, signature: &str) {
    assert_eq!(
        AuthenticatedPeerFrame::new(
            "frame-1",
            valid_sender(),
            valid_recipient(),
            1,
            payload,
            signature,
        ),
        Err(expected)
    );
}

#[test]
fn integration_authenticated_peer_frame_valid_signed_roundtrip_and_inbound_validation() {
    let frame = signed_frame("frame-1", valid_recipient(), 1, "payload-1");
    let wire = frame.to_wire().expect("wire encode should succeed");
    let decoded =
        AuthenticatedPeerFrame::from_wire(wire.as_str()).expect("wire decode should succeed");

    assert_eq!(decoded, frame);
    assert_eq!(decoded.frame_id(), "frame-1");
    assert_eq!(decoded.sender_peer_did(), valid_sender());
    assert_eq!(decoded.recipient_peer_did(), valid_recipient());
    assert_eq!(decoded.nonce(), 1);
    assert_eq!(decoded.payload(), "payload-1");

    authenticator(valid_recipient(), vec![valid_sender().to_owned()])
        .validate_inbound(&decoded)
        .expect("inbound validation should succeed");
}

#[test]
fn integration_authenticated_peer_frame_invalid_inputs_fail_closed() {
    assert_eq!(
        AuthenticatedPeerFrame::new(
            "frame-1",
            "bad-did",
            valid_recipient(),
            1,
            "payload-1",
            "sig-1",
        ),
        Err(AuthenticatedPeerFrameError::InvalidSenderDid {
            field: "sender_peer_did",
            reason_code: "runtime_peer_frame_invalid_sender_did",
            detail: "invalid agent did prefix: bad-did".to_owned(),
        })
    );
    assert_invalid_new(
        AuthenticatedPeerFrameError::InvalidWireFieldDelimiter { field: "payload" },
        "payload|bad",
        "sig-1",
    );
    assert_eq!(
        AuthenticatedPeerFrame::from_wire("frame|frame-1|kamn:did:agent:peer-a"),
        Err(AuthenticatedPeerFrameError::InvalidWireFormat(
            "frame|frame-1|kamn:did:agent:peer-a".to_owned(),
        ))
    );
}

#[test]
fn integration_authenticated_peer_frame_authenticator_rejects_wrong_recipient_unauthorized_sender_and_replay(
) {
    assert_wrong_recipient_rejected();
    assert_unauthorized_sender_rejected();
    assert_replay_nonce_rejected();
}

fn assert_wrong_recipient_rejected() {
    let frame = signed_frame("frame-1", valid_recipient(), 1, "payload-1");
    let mut peer_authenticator =
        authenticator("kamn:did:agent:peer-c", vec![valid_sender().to_owned()]);
    assert_eq!(
        peer_authenticator.validate_inbound(&frame),
        Err(AuthenticatedPeerFrameError::WrongRecipient {
            expected: "kamn:did:agent:peer-c".to_owned(),
            found: valid_recipient().to_owned(),
        })
    );
}

fn assert_unauthorized_sender_rejected() {
    let frame = signed_frame("frame-2", valid_recipient(), 2, "payload-2");
    let mut peer_authenticator =
        authenticator(valid_recipient(), vec!["kamn:did:agent:peer-z".to_owned()]);
    assert_eq!(
        peer_authenticator.validate_inbound(&frame),
        Err(AuthenticatedPeerFrameError::UnauthorizedSender(
            valid_sender().to_owned(),
        ))
    );
}

fn assert_replay_nonce_rejected() {
    let frame = signed_frame("frame-3", valid_recipient(), 3, "payload-3");
    let mut peer_authenticator = authenticator(valid_recipient(), vec![valid_sender().to_owned()]);
    peer_authenticator
        .validate_inbound(&frame)
        .expect("first frame should succeed");
    assert_eq!(
        peer_authenticator.validate_inbound(&frame),
        Err(AuthenticatedPeerFrameError::ReplayNonce {
            sender_did: valid_sender().to_owned(),
            last_nonce: 3,
            found: 3,
        })
    );
}
