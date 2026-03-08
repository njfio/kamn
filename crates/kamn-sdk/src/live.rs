mod agent;
mod agent_mutations;
mod bridge;
mod config;
mod events;
mod observability;
mod routes;
mod state;
mod task_escrow;

pub use self::config::LiveTransportConfig;
use self::state::LiveTransportState;
use crate::{
    KamnServiceEvents, KamnServiceObservability, KamnTransport, SdkError, ServiceApiClient,
    ServiceEventSnapshot, ServiceHealthSnapshot, TransportMode,
};
use std::sync::{Arc, Mutex};

/// Live transport client backed by the Service API.
#[derive(Debug, Clone)]
pub struct LiveTransportKamnClient {
    config: LiveTransportConfig,
    service_client: ServiceApiClient,
    state: Arc<Mutex<LiveTransportState>>,
}

impl LiveTransportKamnClient {
    /// Connects to a service endpoint and returns a live transport client.
    pub fn connect(endpoint: &str) -> Result<Self, SdkError> {
        let config = LiveTransportConfig::new(endpoint)?;
        let service_client = ServiceApiClient::connect(config.endpoint.as_str())?;
        Ok(Self {
            config,
            service_client,
            state: Arc::new(Mutex::new(LiveTransportState::default())),
        })
    }
    /// Returns the configured endpoint for this client.
    pub fn endpoint(&self) -> &str {
        &self.config.endpoint
    }

    /// Reads the service health route through the live SDK transport.
    pub fn service_health(&self) -> Result<ServiceHealthSnapshot, SdkError> {
        <Self as KamnServiceObservability>::service_health(self)
    }

    /// Reads raw service metrics exposition text through the live SDK transport.
    pub fn service_metrics(&self) -> Result<String, SdkError> {
        <Self as KamnServiceObservability>::service_metrics(self)
    }

    /// Reads one event frame from the service websocket route through the live SDK transport.
    pub fn read_service_event(&self) -> Result<ServiceEventSnapshot, SdkError> {
        <Self as KamnServiceEvents>::read_service_event(self)
    }
}

impl KamnTransport for LiveTransportKamnClient {
    fn transport_mode(&self) -> TransportMode {
        TransportMode::Live
    }
}

impl KamnServiceObservability for LiveTransportKamnClient {
    fn service_health(&self) -> Result<ServiceHealthSnapshot, SdkError> {
        self::observability::service_health(self)
    }

    fn service_metrics(&self) -> Result<String, SdkError> {
        self::observability::service_metrics(self)
    }
}

impl KamnServiceEvents for LiveTransportKamnClient {
    fn read_service_event(&self) -> Result<ServiceEventSnapshot, SdkError> {
        self::events::read_service_event(self)
    }
}
