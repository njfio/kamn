#[cfg(feature = "libp2p-live-transport")]
use super::p2p_transport_live::{
    apply_libp2p_runtime_network_config, build_libp2p_runtime_swarm,
    runtime_channel_closed_behavior_failure_class, validate_libp2p_runtime_stack_composition,
    Libp2pLiveDataPlaneState,
};
#[cfg(feature = "libp2p-live-transport")]
use libp2p::{futures::StreamExt, gossipsub, swarm::Swarm};
#[cfg(feature = "libp2p-live-transport")]
use std::collections::VecDeque;
#[cfg(feature = "libp2p-live-transport")]
use std::sync::{Arc, Mutex};

#[cfg(feature = "libp2p-live-transport")]
const LIBP2P_RUNTIME_ADAPTER_LOOP_MARKER: &str = "libp2p-runtime-adapter-loop";

#[cfg(feature = "libp2p-live-transport")]
#[derive(Debug)]
enum Libp2pNativeRuntimeAdapterLoopCommand {
    Advertise {
        record: super::PeerDiscoveryRecord,
        response: std::sync::mpsc::Sender<Result<(), super::P2pTransportError>>,
    },
    Discover {
        requester_peer_id: String,
        topic: String,
        response: std::sync::mpsc::Sender<
            Result<Vec<super::PeerDiscoveryRecord>, super::P2pTransportError>,
        >,
    },
    Send {
        frame: super::PeerGossipFrame,
        response: std::sync::mpsc::Sender<Result<(), super::P2pTransportError>>,
    },
    DrainInbox {
        recipient_peer_id: String,
        response:
            std::sync::mpsc::Sender<Result<Vec<super::PeerGossipFrame>, super::P2pTransportError>>,
    },
    DrainRuntimeEvents {
        response: std::sync::mpsc::Sender<
            Result<Vec<super::Libp2pRuntimeEvent>, super::P2pTransportError>,
        >,
    },
}

#[cfg(feature = "libp2p-live-transport")]
#[derive(Debug)]
enum Libp2pNativeSwarmCommand {
    Publish {
        frame: super::PeerGossipFrame,
        response: std::sync::mpsc::Sender<Result<(), super::P2pTransportError>>,
    },
}

#[cfg(feature = "libp2p-live-transport")]
#[derive(Debug, Clone)]
pub(super) struct Libp2pNativeRuntimeAdapterLoop {
    command_tx: std::sync::mpsc::Sender<Libp2pNativeRuntimeAdapterLoopCommand>,
    state: Arc<Mutex<Libp2pLiveDataPlaneState>>,
}

#[cfg(feature = "libp2p-live-transport")]
impl Libp2pNativeRuntimeAdapterLoop {
    pub(super) fn start(
        config: super::P2pSwarmDeterministicConfig,
        state: Arc<Mutex<Libp2pLiveDataPlaneState>>,
    ) -> Result<Self, super::P2pTransportError> {
        validate_libp2p_runtime_stack_composition(&config)?;
        let local_peer_id = config.local_peer_id().to_owned();
        let (swarm_command_tx, swarm_command_rx) = std::sync::mpsc::channel();
        let swarm_config = config.clone();
        let swarm_state = state.clone();
        std::thread::Builder::new()
            .name(format!("kamn-libp2p-swarm-{local_peer_id}"))
            .spawn(move || {
                run_libp2p_native_swarm_loop(swarm_config, swarm_command_rx, swarm_state);
            })
            .map_err(|_| super::P2pTransportError::StateUnavailable)?;
        let (command_tx, command_rx) = std::sync::mpsc::channel();
        let runtime_state = state.clone();
        std::thread::Builder::new()
            .name(format!("kamn-libp2p-adapter-{local_peer_id}"))
            .spawn(move || {
                run_libp2p_native_runtime_adapter_loop(command_rx, swarm_command_tx, state);
            })
            .map_err(|_| super::P2pTransportError::StateUnavailable)?;
        Ok(Self {
            command_tx,
            state: runtime_state,
        })
    }

