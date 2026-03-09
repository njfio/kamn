const AUTHENTICATED_PEER_FRAME_TEST_SURFACE: &str =
    include_str!("authenticated_peer_frame_integration.rs");

#[test]
fn contract_authenticated_peer_frame_surface_exists_with_expected_markers() {
    for marker in [
        "integration_authenticated_peer_frame_valid_signed_roundtrip_and_inbound_validation",
        "integration_authenticated_peer_frame_invalid_inputs_fail_closed",
        "integration_authenticated_peer_frame_authenticator_rejects_wrong_recipient_unauthorized_sender_and_replay",
    ] {
        assert!(
            AUTHENTICATED_PEER_FRAME_TEST_SURFACE.contains(marker),
            "missing marker: {marker}"
        );
    }
}
