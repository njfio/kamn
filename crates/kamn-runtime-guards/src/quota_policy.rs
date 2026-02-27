//! Deterministic per-scope quota policy checker contracts.

/// Deterministic quota policy reason taxonomy version marker.
pub const QUOTA_POLICY_REASON_TAXONOMY_VERSION: &str =
    "kamn.runtime.quota-policy-reason-taxonomy.v1";
/// Deterministic quota policy reason code marker list.
pub const QUOTA_POLICY_REASON_CODES_CSV: &str =
    "quota_scope_unknown,quota_window_non_positive,quota_limit_non_positive,quota_limit_exceeded";

const ALLOWED_SCOPE_CLASSES: &[&str] = &["processor_ingress", "peer_sync", "channel_broadcast"];

/// Input payload for deterministic quota-policy evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuotaPolicyInput {
    /// Scope class being evaluated.
    pub scope: String,
    /// Evaluation window in seconds.
    pub window_seconds: u64,
    /// Maximum allowed operations in the window.
    pub limit: u64,
    /// Observed operation count in the same window.
    pub observed_count: u64,
}

/// Deterministic fail-closed reason emitted by quota policy checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuotaPolicyViolationReason {
    /// Scope class is unknown and cannot be evaluated safely.
    ScopeUnknown,
    /// Window is invalid (`<= 0`) and cannot be evaluated safely.
    QuotaWindowNonPositive,
    /// Limit is invalid (`<= 0`) and cannot be evaluated safely.
    QuotaLimitNonPositive,
    /// Observed value exceeded configured limit.
    QuotaLimitExceeded,
}

impl QuotaPolicyViolationReason {
    /// Returns the deterministic reason-code marker.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ScopeUnknown => "quota_scope_unknown",
            Self::QuotaWindowNonPositive => "quota_window_non_positive",
            Self::QuotaLimitNonPositive => "quota_limit_non_positive",
            Self::QuotaLimitExceeded => "quota_limit_exceeded",
        }
    }
}

/// Deterministic decision emitted by quota policy checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuotaPolicyDecision {
    /// Input satisfied quota policy constraints.
    Allow,
    /// Input violated quota policy constraints and is rejected fail closed.
    Reject {
        /// Deterministic fail-closed violation reason.
        reason: QuotaPolicyViolationReason,
    },
}

/// Returns the deterministic quota policy reason taxonomy version marker.
pub fn quota_policy_reason_taxonomy_version() -> &'static str {
    QUOTA_POLICY_REASON_TAXONOMY_VERSION
}

/// Returns the deterministic quota policy reason-code marker list.
pub fn quota_policy_reason_codes_csv() -> &'static str {
    QUOTA_POLICY_REASON_CODES_CSV
}

/// Evaluates a quota policy input and returns a deterministic fail-closed decision.
pub fn evaluate_quota_policy(input: &QuotaPolicyInput) -> QuotaPolicyDecision {
    if !ALLOWED_SCOPE_CLASSES.contains(&input.scope.as_str()) {
        return QuotaPolicyDecision::Reject {
            reason: QuotaPolicyViolationReason::ScopeUnknown,
        };
    }
    if input.window_seconds == 0 {
        return QuotaPolicyDecision::Reject {
            reason: QuotaPolicyViolationReason::QuotaWindowNonPositive,
        };
    }
    if input.limit == 0 {
        return QuotaPolicyDecision::Reject {
            reason: QuotaPolicyViolationReason::QuotaLimitNonPositive,
        };
    }
    if input.observed_count > input.limit {
        return QuotaPolicyDecision::Reject {
            reason: QuotaPolicyViolationReason::QuotaLimitExceeded,
        };
    }
    QuotaPolicyDecision::Allow
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_input() -> QuotaPolicyInput {
        QuotaPolicyInput {
            scope: "processor_ingress".to_owned(),
            window_seconds: 60,
            limit: 10,
            observed_count: 5,
        }
    }

    #[test]
    fn spec_c01_quota_policy_rejects_unknown_scope() {
        let mut input = valid_input();
        input.scope = "unknown".to_owned();
        let decision = evaluate_quota_policy(&input);
        assert_eq!(
            decision,
            QuotaPolicyDecision::Reject {
                reason: QuotaPolicyViolationReason::ScopeUnknown,
            }
        );
        assert_eq!(
            QuotaPolicyViolationReason::ScopeUnknown.as_str(),
            "quota_scope_unknown"
        );
    }

    #[test]
    fn spec_c02_quota_policy_rejects_non_positive_window() {
        let mut input = valid_input();
        input.window_seconds = 0;
        let decision = evaluate_quota_policy(&input);
        assert_eq!(
            decision,
            QuotaPolicyDecision::Reject {
                reason: QuotaPolicyViolationReason::QuotaWindowNonPositive,
            }
        );
        assert_eq!(
            QuotaPolicyViolationReason::QuotaWindowNonPositive.as_str(),
            "quota_window_non_positive"
        );
    }

    #[test]
    fn spec_c03_quota_policy_rejects_non_positive_limit() {
        let mut input = valid_input();
        input.limit = 0;
        let decision = evaluate_quota_policy(&input);
        assert_eq!(
            decision,
            QuotaPolicyDecision::Reject {
                reason: QuotaPolicyViolationReason::QuotaLimitNonPositive,
            }
        );
        assert_eq!(
            QuotaPolicyViolationReason::QuotaLimitNonPositive.as_str(),
            "quota_limit_non_positive"
        );
    }

    #[test]
    fn spec_c04_quota_policy_rejects_limit_exceeded_and_allows_boundary() {
        let mut over_limit = valid_input();
        over_limit.observed_count = 11;
        let over_limit_decision = evaluate_quota_policy(&over_limit);
        assert_eq!(
            over_limit_decision,
            QuotaPolicyDecision::Reject {
                reason: QuotaPolicyViolationReason::QuotaLimitExceeded,
            }
        );
        assert_eq!(
            QuotaPolicyViolationReason::QuotaLimitExceeded.as_str(),
            "quota_limit_exceeded"
        );

        let mut boundary = valid_input();
        boundary.observed_count = boundary.limit;
        assert_eq!(evaluate_quota_policy(&boundary), QuotaPolicyDecision::Allow);
    }

    #[test]
    fn quota_policy_reason_helpers_expose_deterministic_markers() {
        assert_eq!(
            quota_policy_reason_taxonomy_version(),
            "kamn.runtime.quota-policy-reason-taxonomy.v1"
        );
        assert_eq!(
            quota_policy_reason_codes_csv(),
            "quota_scope_unknown,quota_window_non_positive,quota_limit_non_positive,quota_limit_exceeded"
        );
    }
}
