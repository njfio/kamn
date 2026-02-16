use super::*;

#[cfg(feature = "libp2p-live-transport")]
use libp2p::{
    futures::StreamExt, gossipsub, identify, noise, swarm::Swarm, tcp, yamux, Multiaddr,
    SwarmBuilder,
};

/// Runtime backend mode selected for live libp2p transport operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Libp2pLiveRuntimeBackend {
    /// Deterministic in-process data-plane fallback path.
    ContractDataPlane,
    /// Native socket-backed path used by feature-enabled runtime builds.
    NativeSocket,
}

impl Libp2pLiveRuntimeBackend {
    /// Returns deterministic backend marker for policy and docs contracts.
    pub fn marker(self) -> &'static str {
        match self {
            Self::ContractDataPlane => "contract-data-plane",
            Self::NativeSocket => "native-libp2p-swarm",
        }
    }
}

/// Resolves live libp2p runtime backend mode from compile-time feature gates.
pub fn resolve_libp2p_live_runtime_backend() -> Libp2pLiveRuntimeBackend {
    #[cfg(feature = "libp2p-live-transport")]
    {
        Libp2pLiveRuntimeBackend::NativeSocket
    }
    #[cfg(not(feature = "libp2p-live-transport"))]
    {
        Libp2pLiveRuntimeBackend::ContractDataPlane
    }
}

/// Live libp2p transport adapter contract backed by deterministic swarm startup.
#[derive(Debug, Clone)]
pub struct Libp2pLivePeerLifecycleTransport {
    swarm_config: P2pSwarmDeterministicConfig,
    harness_report: P2pSwarmHarnessReport,
    live_network_id: String,
    #[cfg(feature = "libp2p-live-transport")]
    native_runtime_loop: Libp2pNativeRuntimeAdapterLoop,
    #[cfg(not(feature = "libp2p-live-transport"))]
    live_data_plane: Libp2pLiveDataPlane,
}

impl Libp2pLivePeerLifecycleTransport {
    /// Builds a live transport adapter and starts deterministic harness startup.
    pub fn new(
        config: P2pSwarmDeterministicConfig,
        harness_mode: P2pSwarmHarnessMode,
    ) -> Result<Self, P2pTransportError> {
        let task = P2pSwarmHarnessTask::new(config.clone());
        let harness_report = task.start(harness_mode)?;
        let network_id = build_live_data_plane_network_id(&config);
        let state = resolve_live_data_plane_state(network_id.as_str())?;
        #[cfg(feature = "libp2p-live-transport")]
        validate_libp2p_native_runtime_config(&config)?;
        #[cfg(feature = "libp2p-live-transport")]
        let native_runtime_loop =
            Libp2pNativeRuntimeAdapterLoop::start(config.clone(), state.clone())?;
        Ok(Self {
            swarm_config: config,
            harness_report,
            live_network_id: network_id.clone(),
            #[cfg(feature = "libp2p-live-transport")]
            native_runtime_loop,
            #[cfg(not(feature = "libp2p-live-transport"))]
            live_data_plane: Libp2pLiveDataPlane { state },
        })
    }

    /// Returns runtime transport profile marker for this adapter.
    pub fn transport_profile(&self) -> RuntimeTransportProfile {
        RuntimeTransportProfile::Libp2pLive
    }

    /// Returns deterministic harness startup report for this live adapter.
    pub fn harness_report(&self) -> &P2pSwarmHarnessReport {
        &self.harness_report
    }

    /// Returns compile-mode runtime backend selected for this adapter.
    pub fn runtime_backend(&self) -> Libp2pLiveRuntimeBackend {
        resolve_libp2p_live_runtime_backend()
    }

    /// Returns configured listen address for this live adapter.
    pub fn listen_address(&self) -> &str {
        self.swarm_config.listen_address()
    }

    /// Returns deterministic live data-plane network identifier.
    pub fn live_data_plane_network_id(&self) -> &str {
        self.live_network_id.as_str()
    }

    /// Drains normalized runtime events emitted by this transport adapter.
    pub fn drain_runtime_events(&self) -> Result<Vec<Libp2pRuntimeEvent>, P2pTransportError> {
        #[cfg(feature = "libp2p-live-transport")]
        {
            self.native_runtime_loop.drain_runtime_events()
        }
        #[cfg(not(feature = "libp2p-live-transport"))]
        {
            let mut state = self.lock_live_data_plane_state()?;
            Ok(state.runtime_events.drain(..).collect())
        }
    }

    #[cfg(feature = "libp2p-live-transport")]
    /// Returns deterministic native runtime loop marker for feature-enabled adapter wiring.
    pub fn native_runtime_loop_marker(&self) -> &'static str {
        self.native_runtime_loop.marker()
    }

    #[cfg(not(feature = "libp2p-live-transport"))]
    fn lock_live_data_plane_state(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, Libp2pLiveDataPlaneState>, P2pTransportError> {
        self.live_data_plane
            .state
            .lock()
            .map_err(|_| P2pTransportError::StateUnavailable)
    }
}

impl PeerLifecycleTransport for Libp2pLivePeerLifecycleTransport {
    fn advertise(&self, record: PeerDiscoveryRecord) -> Result<(), P2pTransportError> {
        #[cfg(feature = "libp2p-live-transport")]
        {
            self.native_runtime_loop.advertise(record)
        }
        #[cfg(not(feature = "libp2p-live-transport"))]
        let mut state = self.lock_live_data_plane_state()?;
        #[cfg(not(feature = "libp2p-live-transport"))]
        {
            let event = Libp2pRuntimeEvent::peer_advertised(record.peer_id.as_str())?;
            state
                .inbox_by_peer
                .entry(record.peer_id.clone())
                .or_insert_with(VecDeque::new);
            state.peers_by_id.insert(record.peer_id.clone(), record);
            state.runtime_events.push_back(event);
            Ok(())
        }
    }

