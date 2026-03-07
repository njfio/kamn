use super::{
    DataLayerShellNeutralPolicyDecision, DataLayerShellNeutralPolicyError,
    DataLayerShellNeutralPolicyInput, DataLayerShellNeutralPolicyReasonCode,
    DataLayerShellNeutralPolicyReport,
};

/// Evaluates shell-neutral orchestration and ratio-budget policy compliance.
pub fn data_layer_evaluate_shell_neutral_policy(
    input: DataLayerShellNeutralPolicyInput,
) -> Result<DataLayerShellNeutralPolicyReport, DataLayerShellNeutralPolicyError> {
    validate_thresholds(&input)?;

    let reason_codes = resolve_reason_codes(&input);
    let decision = resolve_decision(reason_codes.as_slice());
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

fn validate_thresholds(
    input: &DataLayerShellNeutralPolicyInput,
) -> Result<(), DataLayerShellNeutralPolicyError> {
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
    Ok(())
}

fn resolve_reason_codes(
    input: &DataLayerShellNeutralPolicyInput,
) -> Vec<DataLayerShellNeutralPolicyReasonCode> {
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
    if !blocked_reasons.is_empty() {
        return blocked_reasons;
    }
    if input.current_shell_to_rust_ratio > input.warn_shell_to_rust_ratio_max {
        return vec![DataLayerShellNeutralPolicyReasonCode::WarnRatioThreshold];
    }
    vec![DataLayerShellNeutralPolicyReasonCode::Verified]
}

fn resolve_decision(
    reason_codes: &[DataLayerShellNeutralPolicyReasonCode],
) -> DataLayerShellNeutralPolicyDecision {
    match reason_codes.first() {
        Some(DataLayerShellNeutralPolicyReasonCode::Verified) => {
            DataLayerShellNeutralPolicyDecision::Verified
        }
        Some(DataLayerShellNeutralPolicyReasonCode::WarnRatioThreshold) => {
            DataLayerShellNeutralPolicyDecision::Warning
        }
        Some(_) => DataLayerShellNeutralPolicyDecision::Blocked,
        None => DataLayerShellNeutralPolicyDecision::Verified,
    }
}
