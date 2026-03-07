const SHELL_NEUTRAL_POLICY_VERIFIED_REASON: &str = "shell_neutral_policy_verified";
const SHELL_NEUTRAL_POLICY_BLOCK_ORCHESTRATION_REASON: &str =
    "shell_neutral_policy_block_orchestration_violation";
const SHELL_NEUTRAL_POLICY_BLOCK_SHELL_DELTA_REASON: &str =
    "shell_neutral_policy_block_positive_shell_delta";
const SHELL_NEUTRAL_POLICY_BLOCK_RATIO_FAIL_REASON: &str =
    "shell_neutral_policy_block_ratio_fail_threshold";
const SHELL_NEUTRAL_POLICY_WARN_RATIO_REASON: &str = "shell_neutral_policy_warn_ratio_threshold";

use crate::DataLayerPrdCriticalScenarioConformanceReport;

use super::DataLayerShellNeutralPolicyReasonCodeParseError;

/// Canonical reason-code vocabulary for shell-neutral policy decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerShellNeutralPolicyReasonCode {
    /// Policy is fully verified.
    Verified,
    /// Orchestration evidence includes shell-mode violations.
    BlockOrchestrationViolation,
    /// Shell LOC delta is positive.
    BlockPositiveShellDelta,
    /// Shell/rust ratio exceeds fail threshold.
    BlockRatioFailThreshold,
    /// Shell/rust ratio exceeds warn threshold but not fail threshold.
    WarnRatioThreshold,
}

impl DataLayerShellNeutralPolicyReasonCode {
    /// Returns canonical reason-code marker consumed at wire and telemetry boundaries.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Verified => SHELL_NEUTRAL_POLICY_VERIFIED_REASON,
            Self::BlockOrchestrationViolation => SHELL_NEUTRAL_POLICY_BLOCK_ORCHESTRATION_REASON,
            Self::BlockPositiveShellDelta => SHELL_NEUTRAL_POLICY_BLOCK_SHELL_DELTA_REASON,
            Self::BlockRatioFailThreshold => SHELL_NEUTRAL_POLICY_BLOCK_RATIO_FAIL_REASON,
            Self::WarnRatioThreshold => SHELL_NEUTRAL_POLICY_WARN_RATIO_REASON,
        }
    }
}

impl std::str::FromStr for DataLayerShellNeutralPolicyReasonCode {
    type Err = DataLayerShellNeutralPolicyReasonCodeParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            SHELL_NEUTRAL_POLICY_VERIFIED_REASON => Ok(Self::Verified),
            SHELL_NEUTRAL_POLICY_BLOCK_ORCHESTRATION_REASON => {
                Ok(Self::BlockOrchestrationViolation)
            }
            SHELL_NEUTRAL_POLICY_BLOCK_SHELL_DELTA_REASON => Ok(Self::BlockPositiveShellDelta),
            SHELL_NEUTRAL_POLICY_BLOCK_RATIO_FAIL_REASON => Ok(Self::BlockRatioFailThreshold),
            SHELL_NEUTRAL_POLICY_WARN_RATIO_REASON => Ok(Self::WarnRatioThreshold),
            _ => Err(
                DataLayerShellNeutralPolicyReasonCodeParseError::UnknownReasonCode(
                    value.to_owned(),
                ),
            ),
        }
    }
}

/// Decision output for shell-neutral policy evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerShellNeutralPolicyDecision {
    /// Policy is compliant and verified.
    Verified,
    /// Policy is compliant but ratio enters warning band.
    Warning,
    /// Policy is non-compliant and blocked.
    Blocked,
}

/// Input envelope for shell-neutral policy evaluation.
#[derive(Debug, Clone, PartialEq)]
pub struct DataLayerShellNeutralPolicyInput {
    /// Critical scenario conformance report containing orchestration evidence.
    pub critical_scenario_report: DataLayerPrdCriticalScenarioConformanceReport,
    /// Measured shell LOC delta for the issue.
    pub shell_loc_delta_actual: i32,
    /// Measured rust LOC delta for the issue.
    pub rust_loc_delta_actual: i32,
    /// Current shell/rust ratio marker.
    pub current_shell_to_rust_ratio: f64,
    /// Warn threshold for shell/rust ratio.
    pub warn_shell_to_rust_ratio_max: f64,
    /// Fail threshold for shell/rust ratio.
    pub fail_shell_to_rust_ratio_max: f64,
}

/// Deterministic shell-neutral policy report.
#[derive(Debug, Clone, PartialEq)]
pub struct DataLayerShellNeutralPolicyReport {
    /// Final policy decision.
    pub decision: DataLayerShellNeutralPolicyDecision,
    /// Stable typed reason markers for the decision.
    pub reason_codes: Vec<DataLayerShellNeutralPolicyReasonCode>,
    /// Echoed shell LOC delta marker.
    pub shell_loc_delta_actual: i32,
    /// Echoed rust LOC delta marker.
    pub rust_loc_delta_actual: i32,
    /// Echoed current ratio marker.
    pub current_shell_to_rust_ratio: f64,
    /// Echoed warn threshold marker.
    pub warn_shell_to_rust_ratio_max: f64,
    /// Echoed fail threshold marker.
    pub fail_shell_to_rust_ratio_max: f64,
}

impl DataLayerShellNeutralPolicyReport {
    /// Returns canonical reason-code markers for wire and telemetry compatibility.
    pub fn reason_code_strings(&self) -> Vec<&'static str> {
        self.reason_codes
            .iter()
            .map(|reason| reason.as_str())
            .collect()
    }
}