    fn discover(
        &self,
        requester_peer_id: &str,
        topic: &str,
    ) -> Result<Vec<PeerDiscoveryRecord>, P2pTransportError> {
        #[cfg(feature = "libp2p-live-transport")]
        {
            self.native_runtime_loop.discover(requester_peer_id, topic)
        }
        #[cfg(not(feature = "libp2p-live-transport"))]
        validate_peer_id(requester_peer_id)?;
        #[cfg(not(feature = "libp2p-live-transport"))]
        validate_topic(topic)?;
        #[cfg(not(feature = "libp2p-live-transport"))]
        let mut state = self.lock_live_data_plane_state()?;
        #[cfg(not(feature = "libp2p-live-transport"))]
        {
            let discovered = state
                .peers_by_id
                .values()
                .filter(|record| {
                    record.peer_id != requester_peer_id && record.supports_topic(topic)
                })
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
    }

    fn send(&self, frame: PeerGossipFrame) -> Result<(), P2pTransportError> {
        #[cfg(feature = "libp2p-live-transport")]
        {
            self.native_runtime_loop.send(frame)
        }
        #[cfg(not(feature = "libp2p-live-transport"))]
        let mut state = self.lock_live_data_plane_state()?;
        #[cfg(not(feature = "libp2p-live-transport"))]
        if !state.peers_by_id.contains_key(&frame.sender_peer_id) {
            state
                .runtime_events
                .push_back(Libp2pRuntimeEvent::behavior_failure(
                    Libp2pBehaviorFailureClass::UnknownSenderPeer,
                    Some(frame.sender_peer_id.as_str()),
                    Some(frame.topic.as_str()),
                )?);
            return Err(P2pTransportError::UnknownSenderPeer(
                frame.sender_peer_id.clone(),
            ));
        }
        #[cfg(not(feature = "libp2p-live-transport"))]
        if !state.peers_by_id.contains_key(&frame.recipient_peer_id) {
            state
                .runtime_events
                .push_back(Libp2pRuntimeEvent::behavior_failure(
                    Libp2pBehaviorFailureClass::UnknownRecipientPeer,
                    Some(frame.recipient_peer_id.as_str()),
                    Some(frame.topic.as_str()),
                )?);
            return Err(P2pTransportError::UnknownRecipientPeer(
                frame.recipient_peer_id.clone(),
            ));
        }
        #[cfg(not(feature = "libp2p-live-transport"))]
        {
            let published = Libp2pRuntimeEvent::gossip_published(
                frame.sender_peer_id.as_str(),
                frame.topic.as_str(),
                frame.payload.as_str(),
            )?;
            let received = Libp2pRuntimeEvent::gossip_received(
                frame.recipient_peer_id.as_str(),
                frame.topic.as_str(),
                frame.payload.as_str(),
            )?;
            state
                .inbox_by_peer
                .entry(frame.recipient_peer_id.clone())
                .or_insert_with(VecDeque::new)
                .push_back(frame);
            state.runtime_events.push_back(published);
            state.runtime_events.push_back(received);
            Ok(())
        }
    }

    fn drain_inbox(
        &self,
        recipient_peer_id: &str,
    ) -> Result<Vec<PeerGossipFrame>, P2pTransportError> {
        #[cfg(feature = "libp2p-live-transport")]
        {
            self.native_runtime_loop.drain_inbox(recipient_peer_id)
        }
        #[cfg(not(feature = "libp2p-live-transport"))]
        validate_peer_id(recipient_peer_id)?;
        #[cfg(not(feature = "libp2p-live-transport"))]
        let mut state = self.lock_live_data_plane_state()?;
        #[cfg(not(feature = "libp2p-live-transport"))]
        {
            let queue = state
                .inbox_by_peer
                .entry(recipient_peer_id.to_owned())
                .or_insert_with(VecDeque::new);
            Ok(queue.drain(..).collect())
        }
    }
}

#[derive(Debug, Default)]
pub(super) struct Libp2pLiveDataPlaneState {
    peers_by_id: BTreeMap<String, PeerDiscoveryRecord>,
    inbox_by_peer: BTreeMap<String, VecDeque<PeerGossipFrame>>,
    runtime_events: VecDeque<Libp2pRuntimeEvent>,
}

#[cfg(not(feature = "libp2p-live-transport"))]
#[derive(Debug, Clone)]
struct Libp2pLiveDataPlane {
    state: Arc<Mutex<Libp2pLiveDataPlaneState>>,
}

fn libp2p_live_data_plane_registry(
) -> &'static Mutex<BTreeMap<String, Arc<Mutex<Libp2pLiveDataPlaneState>>>> {
    static REGISTRY: OnceLock<Mutex<BTreeMap<String, Arc<Mutex<Libp2pLiveDataPlaneState>>>>> =
        OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(BTreeMap::new()))
}

fn resolve_live_data_plane_state(
    network_id: &str,
) -> Result<Arc<Mutex<Libp2pLiveDataPlaneState>>, P2pTransportError> {
    let mut registry = libp2p_live_data_plane_registry()
        .lock()
        .map_err(|_| P2pTransportError::StateUnavailable)?;
    Ok(registry
        .entry(network_id.to_owned())
        .or_insert_with(|| Arc::new(Mutex::new(Libp2pLiveDataPlaneState::default())))
        .clone())
}

#[cfg(feature = "libp2p-live-transport")]
const LIBP2P_RUNTIME_ADAPTER_LOOP_MARKER: &str = "libp2p-runtime-adapter-loop";

#[cfg(feature = "libp2p-live-transport")]
#[derive(Debug)]
enum Libp2pNativeRuntimeAdapterLoopCommand {
    Advertise {
        record: PeerDiscoveryRecord,
        response: std::sync::mpsc::Sender<Result<(), P2pTransportError>>,
    },
    Discover {
        requester_peer_id: String,
        topic: String,
        response: std::sync::mpsc::Sender<Result<Vec<PeerDiscoveryRecord>, P2pTransportError>>,
    },
    Send {
        frame: PeerGossipFrame,
        response: std::sync::mpsc::Sender<Result<(), P2pTransportError>>,
    },
    DrainInbox {
        recipient_peer_id: String,
        response: std::sync::mpsc::Sender<Result<Vec<PeerGossipFrame>, P2pTransportError>>,
    },
    DrainRuntimeEvents {
        response: std::sync::mpsc::Sender<Result<Vec<Libp2pRuntimeEvent>, P2pTransportError>>,
    },
}

