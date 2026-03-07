use std::fmt;

use super::DataLayerPrdCriticalScenarioMode;

/// Fail-closed error taxonomy for critical-scenario conformance contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerPrdCriticalScenarioConformanceError {
    /// Required field was empty.
    EmptyField(&'static str),
    /// Scenario id is not within required set (`62..71`).
    InvalidScenarioId(u8),
    /// Attempted update mutates a previously recorded result.
    InvalidResultMutation {
        /// Scenario identifier being updated.
        scenario_id: u8,
        /// Existing `passed` value.
        existing_passed: bool,
        /// Requested `passed` value.
        requested_passed: bool,
        /// Existing mode.
        existing_mode: DataLayerPrdCriticalScenarioMode,
        /// Requested mode.
        requested_mode: DataLayerPrdCriticalScenarioMode,
        /// Stable reason marker.
        reason_code: &'static str,
    },
}

impl fmt::Display for DataLayerPrdCriticalScenarioConformanceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field_name) => write!(formatter, "{field_name} must not be empty"),
            Self::InvalidScenarioId(scenario_id) => {
                write!(formatter, "invalid PRD critical scenario id: {scenario_id}")
            }
            Self::InvalidResultMutation {
                scenario_id,
                existing_passed,
                requested_passed,
                existing_mode,
                requested_mode,
                reason_code,
            } => write!(
                formatter,
                "invalid result mutation for scenario {scenario_id}: passed({existing_passed}->{requested_passed}) mode({existing_mode:?}->{requested_mode:?}) ({reason_code})"
            ),
        }
    }
}

impl std::error::Error for DataLayerPrdCriticalScenarioConformanceError {}