    pub(super) fn marker(&self) -> &'static str {
        LIBP2P_RUNTIME_ADAPTER_LOOP_MARKER
    }

    fn emit_channel_closed_runtime_event(&self, operation: super::Libp2pRuntimeAdapterOperation) {
        let class = runtime_channel_closed_behavior_failure_class(operation);
        let event = match super::Libp2pRuntimeEvent::behavior_failure(class, None, None) {
            Ok(event) => event,
            Err(_) => return,
        };
        if let Ok(mut state) = self.state.lock() {
            state.runtime_events.push_back(event);
        }
    }

    fn channel_closed_error(
        &self,
        operation: super::Libp2pRuntimeAdapterOperation,
    ) -> super::P2pTransportError {
        self.emit_channel_closed_runtime_event(operation);
        super::P2pTransportError::Libp2pRuntimeAdapterChannelClosed(operation)
    }

    pub(super) fn advertise(
        &self,
        record: super::PeerDiscoveryRecord,
    ) -> Result<(), super::P2pTransportError> {
        let (response_tx, response_rx) = std::sync::mpsc::channel();
        if self
            .command_tx
            .send(Libp2pNativeRuntimeAdapterLoopCommand::Advertise {
                record,
                response: response_tx,
            })
            .is_err()
        {
            return Err(self.channel_closed_error(super::Libp2pRuntimeAdapterOperation::Connect));
        }
        response_rx
            .recv()
            .map_err(|_| self.channel_closed_error(super::Libp2pRuntimeAdapterOperation::Connect))?
    }

    pub(super) fn discover(
        &self,
        requester_peer_id: &str,
        topic: &str,
    ) -> Result<Vec<super::PeerDiscoveryRecord>, super::P2pTransportError> {
        let (response_tx, response_rx) = std::sync::mpsc::channel();
        if self
            .command_tx
            .send(Libp2pNativeRuntimeAdapterLoopCommand::Discover {
                requester_peer_id: requester_peer_id.to_owned(),
                topic: topic.to_owned(),
                response: response_tx,
            })
            .is_err()
        {
            return Err(self.channel_closed_error(super::Libp2pRuntimeAdapterOperation::Discover));
        }
        response_rx.recv().map_err(|_| {
            self.channel_closed_error(super::Libp2pRuntimeAdapterOperation::Discover)
        })?
    }

    pub(super) fn send(
        &self,
        frame: super::PeerGossipFrame,
    ) -> Result<(), super::P2pTransportError> {
        let (response_tx, response_rx) = std::sync::mpsc::channel();
        if self
            .command_tx
            .send(Libp2pNativeRuntimeAdapterLoopCommand::Send {
                frame,
                response: response_tx,
            })
            .is_err()
        {
            return Err(self.channel_closed_error(super::Libp2pRuntimeAdapterOperation::Publish));
        }
        response_rx
            .recv()
            .map_err(|_| self.channel_closed_error(super::Libp2pRuntimeAdapterOperation::Publish))?
    }

    pub(super) fn drain_inbox(
        &self,
        recipient_peer_id: &str,
    ) -> Result<Vec<super::PeerGossipFrame>, super::P2pTransportError> {
        let (response_tx, response_rx) = std::sync::mpsc::channel();
        if self
            .command_tx
            .send(Libp2pNativeRuntimeAdapterLoopCommand::DrainInbox {
                recipient_peer_id: recipient_peer_id.to_owned(),
                response: response_tx,
            })
            .is_err()
        {
            return Err(self.channel_closed_error(super::Libp2pRuntimeAdapterOperation::Receive));
        }
        response_rx
            .recv()
            .map_err(|_| self.channel_closed_error(super::Libp2pRuntimeAdapterOperation::Receive))?
    }

    pub(super) fn drain_runtime_events(
        &self,
    ) -> Result<Vec<super::Libp2pRuntimeEvent>, super::P2pTransportError> {
        let (response_tx, response_rx) = std::sync::mpsc::channel();
        if self
            .command_tx
            .send(Libp2pNativeRuntimeAdapterLoopCommand::DrainRuntimeEvents {
                response: response_tx,
            })
            .is_err()
        {
            return Err(self.channel_closed_error(super::Libp2pRuntimeAdapterOperation::EventDrain));
        }
        response_rx.recv().map_err(|_| {
            self.channel_closed_error(super::Libp2pRuntimeAdapterOperation::EventDrain)
        })?
    }

    #[cfg(test)]
    pub(super) fn build_closed_for_test(state: Arc<Mutex<Libp2pLiveDataPlaneState>>) -> Self {
        let (command_tx, command_rx) = std::sync::mpsc::channel();
        drop(command_rx);
        Self { command_tx, state }
    }
}