#[cfg(feature = "libp2p-live-transport")]
#[derive(Debug)]
enum Libp2pNativeSwarmCommand {
    Publish {
        frame: PeerGossipFrame,
        response: std::sync::mpsc::Sender<Result<(), P2pTransportError>>,
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
    fn start(
        config: P2pSwarmDeterministicConfig,
        state: Arc<Mutex<Libp2pLiveDataPlaneState>>,
    ) -> Result<Self, P2pTransportError> {
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
            .map_err(|_| P2pTransportError::StateUnavailable)?;
        let (command_tx, command_rx) = std::sync::mpsc::channel();
        let runtime_state = state.clone();
        std::thread::Builder::new()
            .name(format!("kamn-libp2p-adapter-{local_peer_id}"))
            .spawn(move || {
                run_libp2p_native_runtime_adapter_loop(command_rx, swarm_command_tx, state);
            })
            .map_err(|_| P2pTransportError::StateUnavailable)?;
        Ok(Self {
            command_tx,
            state: runtime_state,
        })
    }

    fn marker(&self) -> &'static str {
        LIBP2P_RUNTIME_ADAPTER_LOOP_MARKER
    }

    fn emit_channel_closed_runtime_event(&self, operation: Libp2pRuntimeAdapterOperation) {
        let class = runtime_channel_closed_behavior_failure_class(operation);
        let event = match Libp2pRuntimeEvent::behavior_failure(class, None, None) {
            Ok(event) => event,
            Err(_) => return,
        };
        if let Ok(mut state) = self.state.lock() {
            state.runtime_events.push_back(event);
        }
    }

    fn channel_closed_error(&self, operation: Libp2pRuntimeAdapterOperation) -> P2pTransportError {
        self.emit_channel_closed_runtime_event(operation);
        P2pTransportError::Libp2pRuntimeAdapterChannelClosed(operation)
    }

    pub(super) fn advertise(&self, record: PeerDiscoveryRecord) -> Result<(), P2pTransportError> {
        let (response_tx, response_rx) = std::sync::mpsc::channel();
        if self
            .command_tx
            .send(Libp2pNativeRuntimeAdapterLoopCommand::Advertise {
                record,
                response: response_tx,
            })
            .is_err()
        {
            return Err(self.channel_closed_error(Libp2pRuntimeAdapterOperation::Connect));
        }
        response_rx
            .recv()
            .map_err(|_| self.channel_closed_error(Libp2pRuntimeAdapterOperation::Connect))?
    }

