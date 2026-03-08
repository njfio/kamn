use crate::ServiceHealthStatus;

/// Stable SDK view of the service health route.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceHealthSnapshot {
    /// Service health marker.
    pub status: String,
    /// Runtime mode marker.
    pub runtime_mode: String,
    /// Node role marker.
    pub role: String,
    /// Observability source marker.
    pub observability_source: String,
    /// Observability health marker.
    pub observability_health: String,
}

impl From<ServiceHealthStatus> for ServiceHealthSnapshot {
    fn from(health: ServiceHealthStatus) -> Self {
        Self {
            status: health.status,
            runtime_mode: health.runtime_mode,
            role: health.role,
            observability_source: health.observability_source,
            observability_health: health.observability_health,
        }
    }
}
