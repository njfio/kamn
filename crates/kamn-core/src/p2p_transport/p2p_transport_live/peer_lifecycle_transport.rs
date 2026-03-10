#![allow(missing_docs)]

use super::deterministic_config::{
    P2pSwarmDeterministicConfig, P2pSwarmHarnessMode, P2pSwarmHarnessReport, P2pSwarmHarnessTask,
};
#[cfg(feature = "libp2p-live-transport")]
use super::native_runtime_loop::Libp2pNativeRuntimeAdapterLoop;
use super::runtime_inbox::{build_live_data_plane_network_id, resolve_live_data_plane_state};
#[cfg(not(feature = "libp2p-live-transport"))]
use super::runtime_inbox::{Libp2pLiveDataPlane, Libp2pLiveDataPlaneState};
#[cfg(feature = "libp2p-live-transport")]
use super::swarm_runtime::validate_libp2p_native_runtime_config;
use super::*;

#[cfg(not(feature = "libp2p-live-transport"))]
mod contract_data_plane;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Libp2pLiveRuntimeBackend {
    ContractDataPlane,
    NativeSocket,
}

impl Libp2pLiveRuntimeBackend {
    pub fn marker(self) -> &'static str {
        match self {
            Self::ContractDataPlane => "contract-data-plane",
            Self::NativeSocket => "native-libp2p-swarm",
        }
    }
}

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

    pub fn transport_profile(&self) -> RuntimeTransportProfile {
        RuntimeTransportProfile::Libp2pLive
    }

    pub fn harness_report(&self) -> &P2pSwarmHarnessReport {
        &self.harness_report
    }

    pub fn runtime_backend(&self) -> Libp2pLiveRuntimeBackend {
        resolve_libp2p_live_runtime_backend()
    }

    pub fn listen_address(&self) -> &str {
        self.swarm_config.listen_address()
    }

    pub fn live_data_plane_network_id(&self) -> &str {
        self.live_network_id.as_str()
    }

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
        {
            contract_data_plane::advertise(self, record)
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
        {
            contract_data_plane::discover(self, requester_peer_id, topic)
        }
    }

    fn send(&self, frame: PeerGossipFrame) -> Result<(), P2pTransportError> {
        #[cfg(feature = "libp2p-live-transport")]
        {
            self.native_runtime_loop.send(frame)
        }
        #[cfg(not(feature = "libp2p-live-transport"))]
        {
            contract_data_plane::send(self, frame)
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
        {
            contract_data_plane::drain_inbox(self, recipient_peer_id)
        }
    }
}