    pub(super) fn discover(
        &self,
        requester_peer_id: &str,
        topic: &str,
    ) -> Result<Vec<PeerDiscoveryRecord>, P2pTransportError> {
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
            return Err(self.channel_closed_error(Libp2pRuntimeAdapterOperation::Discover));
        }
        response_rx
            .recv()
            .map_err(|_| self.channel_closed_error(Libp2pRuntimeAdapterOperation::Discover))?
    }

    pub(super) fn send(&self, frame: PeerGossipFrame) -> Result<(), P2pTransportError> {
        let (response_tx, response_rx) = std::sync::mpsc::channel();
        if self
            .command_tx
            .send(Libp2pNativeRuntimeAdapterLoopCommand::Send {
                frame,
                response: response_tx,
            })
            .is_err()
        {
            return Err(self.channel_closed_error(Libp2pRuntimeAdapterOperation::Publish));
        }
        response_rx
            .recv()
            .map_err(|_| self.channel_closed_error(Libp2pRuntimeAdapterOperation::Publish))?
    }

    pub(super) fn drain_inbox(
        &self,
        recipient_peer_id: &str,
    ) -> Result<Vec<PeerGossipFrame>, P2pTransportError> {
        let (response_tx, response_rx) = std::sync::mpsc::channel();
        if self
            .command_tx
            .send(Libp2pNativeRuntimeAdapterLoopCommand::DrainInbox {
                recipient_peer_id: recipient_peer_id.to_owned(),
                response: response_tx,
            })
            .is_err()
        {
            return Err(self.channel_closed_error(Libp2pRuntimeAdapterOperation::Receive));
        }
        response_rx
            .recv()
            .map_err(|_| self.channel_closed_error(Libp2pRuntimeAdapterOperation::Receive))?
    }

    pub(super) fn drain_runtime_events(
        &self,
    ) -> Result<Vec<Libp2pRuntimeEvent>, P2pTransportError> {
        let (response_tx, response_rx) = std::sync::mpsc::channel();
        if self
            .command_tx
            .send(Libp2pNativeRuntimeAdapterLoopCommand::DrainRuntimeEvents {
                response: response_tx,
            })
            .is_err()
        {
            return Err(self.channel_closed_error(Libp2pRuntimeAdapterOperation::EventDrain));
        }
        response_rx
            .recv()
            .map_err(|_| self.channel_closed_error(Libp2pRuntimeAdapterOperation::EventDrain))?
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
                    .map_err(|_| P2pTransportError::StateUnavailable)
                    .and_then(|mut locked_state| {
                        let event = Libp2pRuntimeEvent::peer_advertised(record.peer_id.as_str())?;
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
                let result = validate_peer_id(requester_peer_id.as_str())
                    .and_then(|_| validate_topic(topic.as_str()))
                    .and_then(|_| {
                        state
                            .lock()
                            .map_err(|_| P2pTransportError::StateUnavailable)
                            .and_then(|mut locked_state| {
                                let discovered = locked_state
                                    .peers_by_id
                                    .values()
                                    .filter(|record| {
                                        record.peer_id != requester_peer_id
                                            && record.supports_topic(topic.as_str())
                                    })
                                    .cloned()
                                    .collect::<Vec<PeerDiscoveryRecord>>();
                                for record in &discovered {
                                    locked_state.runtime_events.push_back(
                                        Libp2pRuntimeEvent::peer_discovered(
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
                    .map_err(|_| P2pTransportError::StateUnavailable)
                    .and_then(|mut locked_state| {
                        if !locked_state
                            .peers_by_id
                            .contains_key(sender_peer_id.as_str())
                        {
                            locked_state.runtime_events.push_back(
                                Libp2pRuntimeEvent::behavior_failure(
                                    Libp2pBehaviorFailureClass::UnknownSenderPeer,
                                    Some(sender_peer_id.as_str()),
                                    Some(topic.as_str()),
                                )?,
                            );
                            return Err(P2pTransportError::UnknownSenderPeer(
                                sender_peer_id.clone(),
                            ));
                        }
                        if !locked_state
                            .peers_by_id
                            .contains_key(recipient_peer_id.as_str())
                        {
                            locked_state.runtime_events.push_back(
                                Libp2pRuntimeEvent::behavior_failure(
                                    Libp2pBehaviorFailureClass::UnknownRecipientPeer,
                                    Some(recipient_peer_id.as_str()),
                                    Some(topic.as_str()),
                                )?,
                            );
                            return Err(P2pTransportError::UnknownRecipientPeer(
                                recipient_peer_id.clone(),
                            ));
                        }
                        let (publish_response_tx, publish_response_rx) = std::sync::mpsc::channel();
                        swarm_command_tx
                            .send(Libp2pNativeSwarmCommand::Publish {
                                frame,
                                response: publish_response_tx,
                            })
                            .map_err(|_| P2pTransportError::LiveSocketSendFailed)?;
                        let publish_result = publish_response_rx
                            .recv()
                            .map_err(|_| P2pTransportError::LiveSocketSendFailed)?;
                        if publish_result.is_ok() {
                            let published = Libp2pRuntimeEvent::gossip_published(
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
                let result = validate_peer_id(recipient_peer_id.as_str()).and_then(|_| {
                    state
                        .lock()
                        .map_err(|_| P2pTransportError::StateUnavailable)
                        .map(|mut locked_state| {
                            let queue = locked_state
                                .inbox_by_peer
                                .entry(recipient_peer_id)
                                .or_insert_with(VecDeque::new);
                            queue.drain(..).collect::<Vec<PeerGossipFrame>>()
                        })
                });
                let _ = response.send(result);
            }
            Libp2pNativeRuntimeAdapterLoopCommand::DrainRuntimeEvents { response } => {
                let result = state
                    .lock()
                    .map_err(|_| P2pTransportError::StateUnavailable)
                    .map(|mut locked_state| locked_state.runtime_events.drain(..).collect());
                let _ = response.send(result);
            }
        }
    }
}

#[cfg(feature = "libp2p-live-transport")]
fn run_libp2p_native_swarm_loop(
    config: P2pSwarmDeterministicConfig,
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
    swarm: &mut Swarm<Libp2pDeterministicRuntimeBehaviour>,
    frame: &PeerGossipFrame,
) -> Result<(), P2pTransportError> {
    let topic_id = canonical_libp2p_topic_id(frame.topic.as_str())?;
    let publish_topic = gossipsub::IdentTopic::new(topic_id);
    swarm
        .behaviour_mut()
        .gossipsub
        .publish(
            publish_topic,
            UdpPeerLifecycleTransport::encode_frame(frame),
        )
        .map_err(|_| P2pTransportError::LiveSocketSendFailed)?;
    Ok(())
}

#[cfg(feature = "libp2p-live-transport")]
fn apply_libp2p_swarm_event_to_live_state(
    event: libp2p::swarm::SwarmEvent<Libp2pDeterministicRuntimeBehaviourEvent>,
    state: Arc<Mutex<Libp2pLiveDataPlaneState>>,
    local_peer_id: &str,
) {
    let libp2p::swarm::SwarmEvent::Behaviour(Libp2pDeterministicRuntimeBehaviourEvent::Gossipsub(
        gossipsub::Event::Message { message, .. },
    )) = event
    else {
        return;
    };

    let frame = match UdpPeerLifecycleTransport::decode_frame(message.data.as_slice()) {
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
        if let Ok(event) = Libp2pRuntimeEvent::gossip_received(
            recipient_peer_id.as_str(),
            topic.as_str(),
            payload.as_str(),
        ) {
            locked_state.runtime_events.push_back(event);
        }
    }
}

/// Returns canonical identify protocol id for deterministic libp2p runtime composition.
pub fn canonical_libp2p_identify_protocol_id() -> &'static str {
    LIBP2P_IDENTIFY_PROTOCOL_ID
}

/// Returns canonical gossipsub topic id for deterministic runtime policy checks.
pub fn canonical_libp2p_topic_id(topic: &str) -> Result<String, P2pTransportError> {
    validate_topic(topic)?;
    Ok(format!("{LIBP2P_TOPIC_NAMESPACE}{}", topic.trim()))
}

fn build_live_data_plane_network_id(config: &P2pSwarmDeterministicConfig) -> String {
    let bootstrap_segment = if config.bootstrap_peers().is_empty() {
        format!("listen={}", config.listen_address())
    } else {
        format!("bootstrap={}", config.bootstrap_peers().join(","))
    };
    let topic_segment = format!("topics={}", config.gossip_topics().join(","));
    format!("{bootstrap_segment}|{topic_segment}")
}

#[cfg(feature = "libp2p-live-transport")]
fn validate_libp2p_native_runtime_config(
    config: &P2pSwarmDeterministicConfig,
) -> Result<(), P2pTransportError> {
    use libp2p::Multiaddr;
    config
        .listen_address()
        .parse::<Multiaddr>()
        .map_err(|_| P2pTransportError::Libp2pRuntimeConfigInvalid)?;
    for bootstrap_peer in config.bootstrap_peers() {
        bootstrap_peer
            .parse::<Multiaddr>()
            .map_err(|_| P2pTransportError::Libp2pRuntimeConfigInvalid)?;
    }
    for topic in config.gossip_topics() {
        let topic_id = canonical_libp2p_topic_id(topic.as_str())
            .map_err(|_| P2pTransportError::Libp2pRuntimeConfigInvalid)?;
        let _ = libp2p::gossipsub::IdentTopic::new(topic_id);
    }
    Ok(())
}

/// Fault classes observed during live libp2p reconnect/discovery operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiveTransportFaultClass {
    /// Dial or connection setup timed out.
    DialTimeout,
    /// Discovery backend returned unavailable/unreachable status.
    DiscoveryUnavailable,
    /// Stream churn/drop detected during reconnect sequence.
    StreamChurn,
    /// Protocol legality violation was observed (fail closed).
    ProtocolViolation,
}

impl LiveTransportFaultClass {
    fn retry_reason_code(self) -> &'static str {
        match self {
            Self::DialTimeout => "p2p_live_reconnect_retry_dial_timeout",
            Self::DiscoveryUnavailable => "p2p_live_reconnect_retry_discovery_unavailable",
            Self::StreamChurn => "p2p_live_reconnect_retry_stream_churn",
            Self::ProtocolViolation => "p2p_live_reconnect_protocol_violation",
        }
    }
}

/// Deterministic reconnect/backoff decision output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiveTransportReconnectDecision {
    /// Retry is allowed with bounded deterministic backoff.
    Retry {
        /// Backoff budget in abstract ticks.
        backoff_ticks: u16,
        /// Deterministic reason code.
        reason_code: &'static str,
    },
    /// Retry is disallowed and transport must fail closed.
    FailClosed {
        /// Deterministic reason code.
        reason_code: &'static str,
    },
}

/// Deterministic reconnect/backoff policy for live transport faults.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveTransportReconnectPolicy {
    base_backoff_ticks: u16,
    max_backoff_ticks: u16,
    max_retry_attempts: u16,
}

impl LiveTransportReconnectPolicy {
    /// Builds a validated deterministic reconnect/backoff policy.
    pub fn new(
        base_backoff_ticks: u16,
        max_backoff_ticks: u16,
        max_retry_attempts: u16,
    ) -> Result<Self, P2pTransportError> {
        if max_retry_attempts == 0 {
            return Err(P2pTransportError::InvalidReconnectRetryBudget);
        }
        if base_backoff_ticks == 0
            || max_backoff_ticks == 0
            || base_backoff_ticks > max_backoff_ticks
        {
            return Err(P2pTransportError::InvalidReconnectBackoffWindow);
        }
        Ok(Self {
            base_backoff_ticks,
            max_backoff_ticks,
            max_retry_attempts,
        })
    }

    /// Evaluates deterministic reconnect decision for one fault class + attempt index.
    pub fn evaluate(
        &self,
        fault_class: LiveTransportFaultClass,
        attempt: u16,
    ) -> LiveTransportReconnectDecision {
        if fault_class == LiveTransportFaultClass::ProtocolViolation {
            return LiveTransportReconnectDecision::FailClosed {
                reason_code: fault_class.retry_reason_code(),
            };
        }

        let normalized_attempt = attempt.max(1);
        if normalized_attempt >= self.max_retry_attempts {
            return LiveTransportReconnectDecision::FailClosed {
                reason_code: "p2p_live_reconnect_retry_budget_exhausted",
            };
        }

        LiveTransportReconnectDecision::Retry {
            backoff_ticks: self.backoff_ticks_for_attempt(normalized_attempt),
            reason_code: fault_class.retry_reason_code(),
        }
    }

    fn backoff_ticks_for_attempt(&self, attempt: u16) -> u16 {
        let mut backoff = u32::from(self.base_backoff_ticks);
        let max_backoff = u32::from(self.max_backoff_ticks);
        for _ in 1..attempt {
            if backoff >= max_backoff {
                break;
            }
            backoff = backoff.saturating_mul(2).min(max_backoff);
        }
        backoff as u16
    }
}

/// Deterministic config used to compose a libp2p swarm behavior stack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct P2pSwarmDeterministicConfig {
    local_peer_id: String,
    listen_address: String,
    bootstrap_peers: Vec<String>,
    gossip_topics: Vec<String>,
    harness_tick_budget: u16,
}

impl P2pSwarmDeterministicConfig {
    /// Builds a validated deterministic swarm configuration.
    pub fn new(
        local_peer_id: &str,
        listen_address: &str,
        bootstrap_peers: Vec<String>,
        gossip_topics: Vec<String>,
        harness_tick_budget: u16,
    ) -> Result<Self, P2pTransportError> {
        validate_peer_id(local_peer_id)?;
        validate_swarm_listen_address(listen_address)?;
        if harness_tick_budget == 0 {
            return Err(P2pTransportError::InvalidSwarmHarnessTickBudget);
        }
        if gossip_topics.is_empty() {
            return Err(P2pTransportError::MissingGossipTopics);
        }

        let mut normalized_bootstrap = BTreeSet::new();
        for peer in bootstrap_peers {
            validate_swarm_bootstrap_peer_address(peer.as_str())?;
            normalized_bootstrap.insert(peer.trim().to_owned());
        }

        let mut normalized_topics = BTreeSet::new();
        for topic in gossip_topics {
            validate_topic(topic.as_str())?;
            normalized_topics.insert(topic.trim().to_owned());
        }

        Ok(Self {
            local_peer_id: local_peer_id.to_owned(),
            listen_address: listen_address.trim().to_owned(),
            bootstrap_peers: normalized_bootstrap.into_iter().collect(),
            gossip_topics: normalized_topics.into_iter().collect(),
            harness_tick_budget,
        })
    }

    /// Returns the local peer id.
    pub fn local_peer_id(&self) -> &str {
        &self.local_peer_id
    }

    /// Returns the local listen multiaddr.
    pub fn listen_address(&self) -> &str {
        &self.listen_address
    }

    /// Returns canonical bootstrap peer multiaddrs.
    pub fn bootstrap_peers(&self) -> &[String] {
        &self.bootstrap_peers
    }

    /// Returns canonical gossip topic subscriptions.
    pub fn gossip_topics(&self) -> &[String] {
        &self.gossip_topics
    }

    /// Returns deterministic harness tick budget.
    pub fn harness_tick_budget(&self) -> u16 {
        self.harness_tick_budget
    }
}

/// Builds deterministic swarm config from node config and explicit transport inputs.
pub fn build_p2p_swarm_deterministic_config(
    node_config: &NodeConfig,
    local_peer_id: &str,
    listen_address: &str,
    bootstrap_peers: Vec<String>,
    gossip_topics: Vec<String>,
    harness_tick_budget: u16,
) -> Result<P2pSwarmDeterministicConfig, P2pTransportError> {
    if !node_config.enable_gossip {
        return Err(P2pTransportError::GossipTransportDisabled);
    }
    P2pSwarmDeterministicConfig::new(
        local_peer_id,
        listen_address,
        bootstrap_peers,
        gossip_topics,
        harness_tick_budget,
    )
}

/// Canonical behavior stack summary for deterministic libp2p runtime composition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct P2pSwarmBehaviorStack {
    listen_address: String,
    bootstrap_peers: Vec<String>,
    gossip_topics: Vec<String>,
    behavior_components: Vec<&'static str>,
    identify_protocol_id: &'static str,
    gossip_topic_namespace: &'static str,
}

impl P2pSwarmBehaviorStack {
    /// Returns canonical behavior component ordering.
    pub fn behavior_components(&self) -> Vec<&'static str> {
        self.behavior_components.clone()
    }

    /// Returns canonical gossip topic ordering.
    pub fn gossip_topics(&self) -> Vec<String> {
        self.gossip_topics.clone()
    }

    /// Returns canonical bootstrap peer ordering.
    pub fn bootstrap_peers(&self) -> Vec<String> {
        self.bootstrap_peers.clone()
    }

    /// Returns local listen multiaddr.
    pub fn listen_address(&self) -> &str {
        &self.listen_address
    }

    /// Returns canonical identify protocol id.
    pub fn identify_protocol_id(&self) -> &'static str {
        self.identify_protocol_id
    }

    /// Returns canonical topic namespace prefix used during topic normalization.
    pub fn gossip_topic_namespace(&self) -> &'static str {
        self.gossip_topic_namespace
    }
}

