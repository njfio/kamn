use super::super::runtime_inbox::{
    emit_backpressure_runtime_event, enqueue_live_runtime_inbox_frame,
};
use super::*;
use crate::runtime::PeerLifecycleState;

pub(super) fn advertise(
    transport: &Libp2pLivePeerLifecycleTransport,
    record: PeerDiscoveryRecord,
) -> Result<(), P2pTransportError> {
    let mut state = transport.lock_live_data_plane_state()?;
    let event = Libp2pRuntimeEvent::peer_advertised(record.peer_id.as_str())?;
    state
        .inbox_by_peer
        .entry(record.peer_id.clone())
        .or_insert_with(VecDeque::new);
    state.peers_by_id.insert(record.peer_id.clone(), record);
    state.runtime_events.push_back(event);
    Ok(())
}

pub(super) fn discover(
    transport: &Libp2pLivePeerLifecycleTransport,
    requester_peer_id: &str,
    topic: &str,
) -> Result<Vec<PeerDiscoveryRecord>, P2pTransportError> {
    validate_peer_id(requester_peer_id)?;
    validate_topic(topic)?;
    let mut state = transport.lock_live_data_plane_state()?;
    let discovered = state
        .peers_by_id
        .values()
        .filter(|record| record.peer_id != requester_peer_id && record.supports_topic(topic))
        .cloned()
        .collect::<Vec<PeerDiscoveryRecord>>();
    for record in &discovered {
        state
            .runtime_events
            .push_back(Libp2pRuntimeEvent::peer_discovered(
                record.peer_id.as_str(),
                topic,
            )?);
    }
    Ok(discovered)
}

pub(super) fn send(
    transport: &Libp2pLivePeerLifecycleTransport,
    frame: PeerGossipFrame,
) -> Result<(), P2pTransportError> {
    let mut state = transport.lock_live_data_plane_state()?;
    ensure_known_sender(&mut state, &frame)?;
    ensure_known_recipient(&mut state, &frame)?;
    send_known_frame(&mut state, frame)
}

fn send_known_frame(
    state: &mut Libp2pLiveDataPlaneState,
    frame: PeerGossipFrame,
) -> Result<(), P2pTransportError> {
    let event_fields = clone_event_fields(&frame);
    enqueue_known_frame(state, &event_fields, frame)?;
    record_delivery_events(state, &event_fields)
}

pub(super) fn drain_inbox(
    transport: &Libp2pLivePeerLifecycleTransport,
    recipient_peer_id: &str,
) -> Result<Vec<PeerGossipFrame>, P2pTransportError> {
    validate_peer_id(recipient_peer_id)?;
    let mut state = transport.lock_live_data_plane_state()?;
    let queue = state
        .inbox_by_peer
        .entry(recipient_peer_id.to_owned())
        .or_insert_with(VecDeque::new);
    Ok(queue.drain(..).collect())
}

fn ensure_known_sender(
    state: &mut Libp2pLiveDataPlaneState,
    frame: &PeerGossipFrame,
) -> Result<(), P2pTransportError> {
    ensure_known_peer(
        state,
        state.peers_by_id.contains_key(&frame.sender_peer_id),
        Libp2pBehaviorFailureClass::UnknownSenderPeer,
        frame.sender_peer_id.as_str(),
        frame.topic.as_str(),
        P2pTransportError::UnknownSenderPeer(frame.sender_peer_id.clone()),
    )
}

fn ensure_known_recipient(
    state: &mut Libp2pLiveDataPlaneState,
    frame: &PeerGossipFrame,
) -> Result<(), P2pTransportError> {
    ensure_known_peer(
        state,
        state.peers_by_id.contains_key(&frame.recipient_peer_id),
        Libp2pBehaviorFailureClass::UnknownRecipientPeer,
        frame.recipient_peer_id.as_str(),
        frame.topic.as_str(),
        P2pTransportError::UnknownRecipientPeer(frame.recipient_peer_id.clone()),
    )
}

fn ensure_known_peer(
    state: &mut Libp2pLiveDataPlaneState,
    is_known: bool,
    class: Libp2pBehaviorFailureClass,
    peer_id: &str,
    topic: &str,
    error: P2pTransportError,
) -> Result<(), P2pTransportError> {
    if is_known {
        return Ok(());
    }
    state
        .runtime_events
        .push_back(Libp2pRuntimeEvent::behavior_failure(
            class,
            Some(peer_id),
            Some(topic),
        )?);
    Err(error)
}

fn clone_event_fields(frame: &PeerGossipFrame) -> (String, String, String, String) {
    (
        frame.sender_peer_id.clone(),
        frame.recipient_peer_id.clone(),
        frame.topic.clone(),
        frame.payload.clone(),
    )
}

fn enqueue_known_frame(
    state: &mut Libp2pLiveDataPlaneState,
    event_fields: &(String, String, String, String),
    frame: PeerGossipFrame,
) -> Result<(), P2pTransportError> {
    if let Err(error) = enqueue_live_runtime_inbox_frame(
        state,
        event_fields.1.as_str(),
        PeerLifecycleState::Active,
        frame,
    ) {
        emit_backpressure_runtime_event(
            state,
            event_fields.1.as_str(),
            event_fields.2.as_str(),
            &error,
        );
        return Err(error);
    }
    Ok(())
}

fn record_delivery_events(
    state: &mut Libp2pLiveDataPlaneState,
    event_fields: &(String, String, String, String),
) -> Result<(), P2pTransportError> {
    let published = Libp2pRuntimeEvent::gossip_published(
        event_fields.0.as_str(),
        event_fields.2.as_str(),
        event_fields.3.as_str(),
    )?;
    let received = Libp2pRuntimeEvent::gossip_received(
        event_fields.1.as_str(),
        event_fields.2.as_str(),
        event_fields.3.as_str(),
    )?;
    state.runtime_events.push_back(published);
    state.runtime_events.push_back(received);
    Ok(())
}
