use super::models::BlockPipelineError;

/// Durable commit checker lane mode used for CI/local boundary enforcement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DurableCommitCheckerLaneMode {
    /// Low-cost CI smoke lane mode.
    CiSmoke,
    /// Opt-in local-heavy lane mode.
    LocalHeavy,
}

/// Deterministic lane-boundary report for durable commit checker enforcement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DurableCommitCheckerLaneBoundaryReport {
    /// Final gate decision for the requested lane.
    pub final_decision: String,
    /// Status marker describing CI/local-heavy boundary enforcement.
    pub ci_smoke_local_heavy_boundary_status: String,
    /// Cost profile assigned to the CI smoke lane.
    pub ci_smoke_lane_cost_profile: String,
    /// Execution mode selected for the local-heavy lane.
    pub local_heavy_lane_execution_mode: String,
    /// Deterministic reason code emitted by the boundary decision.
    pub enforcement_reason_code: String,
}

/// Enforces the durable-commit-checker lane boundary for CI-smoke versus local-heavy runs.
pub fn enforce_durable_commit_checker_lane_boundary(
    lane_mode: DurableCommitCheckerLaneMode,
    ci_fast_gate: bool,
    local_heavy_opt_in: bool,
) -> Result<DurableCommitCheckerLaneBoundaryReport, BlockPipelineError> {
    match lane_mode {
        DurableCommitCheckerLaneMode::CiSmoke => ci_smoke_boundary(ci_fast_gate),
        DurableCommitCheckerLaneMode::LocalHeavy => {
            local_heavy_boundary(ci_fast_gate, local_heavy_opt_in)
        }
    }
}

fn ci_smoke_boundary(
    ci_fast_gate: bool,
) -> Result<DurableCommitCheckerLaneBoundaryReport, BlockPipelineError> {
    if !ci_fast_gate {
        return Err(BlockPipelineError::ReplayDrift {
            reason_code: "durable_commit_checker_ci_smoke_fast_gate_required".to_owned(),
            detail: "ci-smoke durable commit checker mode requires ci-fast-gate PASS".to_owned(),
        });
    }
    Ok(verified_lane_boundary(
        "not-applicable",
        "durable_commit_checker_ci_smoke_boundary_verified",
    ))
}

fn local_heavy_boundary(
    ci_fast_gate: bool,
    local_heavy_opt_in: bool,
) -> Result<DurableCommitCheckerLaneBoundaryReport, BlockPipelineError> {
    if ci_fast_gate {
        return Err(BlockPipelineError::ReplayDrift {
            reason_code: "durable_commit_checker_local_heavy_ci_fast_gate_mismatch".to_owned(),
            detail:
                "local-heavy durable commit checker mode must remain excluded from ci-fast-gate"
                    .to_owned(),
        });
    }
    if !local_heavy_opt_in {
        return Err(BlockPipelineError::ReplayDrift {
            reason_code: "durable_commit_checker_local_heavy_opt_in_required".to_owned(),
            detail: "local-heavy durable commit checker mode requires explicit local opt-in"
                .to_owned(),
        });
    }
    Ok(verified_lane_boundary(
        "opt_in",
        "durable_commit_checker_local_heavy_boundary_verified",
    ))
}

fn verified_lane_boundary(
    execution_mode: &str,
    enforcement_reason_code: &str,
) -> DurableCommitCheckerLaneBoundaryReport {
    DurableCommitCheckerLaneBoundaryReport {
        final_decision: "GO".to_owned(),
        ci_smoke_local_heavy_boundary_status: "verified".to_owned(),
        ci_smoke_lane_cost_profile: "low".to_owned(),
        local_heavy_lane_execution_mode: execution_mode.to_owned(),
        enforcement_reason_code: enforcement_reason_code.to_owned(),
    }
}
