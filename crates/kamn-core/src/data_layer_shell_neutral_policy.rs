//! Shell-neutral orchestration and ratio-budget policy contracts.
//!
//! This module evaluates shell-neutral evidence and ratio-budget markers using
//! deterministic decision semantics and fail-closed threshold validation.

use crate::DataLayerPrdCriticalScenarioConformanceReport;

const SHELL_NEUTRAL_POLICY_VERIFIED_REASON: &str = "shell_neutral_policy_verified";
const SHELL_NEUTRAL_POLICY_BLOCK_ORCHESTRATION_REASON: &str =
    "shell_neutral_policy_block_orchestration_violation";
const SHELL_NEUTRAL_POLICY_BLOCK_SHELL_DELTA_REASON: &str =
    "shell_neutral_policy_block_positive_shell_delta";
const SHELL_NEUTRAL_POLICY_BLOCK_RATIO_FAIL_REASON: &str =
    "shell_neutral_policy_block_ratio_fail_threshold";
const SHELL_NEUTRAL_POLICY_WARN_RATIO_REASON: &str = "shell_neutral_policy_warn_ratio_threshold";

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
    /// Returns canonical reason-code marker consumed at wire/telemetry boundaries.
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

/// Fail-closed error taxonomy for shell-neutral policy contracts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataLayerShellNeutralPolicyError {
    /// Threshold values are invalid (negative/zero/not-finite).
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

impl DataLayerShellNeutralPolicyReport {
    /// Returns canonical reason-code markers for wire/telemetry compatibility.
    pub fn reason_code_strings(&self) -> Vec<&'static str> {
        self.reason_codes
            .iter()
            .map(|reason| reason.as_str())
            .collect()
    }
}

/// Evaluates shell-neutral orchestration and ratio-budget policy compliance.
pub fn data_layer_evaluate_shell_neutral_policy(
    input: DataLayerShellNeutralPolicyInput,
) -> Result<DataLayerShellNeutralPolicyReport, DataLayerShellNeutralPolicyError> {
    if !input.warn_shell_to_rust_ratio_max.is_finite()
        || !input.fail_shell_to_rust_ratio_max.is_finite()
        || input.warn_shell_to_rust_ratio_max <= 0.0
        || input.fail_shell_to_rust_ratio_max <= 0.0
    {
        return Err(DataLayerShellNeutralPolicyError::InvalidThresholdValue);
    }
    if input.warn_shell_to_rust_ratio_max >= input.fail_shell_to_rust_ratio_max {
        return Err(DataLayerShellNeutralPolicyError::InvalidThresholdOrder);
    }

    let mut blocked_reasons = Vec::new();
    if !input
        .critical_scenario_report
        .shell_policy_violation_scenario_ids
        .is_empty()
    {
        blocked_reasons.push(DataLayerShellNeutralPolicyReasonCode::BlockOrchestrationViolation);
    }
    if input.shell_loc_delta_actual > 0 {
        blocked_reasons.push(DataLayerShellNeutralPolicyReasonCode::BlockPositiveShellDelta);
    }
    if input.current_shell_to_rust_ratio > input.fail_shell_to_rust_ratio_max {
        blocked_reasons.push(DataLayerShellNeutralPolicyReasonCode::BlockRatioFailThreshold);
    }

    let (decision, reason_codes) = if !blocked_reasons.is_empty() {
        (
            DataLayerShellNeutralPolicyDecision::Blocked,
            blocked_reasons,
        )
    } else if input.current_shell_to_rust_ratio > input.warn_shell_to_rust_ratio_max {
        (
            DataLayerShellNeutralPolicyDecision::Warning,
            vec![DataLayerShellNeutralPolicyReasonCode::WarnRatioThreshold],
        )
    } else {
        (
            DataLayerShellNeutralPolicyDecision::Verified,
            vec![DataLayerShellNeutralPolicyReasonCode::Verified],
        )
    };

    Ok(DataLayerShellNeutralPolicyReport {
        decision,
        reason_codes,
        shell_loc_delta_actual: input.shell_loc_delta_actual,
        rust_loc_delta_actual: input.rust_loc_delta_actual,
        current_shell_to_rust_ratio: input.current_shell_to_rust_ratio,
        warn_shell_to_rust_ratio_max: input.warn_shell_to_rust_ratio_max,
        fail_shell_to_rust_ratio_max: input.fail_shell_to_rust_ratio_max,
    })
}
