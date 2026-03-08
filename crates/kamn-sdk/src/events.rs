use crate::{SdkError, ServiceRouteEvent};

/// Stable SDK view of one websocket event frame from the service route.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceEventSnapshot {
    /// Event name.
    pub event: String,
    /// Runtime mode marker.
    pub runtime_mode: String,
    /// Node role marker.
    pub role: String,
    /// Event sequence identifier.
    pub sequence: u64,
}

impl From<ServiceRouteEvent> for ServiceEventSnapshot {
    fn from(event: ServiceRouteEvent) -> Self {
        Self {
            event: event.event,
            runtime_mode: event.runtime_mode,
            role: event.role,
            sequence: event.sequence,
        }
    }
}

/// Public one-shot service event capabilities exposed by SDK transports.
pub trait KamnServiceEvents {
    /// Reads one event frame from the service websocket route.
    fn read_service_event(&self) -> Result<ServiceEventSnapshot, SdkError>;
}
