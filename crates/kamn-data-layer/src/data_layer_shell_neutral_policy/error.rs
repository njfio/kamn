/// Fail-closed parsing errors for shell-neutral reason-code markers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerShellNeutralPolicyReasonCodeParseError {
    /// Marker is unknown and cannot be mapped to a typed reason.
    UnknownReasonCode(String),
}

impl std::fmt::Display for DataLayerShellNeutralPolicyReasonCodeParseError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownReasonCode(value) => {
                write!(
                    formatter,
                    "unknown shell-neutral policy reason code: {value}"
                )
            }
        }
    }
}

impl std::error::Error for DataLayerShellNeutralPolicyReasonCodeParseError {}

/// Fail-closed error taxonomy for shell-neutral policy contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerShellNeutralPolicyError {
    /// Threshold values are invalid (negative, zero, or not finite).
    InvalidThresholdValue,
    /// Threshold order is invalid (`warn` must be strictly lower than `fail`).
    InvalidThresholdOrder,
}

impl std::fmt::Display for DataLayerShellNeutralPolicyError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidThresholdValue => {
                write!(formatter, "invalid shell/rust ratio threshold value")
            }
            Self::InvalidThresholdOrder => {
                write!(formatter, "invalid shell/rust ratio threshold ordering")
            }
        }
    }
}

impl std::error::Error for DataLayerShellNeutralPolicyError {}
