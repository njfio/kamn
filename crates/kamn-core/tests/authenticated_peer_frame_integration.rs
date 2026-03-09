use kamn_core::{
    AuthenticatedPeerFrame, AuthenticatedPeerFrameError, PeerFrameAuthenticator,
};

fn valid_sender() -> &'static str {
    "kamn:did:agent:peer-a"
}

fn valid_recipient() -> &'static str {
    "kamn:did:agent:peer-b"
}

#[test]
fn integration_authenticated_peer_frame_valid_signed_roundtrip_and_inbound_validation() {
    let frame = AuthenticatedPeerFrame::signed(
        "frame-1",
        valid_sender(),
        valid_recipient(),
        1,
        "payload-1",
    )
    .expect("signed frame should construct");

    let wire = frame.to_wire().expect("wire encode should succeed");
    let decoded = AuthenticatedPeerFrame::from_wire(wire.as_str())
        .expect("wire decode should succeed");

    assert_eq!(decoded, frame);
    assert_eq!(decoded.frame_id(), "frame-1");
    assert_eq!(decoded.sender_peer_did(), valid_sender());
    assert_eq!(decoded.recipient_peer_did(), valid_recipient());
    assert_eq!(decoded.nonce(), 1);
    assert_eq!(decoded.payload(), "payload-1");

    let mut authenticator = PeerFrameAuthenticator::new(
        valid_recipient(),
        vec![valid_sender().to_owned()],
    )
    .expect("authenticator should construct");
    authenticator
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

    assert_eq!(
        AuthenticatedPeerFrame::new(
            "frame-1",
            valid_sender(),
            valid_recipient(),
            1,
            "payload|bad",
            "sig-1",
        ),
        Err(AuthenticatedPeerFrameError::InvalidWireFieldDelimiter { field: "payload" })
    );

    assert_eq!(
        AuthenticatedPeerFrame::from_wire("frame|frame-1|kamn:did:agent:peer-a"),
        Err(AuthenticatedPeerFrameError::InvalidWireFormat(
            "frame|frame-1|kamn:did:agent:peer-a".to_owned(),
        ))
    );
}

#[test]
fn integration_authenticated_peer_frame_authenticator_rejects_wrong_recipient_unauthorized_sender_and_replay() {
    let wrong_recipient_frame = AuthenticatedPeerFrame::signed(
        "frame-1",
        valid_sender(),
        valid_recipient(),
        1,
        "payload-1",
    )
    .expect("signed frame should construct");
    let mut wrong_recipient_authenticator = PeerFrameAuthenticator::new(
        "kamn:did:agent:peer-c",
        vec![valid_sender().to_owned()],
    )
    .expect("authenticator should construct");
    assert_eq!(
        wrong_recipient_authenticator.validate_inbound(&wrong_recipient_frame),
        Err(AuthenticatedPeerFrameError::WrongRecipient {
            expected: "kamn:did:agent:peer-c".to_owned(),
            found: valid_recipient().to_owned(),
        })
    );

    let unauthorized_sender_frame = AuthenticatedPeerFrame::signed(
        "frame-2",
        valid_sender(),
        valid_recipient(),
        2,
        "payload-2",
    )
    .expect("signed frame should construct");
    let mut unauthorized_sender_authenticator = PeerFrameAuthenticator::new(
        valid_recipient(),
        vec!["kamn:did:agent:peer-z".to_owned()],
    )
    .expect("authenticator should construct");
    assert_eq!(
        unauthorized_sender_authenticator.validate_inbound(&unauthorized_sender_frame),
        Err(AuthenticatedPeerFrameError::UnauthorizedSender(
            valid_sender().to_owned(),
        ))
    );

    let replay_frame = AuthenticatedPeerFrame::signed(
        "frame-3",
        valid_sender(),
        valid_recipient(),
        3,
        "payload-3",
    )
    .expect("signed frame should construct");
    let mut replay_authenticator = PeerFrameAuthenticator::new(
        valid_recipient(),
        vec![valid_sender().to_owned()],
    )
    .expect("authenticator should construct");
    replay_authenticator
        .validate_inbound(&replay_frame)
        .expect("first frame should succeed");
    assert_eq!(
        replay_authenticator.validate_inbound(&replay_frame),
        Err(AuthenticatedPeerFrameError::ReplayNonce {
            sender_did: valid_sender().to_owned(),
            last_nonce: 3,
            found: 3,
        })
    );
}