/// Composes deterministic libp2p behavior stack metadata.
pub fn compose_libp2p_swarm_behavior_stack(
    config: &P2pSwarmDeterministicConfig,
) -> P2pSwarmBehaviorStack {
    P2pSwarmBehaviorStack {
        listen_address: config.listen_address().to_owned(),
        bootstrap_peers: config.bootstrap_peers().to_vec(),
        gossip_topics: config.gossip_topics().to_vec(),
        behavior_components: LIBP2P_SWARM_BEHAVIOR_COMPONENTS.to_vec(),
        identify_protocol_id: canonical_libp2p_identify_protocol_id(),
        gossip_topic_namespace: LIBP2P_TOPIC_NAMESPACE,
    }
}

/// Canonical Kademlia bootstrap seed set for deterministic discovery startup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KademliaBootstrapSeedSet {
    seed_peers: Vec<String>,
}

impl KademliaBootstrapSeedSet {
    /// Builds a validated deterministic Kademlia bootstrap seed set.
    pub fn new(seed_peers: Vec<String>) -> Result<Self, P2pTransportError> {
        if seed_peers.is_empty() {
            return Err(P2pTransportError::MissingKademliaBootstrapSeeds);
        }

        let mut normalized = BTreeSet::new();
        for peer in seed_peers {
            validate_swarm_bootstrap_peer_address(peer.as_str())?;
            normalized.insert(peer.trim().to_owned());
        }

        Ok(Self {
            seed_peers: normalized.into_iter().collect(),
        })
    }

