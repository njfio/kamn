//! Deterministic fairness and starvation policy checker contracts.

/// Deterministic fairness policy reason taxonomy version marker.
pub const FAIRNESS_POLICY_REASON_TAXONOMY_VERSION: &str =
    "kamn.runtime.fairness-policy-reason-taxonomy.v1";
/// Deterministic fairness policy reason code marker list.
pub const FAIRNESS_POLICY_REASON_CODES_CSV: &str =
    "fairness_scope_unknown,fairness_window_non_positive,fairness_max_gap_non_positive,fairness_weighted_share_exceeds_gap";

const ALLOWED_SCOPE_CLASSES: &[&str] = &["control_plane", "tenant_interactive", "bulk_replication"];

/// Input payload for deterministic fairness-policy evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FairnessPolicyInput {
    /// Scope class being evaluated.
    pub scope: String,
    /// Evaluation window in seconds.
    pub window_seconds: u64,
    /// Observed weighted-share gap for the scope in the same window.
    pub active_weighted_share: u64,
    /// Maximum allowed weighted-share gap before starvation is declared.
    pub max_weighted_share_gap: u64,
}

/// Deterministic fail-closed reason emitted by fairness policy checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FairnessPolicyViolationReason {
    /// Scope class is unknown and cannot be evaluated safely.
    ScopeUnknown,
    /// Window is invalid (`<= 0`) and cannot be evaluated safely.
    WindowNonPositive,
    /// Maximum weighted-share gap is invalid (`<= 0`) and cannot be evaluated safely.
    MaxGapNonPositive,
    /// Observed weighted-share gap exceeds allowed gap and indicates starvation.
    WeightedShareExceedsGap,
}

impl FairnessPolicyViolationReason {
    /// Returns the deterministic reason-code marker.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ScopeUnknown => "fairness_scope_unknown",
            Self::WindowNonPositive => "fairness_window_non_positive",
            Self::MaxGapNonPositive => "fairness_max_gap_non_positive",
            Self::WeightedShareExceedsGap => "fairness_weighted_share_exceeds_gap",
        }
    }
}

/// Deterministic decision emitted by fairness policy checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FairnessPolicyDecision {
    /// Input satisfied fairness policy constraints.
    Allow,
    /// Input violated fairness policy constraints and is rejected fail closed.
    Reject {
        /// Deterministic fail-closed violation reason.
        reason: FairnessPolicyViolationReason,
    },
}

/// Returns the deterministic fairness policy reason taxonomy version marker.
pub fn fairness_policy_reason_taxonomy_version() -> &'static str {
    FAIRNESS_POLICY_REASON_TAXONOMY_VERSION
}

/// Returns the deterministic fairness policy reason-code marker list.
pub fn fairness_policy_reason_codes_csv() -> &'static str {
    FAIRNESS_POLICY_REASON_CODES_CSV
}

/// Evaluates a fairness policy input and returns a deterministic fail-closed decision.
pub fn evaluate_fairness_policy(input: &FairnessPolicyInput) -> FairnessPolicyDecision {
    if !ALLOWED_SCOPE_CLASSES.contains(&input.scope.as_str()) {
        return FairnessPolicyDecision::Reject {
            reason: FairnessPolicyViolationReason::ScopeUnknown,
        };
    }
    if input.window_seconds == 0 {
        return FairnessPolicyDecision::Reject {
            reason: FairnessPolicyViolationReason::WindowNonPositive,
        };
    }
    if input.max_weighted_share_gap == 0 {
        return FairnessPolicyDecision::Reject {
            reason: FairnessPolicyViolationReason::MaxGapNonPositive,
        };
    }
    if input.active_weighted_share > input.max_weighted_share_gap {
        return FairnessPolicyDecision::Reject {
            reason: FairnessPolicyViolationReason::WeightedShareExceedsGap,
        };
    }
    FairnessPolicyDecision::Allow
}