#[cfg(feature = "libp2p-live-transport")]
fn run_libp2p_native_runtime_adapter_loop(
    command_rx: std::sync::mpsc::Receiver<Libp2pNativeRuntimeAdapterLoopCommand>,
    swarm_command_tx: std::sync::mpsc::Sender<Libp2pNativeSwarmCommand>,
    state: Arc<Mutex<Libp2pLiveDataPlaneState>>,
) {
    while let Ok(command) = command_rx.recv() {
        match command {
            Libp2pNativeRuntimeAdapterLoopCommand::Advertise { record, response } => {
                let result = state
                    .lock()
                    .map_err(|_| super::P2pTransportError::StateUnavailable)
                    .and_then(|mut locked_state| {
                        let event =
                            super::Libp2pRuntimeEvent::peer_advertised(record.peer_id.as_str())?;
                        locked_state
                            .inbox_by_peer
                            .entry(record.peer_id.clone())
                            .or_insert_with(VecDeque::new);
                        locked_state
                            .peers_by_id
                            .insert(record.peer_id.clone(), record);
                        locked_state.runtime_events.push_back(event);
                        Ok(())
                    });
                let _ = response.send(result);
            }
            Libp2pNativeRuntimeAdapterLoopCommand::Discover {
                requester_peer_id,
                topic,
                response,
            } => {
                let result = super::validate_peer_id(requester_peer_id.as_str())
                    .and_then(|_| super::validate_topic(topic.as_str()))
                    .and_then(|_| {
                        state
                            .lock()
                            .map_err(|_| super::P2pTransportError::StateUnavailable)
                            .and_then(|mut locked_state| {
                                let discovered = locked_state
                                    .peers_by_id
                                    .values()
                                    .filter(|record| {
                                        record.peer_id != requester_peer_id
                                            && record.supports_topic(topic.as_str())
                                    })
                                    .cloned()
                                    .collect::<Vec<super::PeerDiscoveryRecord>>();
                                for record in &discovered {
                                    locked_state.runtime_events.push_back(
                                        super::Libp2pRuntimeEvent::peer_discovered(
                                            record.peer_id.as_str(),
                                            topic.as_str(),
                                        )?,
                                    );
                                }
                                Ok(discovered)
                            })
                    });
                let _ = response.send(result);
            }
            Libp2pNativeRuntimeAdapterLoopCommand::Send { frame, response } => {
                let sender_peer_id = frame.sender_peer_id.clone();
                let recipient_peer_id = frame.recipient_peer_id.clone();
                let topic = frame.topic.clone();
                let payload = frame.payload.clone();
                let result = state
                    .lock()
                    .map_err(|_| super::P2pTransportError::StateUnavailable)
                    .and_then(|mut locked_state| {
                        if !locked_state
                            .peers_by_id
                            .contains_key(sender_peer_id.as_str())
                        {
                            locked_state.runtime_events.push_back(
                                super::Libp2pRuntimeEvent::behavior_failure(
                                    super::Libp2pBehaviorFailureClass::UnknownSenderPeer,
                                    Some(sender_peer_id.as_str()),
                                    Some(topic.as_str()),
                                )?,
                            );
                            return Err(super::P2pTransportError::UnknownSenderPeer(
                                sender_peer_id.clone(),
                            ));
                        }
                        if !locked_state
                            .peers_by_id
                            .contains_key(recipient_peer_id.as_str())
                        {
                            locked_state.runtime_events.push_back(
                                super::Libp2pRuntimeEvent::behavior_failure(
                                    super::Libp2pBehaviorFailureClass::UnknownRecipientPeer,
                                    Some(recipient_peer_id.as_str()),
                                    Some(topic.as_str()),
                                )?,
                            );
                            return Err(super::P2pTransportError::UnknownRecipientPeer(
                                recipient_peer_id.clone(),
                            ));
                        }
                        let (publish_response_tx, publish_response_rx) = std::sync::mpsc::channel();
                        swarm_command_tx
                            .send(Libp2pNativeSwarmCommand::Publish {
                                frame,
                                response: publish_response_tx,
                            })
                            .map_err(|_| super::P2pTransportError::LiveSocketSendFailed)?;
                        let publish_result = publish_response_rx
                            .recv()
                            .map_err(|_| super::P2pTransportError::LiveSocketSendFailed)?;
                        if publish_result.is_ok() {
                            let published = super::Libp2pRuntimeEvent::gossip_published(
                                sender_peer_id.as_str(),
                                topic.as_str(),
                                payload.as_str(),
                            )?;
                            locked_state.runtime_events.push_back(published);
                        }
                        publish_result
                    });
                let _ = response.send(result);
            }
            Libp2pNativeRuntimeAdapterLoopCommand::DrainInbox {
                recipient_peer_id,
                response,
            } => {
                let result = super::validate_peer_id(recipient_peer_id.as_str()).and_then(|_| {
                    state
                        .lock()
                        .map_err(|_| super::P2pTransportError::StateUnavailable)
                        .map(|mut locked_state| {
                            let queue = locked_state
                                .inbox_by_peer
                                .entry(recipient_peer_id)
                                .or_insert_with(VecDeque::new);
                            queue.drain(..).collect::<Vec<super::PeerGossipFrame>>()
                        })
                });
                let _ = response.send(result);
            }
            Libp2pNativeRuntimeAdapterLoopCommand::DrainRuntimeEvents { response } => {
                let result = state
                    .lock()
                    .map_err(|_| super::P2pTransportError::StateUnavailable)
                    .map(|mut locked_state| locked_state.runtime_events.drain(..).collect());
                let _ = response.send(result);
            }
        }
    }
}

