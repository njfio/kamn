mod agent;
mod agent_mutations;
mod config;
mod routes;
mod state;
mod task_escrow;

pub use self::config::LiveTransportConfig;
use self::state::LiveTransportState;
use crate::{KamnTransport, SdkError, ServiceApiClient, TransportMode};
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
}

impl KamnTransport for LiveTransportKamnClient {
    fn transport_mode(&self) -> TransportMode {
        TransportMode::Live
    }
}
