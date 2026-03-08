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
