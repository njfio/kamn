//! Shell-neutral orchestration and ratio-budget policy contracts.
//!
//! This module evaluates shell-neutral evidence and ratio-budget markers using
//! deterministic decision semantics and fail-closed threshold validation.

use crate::DataLayerPrdCriticalScenarioConformanceReport;

/// Stable reason marker when shell-neutral policy is fully verified.
pub const DATA_LAYER_SHELL_NEUTRAL_POLICY_VERIFIED_REASON_CODE: &str =
    "shell_neutral_policy_verified";
/// Stable reason marker when orchestration evidence contains shell-mode violations.
pub const DATA_LAYER_SHELL_NEUTRAL_POLICY_BLOCK_ORCHESTRATION_REASON_CODE: &str =
    "shell_neutral_policy_block_orchestration_violation";
/// Stable reason marker when shell LOC delta is positive.
pub const DATA_LAYER_SHELL_NEUTRAL_POLICY_BLOCK_SHELL_DELTA_REASON_CODE: &str =
    "shell_neutral_policy_block_positive_shell_delta";
/// Stable reason marker when shell/rust ratio exceeds fail threshold.
pub const DATA_LAYER_SHELL_NEUTRAL_POLICY_BLOCK_RATIO_FAIL_REASON_CODE: &str =
    "shell_neutral_policy_block_ratio_fail_threshold";
/// Stable reason marker when shell/rust ratio exceeds warn threshold but not fail threshold.
pub const DATA_LAYER_SHELL_NEUTRAL_POLICY_WARN_RATIO_REASON_CODE: &str =
    "shell_neutral_policy_warn_ratio_threshold";

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
    /// Stable reason markers for the decision.
    pub reason_codes: Vec<&'static str>,
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
        blocked_reasons.push(DATA_LAYER_SHELL_NEUTRAL_POLICY_BLOCK_ORCHESTRATION_REASON_CODE);
    }
    if input.shell_loc_delta_actual > 0 {
        blocked_reasons.push(DATA_LAYER_SHELL_NEUTRAL_POLICY_BLOCK_SHELL_DELTA_REASON_CODE);
    }
    if input.current_shell_to_rust_ratio > input.fail_shell_to_rust_ratio_max {
        blocked_reasons.push(DATA_LAYER_SHELL_NEUTRAL_POLICY_BLOCK_RATIO_FAIL_REASON_CODE);
    }

    let (decision, reason_codes) = if !blocked_reasons.is_empty() {
        (
            DataLayerShellNeutralPolicyDecision::Blocked,
            blocked_reasons,
        )
    } else if input.current_shell_to_rust_ratio > input.warn_shell_to_rust_ratio_max {
        (
            DataLayerShellNeutralPolicyDecision::Warning,
            vec![DATA_LAYER_SHELL_NEUTRAL_POLICY_WARN_RATIO_REASON_CODE],
        )
    } else {
        (
            DataLayerShellNeutralPolicyDecision::Verified,
            vec![DATA_LAYER_SHELL_NEUTRAL_POLICY_VERIFIED_REASON_CODE],
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