#[cfg(feature = "libp2p-live-transport")]
fn run_libp2p_native_swarm_loop(
    config: super::P2pSwarmDeterministicConfig,
    swarm_command_rx: std::sync::mpsc::Receiver<Libp2pNativeSwarmCommand>,
    state: Arc<Mutex<Libp2pLiveDataPlaneState>>,
) {
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
    {
        Ok(runtime) => runtime,
        Err(_) => return,
    };
    runtime.block_on(async move {
        let mut swarm = match build_libp2p_runtime_swarm(&config) {
            Ok(swarm) => swarm,
            Err(_) => return,
        };
        if apply_libp2p_runtime_network_config(&mut swarm, &config).is_err() {
            return;
        }
        let local_peer_id = config.local_peer_id().to_owned();

        loop {
            loop {
                match swarm_command_rx.try_recv() {
                    Ok(Libp2pNativeSwarmCommand::Publish { frame, response }) => {
                        let publish_result = publish_libp2p_gossip_frame(&mut swarm, &frame);
                        let _ = response.send(publish_result);
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => break,
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => return,
                }
            }

            if let Ok(event) = tokio::time::timeout(
                std::time::Duration::from_millis(10),
                swarm.select_next_some(),
            )
            .await
            {
                apply_libp2p_swarm_event_to_live_state(
                    event,
                    state.clone(),
                    local_peer_id.as_str(),
                );
            }
        }
    });
}

#[cfg(feature = "libp2p-live-transport")]
fn publish_libp2p_gossip_frame(
    swarm: &mut Swarm<super::Libp2pDeterministicRuntimeBehaviour>,
    frame: &super::PeerGossipFrame,
) -> Result<(), super::P2pTransportError> {
    let topic_id = super::canonical_libp2p_topic_id(frame.topic.as_str())?;
    let publish_topic = gossipsub::IdentTopic::new(topic_id);
    swarm
        .behaviour_mut()
        .gossipsub
        .publish(
            publish_topic,
            super::UdpPeerLifecycleTransport::encode_frame(frame),
        )
        .map_err(|_| super::P2pTransportError::LiveSocketSendFailed)?;
    Ok(())
}

#[cfg(feature = "libp2p-live-transport")]
fn apply_libp2p_swarm_event_to_live_state(
    event: libp2p::swarm::SwarmEvent<super::Libp2pDeterministicRuntimeBehaviourEvent>,
    state: Arc<Mutex<Libp2pLiveDataPlaneState>>,
    local_peer_id: &str,
) {
    let libp2p::swarm::SwarmEvent::Behaviour(
        super::Libp2pDeterministicRuntimeBehaviourEvent::Gossipsub(gossipsub::Event::Message {
            message,
            ..
        }),
    ) = event
    else {
        return;
    };

    let frame = match super::UdpPeerLifecycleTransport::decode_frame(message.data.as_slice()) {
        Ok(frame) => frame,
        Err(_) => return,
    };
    if frame.recipient_peer_id != local_peer_id {
        return;
    }

    let topic = frame.topic.clone();
    let payload = frame.payload.clone();
    let recipient_peer_id = frame.recipient_peer_id.clone();
    if let Ok(mut locked_state) = state.lock() {
        locked_state
            .inbox_by_peer
            .entry(recipient_peer_id.clone())
            .or_insert_with(VecDeque::new)
            .push_back(frame);
        if let Ok(event) = super::Libp2pRuntimeEvent::gossip_received(
            recipient_peer_id.as_str(),
            topic.as_str(),
            payload.as_str(),
        ) {
            locked_state.runtime_events.push_back(event);
        }
    }
}
