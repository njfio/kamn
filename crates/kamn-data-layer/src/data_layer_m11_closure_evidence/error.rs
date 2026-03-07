/// Fail-closed error taxonomy for closure-evidence contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerM11ClosureEvidenceError {
    /// Release marker is empty.
    EmptyReleaseMarker,
}

impl std::fmt::Display for DataLayerM11ClosureEvidenceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyReleaseMarker => write!(formatter, "release_marker must not be empty"),
        }
    }
}

impl std::error::Error for DataLayerM11ClosureEvidenceError {}
