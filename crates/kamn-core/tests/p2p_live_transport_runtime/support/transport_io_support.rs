use super::*;

#[cfg(not(feature = "libp2p-live-transport"))]
fn send_frame(
    transport: &Libp2pLivePeerLifecycleTransport,
    sender_peer_id: &str,
    recipient_peer_id: &str,
    payload: &str,
) -> Result<(), P2pTransportError> {
    let frame = PeerGossipFrame::new("messages", sender_peer_id, recipient_peer_id, payload)
        .expect("frame should build");
    transport.send(frame)
}

#[cfg(not(feature = "libp2p-live-transport"))]
pub(crate) fn send_frames_until_error(
    transport: &Libp2pLivePeerLifecycleTransport,
    sender_peer_id: &str,
    recipient_peer_id: &str,
    payload_prefix: &str,
    range: std::ops::Range<u16>,
) -> Option<P2pTransportError> {
    for nonce in range {
        let payload = format!("{payload_prefix}-{nonce}");
        if let Err(error) = send_frame(transport, sender_peer_id, recipient_peer_id, &payload) {
            return Some(error);
        }
    }
    None
}

#[cfg(not(feature = "libp2p-live-transport"))]
pub(crate) fn send_frames_expect_success(
    transport: &Libp2pLivePeerLifecycleTransport,
    sender_peer_id: &str,
    recipient_peer_id: &str,
    payload_prefix: &str,
    range: std::ops::Range<u16>,
) {
    for nonce in range {
        let payload = format!("{payload_prefix}-{nonce}");
        send_frame(transport, sender_peer_id, recipient_peer_id, &payload)
            .expect("dispatch should continue accepting");
    }
}

pub(crate) fn send_with_retry(
    transport: &Libp2pLivePeerLifecycleTransport,
    frame: &PeerGossipFrame,
    timeout: Duration,
) -> Result<(), P2pTransportError> {
    let started = Instant::now();
    loop {
        match transport.send(frame.clone()) {
            Ok(()) => return Ok(()),
            Err(P2pTransportError::LiveSocketSendFailed) if started.elapsed() < timeout => {
                std::thread::sleep(Duration::from_millis(25));
            }
            Err(error) => return Err(error),
        }
    }
}

pub(crate) fn drain_until_count(
    transport: &Libp2pLivePeerLifecycleTransport,
    recipient_peer_id: &str,
    expected: usize,
    timeout: Duration,
) -> Vec<PeerGossipFrame> {
    let started = Instant::now();
    let mut frames = Vec::new();
    loop {
        append_drained_frames(transport, recipient_peer_id, &mut frames);
        if frames.len() >= expected {
            return frames;
        }
        assert_drain_deadline(started, timeout, expected, frames.len());
    }
}

pub(crate) fn drain_runtime_events_until(
    transport: &Libp2pLivePeerLifecycleTransport,
    expected: usize,
    timeout: Duration,
) -> Vec<kamn_core::Libp2pRuntimeEvent> {
    let started = Instant::now();
    loop {
        let drained = transport
            .drain_runtime_events()
            .expect("runtime events should drain");
        if drained.len() >= expected {
            return drained;
        }
        assert_event_deadline(started, timeout);
    }
}

fn append_drained_frames(
    transport: &Libp2pLivePeerLifecycleTransport,
    recipient_peer_id: &str,
    frames: &mut Vec<PeerGossipFrame>,
) {
    let mut drained = transport
        .drain_inbox(recipient_peer_id)
        .expect("recipient inbox should drain");
    if !drained.is_empty() {
        frames.append(&mut drained);
    }
}

fn assert_drain_deadline(started: Instant, timeout: Duration, expected: usize, current: usize) {
    assert!(
        started.elapsed() < timeout,
        "expected {expected} frames but only received {current} within {timeout:?}",
    );
    std::thread::sleep(Duration::from_millis(25));
}

fn assert_event_deadline(started: Instant, timeout: Duration) {
    assert!(
        started.elapsed() < timeout,
        "runtime events did not reach expected count within timeout"
    );
    std::thread::sleep(Duration::from_millis(25));
}