    /// Returns canonical bootstrap peer ordering.
    pub fn seed_peers(&self) -> Vec<String> {
        self.seed_peers.clone()
    }
}

/// Deterministic Kademlia discovery bootstrap plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KademliaDiscoveryBootstrapPlan {
    discovery_backend: &'static str,
    seed_peers: Vec<String>,
}

impl KademliaDiscoveryBootstrapPlan {
    /// Returns the deterministic discovery backend marker.
    pub fn discovery_backend(&self) -> &'static str {
        self.discovery_backend
    }

    /// Returns canonical Kademlia bootstrap seed ordering.
    pub fn seed_peers(&self) -> Vec<String> {
        self.seed_peers.clone()
    }
}

/// Composes deterministic Kademlia bootstrap behavior from swarm config seed peers.
pub fn compose_kademlia_discovery_bootstrap(
    config: &P2pSwarmDeterministicConfig,
) -> Result<KademliaDiscoveryBootstrapPlan, P2pTransportError> {
    let seed_set = KademliaBootstrapSeedSet::new(config.bootstrap_peers().to_vec())?;
    Ok(KademliaDiscoveryBootstrapPlan {
        discovery_backend: "kademlia",
        seed_peers: seed_set.seed_peers(),
    })
}

/// Expected outcome category for a lifecycle regression replay case.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PeerLifecycleRegressionExpectedOutcome {
    /// Replay should complete and end on the provided lifecycle state.
    FinalState(PeerLifecycleState),
    /// Replay should fail closed with the provided transition error.
    TransitionError(RuntimeLifecycleError),
}

/// Deterministic lifecycle regression replay case.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerLifecycleRegressionCase {
    case_id: String,
    events: Vec<PeerLifecycleEvent>,
    expected_outcome: PeerLifecycleRegressionExpectedOutcome,
}

impl PeerLifecycleRegressionCase {
    /// Builds a validated lifecycle regression replay case.
    pub fn new(
        case_id: &str,
        events: Vec<PeerLifecycleEvent>,
        expected_outcome: PeerLifecycleRegressionExpectedOutcome,
    ) -> Result<Self, PeerLifecycleRegressionError> {
        if case_id.trim().is_empty() {
            return Err(PeerLifecycleRegressionError::EmptyCaseId);
        }
        if events.is_empty() {
            return Err(PeerLifecycleRegressionError::EmptyEventSequence);
        }
        Ok(Self {
            case_id: case_id.to_owned(),
            events,
            expected_outcome,
        })
    }

    /// Returns deterministic replay case id.
    pub fn case_id(&self) -> &str {
        &self.case_id
    }

    /// Returns replay event sequence.
    pub fn events(&self) -> &[PeerLifecycleEvent] {
        &self.events
    }

    /// Returns expected replay outcome.
    pub fn expected_outcome(&self) -> &PeerLifecycleRegressionExpectedOutcome {
        &self.expected_outcome
    }
}

/// Deterministic lifecycle regression replay outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerLifecycleRegressionOutcome {
    case_id: String,
    final_state: Option<PeerLifecycleState>,
    transition_error_reason_code: Option<&'static str>,
}

impl PeerLifecycleRegressionOutcome {
    /// Returns replay case id.
    pub fn case_id(&self) -> &str {
        &self.case_id
    }

    /// Returns final lifecycle state when replay succeeded.
    pub fn final_state(&self) -> Option<PeerLifecycleState> {
        self.final_state
    }

    /// Returns deterministic transition error reason code when replay failed.
    pub fn transition_error_reason_code(&self) -> Option<&'static str> {
        self.transition_error_reason_code
    }
}

/// Lifecycle regression replay error variants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PeerLifecycleRegressionError {
    /// Case id is empty.
    EmptyCaseId,
    /// Event sequence is empty.
    EmptyEventSequence,
    /// Lifecycle construction or transition returned a runtime error.
    Lifecycle(RuntimeLifecycleError),
    /// Final lifecycle state differs from expected deterministic state.
    ExpectedFinalStateMismatch {
        /// Case id.
        case_id: String,
        /// Expected state.
        expected: PeerLifecycleState,
        /// Observed state.
        found: PeerLifecycleState,
    },
    /// Transition error occurred when case expected a final-state result.
    UnexpectedTransitionError {
        /// Case id.
        case_id: String,
        /// Observed transition error.
        found: RuntimeLifecycleError,
    },
    /// Expected transition-error contract differs from observed result.
    ExpectedTransitionErrorMismatch {
        /// Case id.
        case_id: String,
        /// Expected transition error.
        expected: RuntimeLifecycleError,
        /// Observed transition error, if one occurred.
        found: Option<RuntimeLifecycleError>,
    },
}

