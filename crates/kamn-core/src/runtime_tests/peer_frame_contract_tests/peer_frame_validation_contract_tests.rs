use super::super::*;
use super::support::{peer_authenticator, signed_peer_frame};

#[test]
fn unit_authenticated_peer_frame_rejects_invalid_wire_format() {
    assert_eq!(
        AuthenticatedPeerFrame::from_wire("frame|broken"),
        Err(AuthenticatedPeerFrameError::InvalidWireFormat(
            "frame|broken".to_owned()
        ))
    );
}

#[test]
fn functional_authenticated_peer_frame_roundtrips_wire_and_signature() {
    let frame = AuthenticatedPeerFrame::signed(
        "frame-1",
        "kamn:did:agent:peer-a",
        "kamn:did:agent:peer-b",
        1,
        "payload-1",
    )
    .expect("signed frame should build");
    let wire = frame.to_wire().expect("wire encode should pass");
    let decoded = AuthenticatedPeerFrame::from_wire(&wire).expect("wire decode should pass");
    decoded
        .verify_signature()
        .expect("signature verification should pass");
    assert_eq!(decoded, frame);
}

#[test]
fn regression_authenticated_peer_frame_signed_uses_crypto_signature_profile() {
    let frame = AuthenticatedPeerFrame::signed(
        "frame-crypto",
        "kamn:did:agent:peer-a",
        "kamn:did:agent:peer-b",
        7,
        "payload-crypto",
    )
    .expect("signed frame should build");
    assert!(
        frame.signature().starts_with("sig:secp256k1:baseline-v2:"),
        "runtime peer frame signature must use cryptographic service-auth profile: {}",
        frame.signature()
    );
}

#[test]
fn regression_authenticated_peer_frame_rejects_legacy_deterministic_signature_fixture() {
    let legacy_signature = baseline_signature_for_fields(
        "kamn:did:agent:peer-a",
        5,
        "kamn:did:agent:peer-b",
        "payload-legacy",
    );
    let frame = AuthenticatedPeerFrame::new(
        "frame-legacy",
        "kamn:did:agent:peer-a",
        "kamn:did:agent:peer-b",
        5,
        "payload-legacy",
        legacy_signature.as_str(),
    )
    .expect("legacy fixture frame should parse");
    assert!(matches!(
        frame.verify_signature(),
        Err(AuthenticatedPeerFrameError::SignatureMismatch { .. })
    ));
}

#[test]
fn integration_peer_frame_authenticator_accepts_monotonic_nonce_flow() {
    let mut authenticator = peer_authenticator();
    let frame_1 = signed_peer_frame("frame-1", 1, "payload-1");
    let frame_2 = signed_peer_frame("frame-2", 2, "payload-2");
    assert!(authenticator.validate_inbound(&frame_1).is_ok());
    assert!(authenticator.validate_inbound(&frame_2).is_ok());
}

#[test]
fn regression_forged_or_unauthorized_peer_frame_is_rejected() {
    let mut authenticator = peer_authenticator();
    let forged = tampered_peer_frame();
    assert!(matches!(
        authenticator.validate_inbound(&forged),
        Err(AuthenticatedPeerFrameError::SignatureMismatch { .. })
    ));

    let unauthorized = unauthorized_peer_frame();
    assert_eq!(
        authenticator.validate_inbound(&unauthorized),
        Err(AuthenticatedPeerFrameError::UnauthorizedSender(
            "kamn:did:agent:peer-z".to_owned()
        ))
    );
}

fn tampered_peer_frame() -> AuthenticatedPeerFrame {
    AuthenticatedPeerFrame::new(
        "frame-1",
        "kamn:did:agent:peer-a",
        "kamn:did:agent:peer-b",
        1,
        "payload-1",
        "tampered-signature",
    )
    .expect("frame should build")
}

fn unauthorized_peer_frame() -> AuthenticatedPeerFrame {
    AuthenticatedPeerFrame::signed(
        "frame-2",
        "kamn:did:agent:peer-z",
        "kamn:did:agent:peer-b",
        1,
        "payload-2",
    )
    .expect("frame should build")
}

#[test]
fn regression_replayed_peer_frame_nonce_is_rejected() {
    let mut authenticator = peer_authenticator();
    let frame = signed_peer_frame("frame-1", 1, "payload-1");
    authenticator
        .validate_inbound(&frame)
        .expect("first frame should be accepted");
    assert_eq!(
        authenticator.validate_inbound(&frame),
        Err(AuthenticatedPeerFrameError::ReplayNonce {
            sender_did: "kamn:did:agent:peer-a".to_owned(),
            last_nonce: 1,
            found: 1
        })
    );
}
