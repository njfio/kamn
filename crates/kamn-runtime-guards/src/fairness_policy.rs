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

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_input() -> FairnessPolicyInput {
        FairnessPolicyInput {
            scope: "control_plane".to_owned(),
            window_seconds: 60,
            active_weighted_share: 3,
            max_weighted_share_gap: 5,
        }
    }

    #[test]
    fn spec_c05_fairness_policy_rejects_unknown_scope() {
        let mut input = valid_input();
        input.scope = "unknown".to_owned();
        let decision = evaluate_fairness_policy(&input);
        assert_eq!(
            decision,
            FairnessPolicyDecision::Reject {
                reason: FairnessPolicyViolationReason::ScopeUnknown,
            }
        );
        assert_eq!(
            FairnessPolicyViolationReason::ScopeUnknown.as_str(),
            "fairness_scope_unknown"
        );
    }

    #[test]
    fn spec_c06_fairness_policy_rejects_non_positive_window() {
        let mut input = valid_input();
        input.window_seconds = 0;
        let decision = evaluate_fairness_policy(&input);
        assert_eq!(
            decision,
            FairnessPolicyDecision::Reject {
                reason: FairnessPolicyViolationReason::WindowNonPositive,
            }
        );
        assert_eq!(
            FairnessPolicyViolationReason::WindowNonPositive.as_str(),
            "fairness_window_non_positive"
        );
    }

    #[test]
    fn spec_c07_fairness_policy_rejects_non_positive_max_gap() {
        let mut input = valid_input();
        input.max_weighted_share_gap = 0;
        let decision = evaluate_fairness_policy(&input);
        assert_eq!(
            decision,
            FairnessPolicyDecision::Reject {
                reason: FairnessPolicyViolationReason::MaxGapNonPositive,
            }
        );
        assert_eq!(
            FairnessPolicyViolationReason::MaxGapNonPositive.as_str(),
            "fairness_max_gap_non_positive"
        );
    }

    #[test]
    fn spec_c08_fairness_policy_rejects_gap_exceeded_and_allows_boundary() {
        let mut over_gap = valid_input();
        over_gap.active_weighted_share = 6;
        let over_gap_decision = evaluate_fairness_policy(&over_gap);
        assert_eq!(
            over_gap_decision,
            FairnessPolicyDecision::Reject {
                reason: FairnessPolicyViolationReason::WeightedShareExceedsGap,
            }
        );
        assert_eq!(
            FairnessPolicyViolationReason::WeightedShareExceedsGap.as_str(),
            "fairness_weighted_share_exceeds_gap"
        );

        let mut boundary = valid_input();
        boundary.active_weighted_share = boundary.max_weighted_share_gap;
        assert_eq!(
            evaluate_fairness_policy(&boundary),
            FairnessPolicyDecision::Allow
        );
    }

    #[test]
    fn fairness_policy_reason_helpers_expose_deterministic_markers() {
        assert_eq!(
            fairness_policy_reason_taxonomy_version(),
            "kamn.runtime.fairness-policy-reason-taxonomy.v1"
        );
        assert_eq!(
            fairness_policy_reason_codes_csv(),
            "fairness_scope_unknown,fairness_window_non_positive,fairness_max_gap_non_positive,fairness_weighted_share_exceeds_gap"
        );
    }
}