impl Display for PeerLifecycleRegressionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyCaseId => write!(f, "lifecycle regression case id cannot be empty"),
            Self::EmptyEventSequence => write!(f, "lifecycle regression event sequence cannot be empty"),
            Self::Lifecycle(error) => write!(f, "{error}"),
            Self::ExpectedFinalStateMismatch {
                case_id,
                expected,
                found,
            } => write!(
                f,
                "lifecycle regression case {case_id} expected final state {expected:?}, found {found:?}"
            ),
            Self::UnexpectedTransitionError { case_id, found } => write!(
                f,
                "lifecycle regression case {case_id} observed unexpected transition error {found:?}"
            ),
            Self::ExpectedTransitionErrorMismatch {
                case_id, expected, found
            } => write!(
                f,
                "lifecycle regression case {case_id} expected transition error {expected:?}, found {found:?}"
            ),
        }
    }
}

impl Error for PeerLifecycleRegressionError {}

/// Builds deterministic default lifecycle regression corpus for libp2p transport transitions.
pub fn build_libp2p_lifecycle_regression_corpus() -> Vec<PeerLifecycleRegressionCase> {
    vec![
        PeerLifecycleRegressionCase {
            case_id: "connect_handshake_disconnect".to_owned(),
            events: vec![
                PeerLifecycleEvent::StartConnect,
                PeerLifecycleEvent::HandshakeSucceeded,
                PeerLifecycleEvent::Disconnect,
            ],
            expected_outcome: PeerLifecycleRegressionExpectedOutcome::FinalState(
                PeerLifecycleState::Disconnected,
            ),
        },
        PeerLifecycleRegressionCase {
            case_id: "connect_heartbeat_timeout_recovery".to_owned(),
            events: vec![
                PeerLifecycleEvent::StartConnect,
                PeerLifecycleEvent::HandshakeSucceeded,
                PeerLifecycleEvent::HeartbeatMissed,
                PeerLifecycleEvent::HeartbeatRestored,
            ],
            expected_outcome: PeerLifecycleRegressionExpectedOutcome::FinalState(
                PeerLifecycleState::Active,
            ),
        },
        PeerLifecycleRegressionCase {
            case_id: "connect_drop_rejoin".to_owned(),
            events: vec![
                PeerLifecycleEvent::StartConnect,
                PeerLifecycleEvent::HandshakeSucceeded,
                PeerLifecycleEvent::Disconnect,
                PeerLifecycleEvent::Rejoin,
                PeerLifecycleEvent::HandshakeSucceeded,
            ],
            expected_outcome: PeerLifecycleRegressionExpectedOutcome::FinalState(
                PeerLifecycleState::Active,
            ),
        },
        PeerLifecycleRegressionCase {
            case_id: "invalid_heartbeat_from_disconnected".to_owned(),
            events: vec![PeerLifecycleEvent::HeartbeatMissed],
            expected_outcome: PeerLifecycleRegressionExpectedOutcome::TransitionError(
                RuntimeLifecycleError::InvalidTransition {
                    from: PeerLifecycleState::Disconnected,
                    event: PeerLifecycleEvent::HeartbeatMissed,
                },
            ),
        },
    ]
}

/// Replays one deterministic lifecycle regression case.
pub fn run_libp2p_lifecycle_regression_case(
    peer_id: &str,
    case: &PeerLifecycleRegressionCase,
) -> Result<PeerLifecycleRegressionOutcome, PeerLifecycleRegressionError> {
    let mut lifecycle =
        PeerLifecycle::new(peer_id).map_err(PeerLifecycleRegressionError::Lifecycle)?;

    let mut observed_error = None;
    let mut observed_state = lifecycle.state();
    for event in case.events() {
        match lifecycle.transition(*event) {
            Ok(next_state) => observed_state = next_state,
            Err(error) => {
                observed_error = Some(error);
                break;
            }
        }
    }

    match case.expected_outcome() {
        PeerLifecycleRegressionExpectedOutcome::FinalState(expected) => {
            if let Some(error) = observed_error {
                return Err(PeerLifecycleRegressionError::UnexpectedTransitionError {
                    case_id: case.case_id().to_owned(),
                    found: error,
                });
            }
            if &observed_state != expected {
                return Err(PeerLifecycleRegressionError::ExpectedFinalStateMismatch {
                    case_id: case.case_id().to_owned(),
                    expected: *expected,
                    found: observed_state,
                });
            }
            Ok(PeerLifecycleRegressionOutcome {
                case_id: case.case_id().to_owned(),
                final_state: Some(observed_state),
                transition_error_reason_code: None,
            })
        }
        PeerLifecycleRegressionExpectedOutcome::TransitionError(expected_error) => {
            let Some(found_error) = observed_error else {
                return Err(
                    PeerLifecycleRegressionError::ExpectedTransitionErrorMismatch {
                        case_id: case.case_id().to_owned(),
                        expected: expected_error.clone(),
                        found: None,
                    },
                );
            };
            if &found_error != expected_error {
                return Err(
                    PeerLifecycleRegressionError::ExpectedTransitionErrorMismatch {
                        case_id: case.case_id().to_owned(),
                        expected: expected_error.clone(),
                        found: Some(found_error),
                    },
                );
            }
            Ok(PeerLifecycleRegressionOutcome {
                case_id: case.case_id().to_owned(),
                final_state: None,
                transition_error_reason_code: Some(found_error.reason_code()),
            })
        }
    }
}

/// Replays deterministic lifecycle regression corpus in the provided order.
pub fn run_libp2p_lifecycle_regression_corpus(
    peer_id: &str,
    corpus: &[PeerLifecycleRegressionCase],
) -> Result<Vec<PeerLifecycleRegressionOutcome>, PeerLifecycleRegressionError> {
    let mut outcomes = Vec::with_capacity(corpus.len());
    for case in corpus {
        outcomes.push(run_libp2p_lifecycle_regression_case(peer_id, case)?);
    }
    Ok(outcomes)
}

/// Swarm harness execution mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P2pSwarmHarnessMode {
    /// Build and validate deterministic stack without running loop ticks.
    DryRun,
    /// Start deterministic runtime harness and execute bounded loop ticks.
    Run,
}

/// Deterministic swarm harness startup report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct P2pSwarmHarnessReport {
    mode: P2pSwarmHarnessMode,
    started: bool,
    executed_ticks: u16,
    bootstrap_peer_count: usize,
    behavior_components: Vec<&'static str>,
}

