use super::super::runtime_inbox::{
    emit_backpressure_runtime_event, enqueue_live_runtime_inbox_frame,
};
use super::*;

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
    if !state.peers_by_id.contains_key(&frame.sender_peer_id) {
        state
            .runtime_events
            .push_back(Libp2pRuntimeEvent::behavior_failure(
                Libp2pBehaviorFailureClass::UnknownSenderPeer,
                Some(frame.sender_peer_id.as_str()),
                Some(frame.topic.as_str()),
            )?);
        return Err(P2pTransportError::UnknownSenderPeer(frame.sender_peer_id));
    }
    if !state.peers_by_id.contains_key(&frame.recipient_peer_id) {
        state
            .runtime_events
            .push_back(Libp2pRuntimeEvent::behavior_failure(
                Libp2pBehaviorFailureClass::UnknownRecipientPeer,
                Some(frame.recipient_peer_id.as_str()),
                Some(frame.topic.as_str()),
            )?);
        return Err(P2pTransportError::UnknownRecipientPeer(
            frame.recipient_peer_id,
        ));
    }
    send_known_frame(&mut state, frame)
}

fn send_known_frame(
    state: &mut Libp2pLiveDataPlaneState,
    frame: PeerGossipFrame,
) -> Result<(), P2pTransportError> {
    let sender_peer_id = frame.sender_peer_id.clone();
    let recipient_peer_id = frame.recipient_peer_id.clone();
    let topic = frame.topic.clone();
    let payload = frame.payload.clone();
    if let Err(error) = enqueue_live_runtime_inbox_frame(
        state,
        recipient_peer_id.as_str(),
        PeerLifecycleState::Active,
        frame,
    ) {
        emit_backpressure_runtime_event(state, recipient_peer_id.as_str(), topic.as_str(), &error);
        return Err(error);
    }
    let published = Libp2pRuntimeEvent::gossip_published(
        sender_peer_id.as_str(),
        topic.as_str(),
        payload.as_str(),
    )?;
    let received = Libp2pRuntimeEvent::gossip_received(
        recipient_peer_id.as_str(),
        topic.as_str(),
        payload.as_str(),
    )?;
    state.runtime_events.push_back(published);
    state.runtime_events.push_back(received);
    Ok(())
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