impl P2pSwarmHarnessReport {
    /// Returns harness mode.
    pub fn mode(&self) -> P2pSwarmHarnessMode {
        self.mode
    }

    /// Returns whether run mode started the deterministic loop.
    pub fn started(&self) -> bool {
        self.started
    }

    /// Returns deterministic executed tick count.
    pub fn executed_ticks(&self) -> u16 {
        self.executed_ticks
    }

    /// Returns canonical bootstrap peer count.
    pub fn bootstrap_peer_count(&self) -> usize {
        self.bootstrap_peer_count
    }

    /// Returns canonical behavior stack ordering used during startup.
    pub fn behavior_components(&self) -> Vec<&'static str> {
        self.behavior_components.clone()
    }
}

/// Runtime harness wrapper used to start deterministic swarm loops in tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct P2pSwarmHarnessTask {
    config: P2pSwarmDeterministicConfig,
    stack: P2pSwarmBehaviorStack,
}

impl P2pSwarmHarnessTask {
    /// Builds a deterministic harness task for the provided swarm config.
    pub fn new(config: P2pSwarmDeterministicConfig) -> Self {
        let stack = compose_libp2p_swarm_behavior_stack(&config);
        Self { config, stack }
    }

    /// Starts deterministic harness mode and returns startup report.
    pub fn start(
        &self,
        mode: P2pSwarmHarnessMode,
    ) -> Result<P2pSwarmHarnessReport, P2pTransportError> {
        let started = matches!(mode, P2pSwarmHarnessMode::Run);
        let executed_ticks = if started {
            self.config.harness_tick_budget()
        } else {
            0
        };
        let behavior_components = self.stack.behavior_components();
        #[cfg(feature = "libp2p-live-transport")]
        let mut behavior_components = behavior_components;
        #[cfg(feature = "libp2p-live-transport")]
        if started {
            validate_libp2p_runtime_stack_composition(&self.config)?;
            behavior_components.push("libp2p-runtime-swarm");
        }
        Ok(P2pSwarmHarnessReport {
            mode,
            started,
            executed_ticks,
            bootstrap_peer_count: self.config.bootstrap_peers().len(),
            behavior_components,
        })
    }
}

#[cfg(feature = "libp2p-live-transport")]
pub(super) fn build_libp2p_runtime_swarm(
    config: &P2pSwarmDeterministicConfig,
) -> Result<Swarm<Libp2pDeterministicRuntimeBehaviour>, P2pTransportError> {
    let swarm = SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )
        .map_err(|_| P2pTransportError::Libp2pRuntimeConfigInvalid)?
        .with_behaviour(|key| {
            let gossipsub_config = gossipsub::ConfigBuilder::default()
                .validation_mode(gossipsub::ValidationMode::Permissive)
                .build()
                .map_err(|_| P2pTransportError::Libp2pRuntimeConfigInvalid)?;
            let mut gossipsub_behavior = gossipsub::Behaviour::new(
                gossipsub::MessageAuthenticity::Signed(key.clone()),
                gossipsub_config,
            )
            .map_err(|_| P2pTransportError::Libp2pRuntimeConfigInvalid)?;
            for topic in config.gossip_topics() {
                let topic_id = canonical_libp2p_topic_id(topic.as_str())
                    .map_err(|_| P2pTransportError::Libp2pRuntimeConfigInvalid)?;
                gossipsub_behavior
                    .subscribe(&gossipsub::IdentTopic::new(topic_id))
                    .map_err(|_| P2pTransportError::Libp2pRuntimeConfigInvalid)?;
            }
            Ok(Libp2pDeterministicRuntimeBehaviour {
                gossipsub: gossipsub_behavior,
                identify: identify::Behaviour::new(identify::Config::new(
                    canonical_libp2p_identify_protocol_id().to_owned(),
                    key.public(),
                )),
            })
        })
        .map_err(|_| P2pTransportError::Libp2pRuntimeConfigInvalid)?
        .build();
    Ok(swarm)
}

#[cfg(feature = "libp2p-live-transport")]
pub(super) fn apply_libp2p_runtime_network_config(
    swarm: &mut Swarm<Libp2pDeterministicRuntimeBehaviour>,
    config: &P2pSwarmDeterministicConfig,
) -> Result<(), P2pTransportError> {
    let listen_multiaddr = config
        .listen_address()
        .parse::<Multiaddr>()
        .map_err(|_| P2pTransportError::Libp2pRuntimeConfigInvalid)?;
    Swarm::listen_on(swarm, listen_multiaddr)
        .map_err(|_| P2pTransportError::Libp2pRuntimeConfigInvalid)?;

    for bootstrap_peer in config.bootstrap_peers() {
        let bootstrap_multiaddr = bootstrap_peer
            .parse::<Multiaddr>()
            .map_err(|_| P2pTransportError::Libp2pRuntimeConfigInvalid)?;
        let _ = Swarm::dial(swarm, bootstrap_multiaddr);
    }
    Ok(())
}

#[cfg(feature = "libp2p-live-transport")]
pub(super) fn validate_libp2p_runtime_stack_composition(
    config: &P2pSwarmDeterministicConfig,
) -> Result<(), P2pTransportError> {
    let config = config.clone();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .map_err(|_| P2pTransportError::Libp2pRuntimeConfigInvalid)?;

    runtime.block_on(async move {
        let mut swarm = build_libp2p_runtime_swarm(&config)?;
        apply_libp2p_runtime_network_config(&mut swarm, &config)?;
        Ok(())
    })
}

#[cfg(feature = "libp2p-live-transport")]
pub(super) fn runtime_channel_closed_behavior_failure_class(
    operation: Libp2pRuntimeAdapterOperation,
) -> Libp2pBehaviorFailureClass {
    match operation {
        Libp2pRuntimeAdapterOperation::Connect => {
            Libp2pBehaviorFailureClass::RuntimeConnectChannelClosed
        }
        Libp2pRuntimeAdapterOperation::Discover => {
            Libp2pBehaviorFailureClass::RuntimeDiscoverChannelClosed
        }
        Libp2pRuntimeAdapterOperation::Publish => {
            Libp2pBehaviorFailureClass::RuntimePublishChannelClosed
        }
        Libp2pRuntimeAdapterOperation::Receive => {
            Libp2pBehaviorFailureClass::RuntimeReceiveChannelClosed
        }
        Libp2pRuntimeAdapterOperation::EventDrain => {
            Libp2pBehaviorFailureClass::RuntimeEventDrainChannelClosed
        }
    }
}
