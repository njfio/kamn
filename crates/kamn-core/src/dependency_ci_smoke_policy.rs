//! Deterministic dependency CI smoke checker policy contracts.

/// Deterministic dependency CI smoke checker reason taxonomy version marker.
pub const DEPENDENCY_CI_SMOKE_REASON_TAXONOMY_VERSION: &str =
    "kamn.ci.dependency-ci-smoke-reason-taxonomy.v1";
/// Deterministic dependency CI smoke checker reason code marker list.
pub const DEPENDENCY_CI_SMOKE_REASON_CODES_CSV: &str =
    "dependency_advisory_input_empty,dependency_advisory_severity_unknown,dependency_advisory_threshold_exceeded";

/// Dependency advisory record evaluated by the CI smoke checker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyAdvisoryRecord {
    /// Dependency package identifier.
    pub package: String,
    /// Normalized advisory severity label.
    pub severity: String,
}

/// Input payload for dependency CI smoke policy evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyCiSmokePolicyInput {
    /// Maximum advisory severity allowed by CI smoke policy.
    pub threshold_max_severity: String,
    /// Advisory records collected for this CI smoke evaluation.
    pub advisories: Vec<DependencyAdvisoryRecord>,
}

/// Deterministic fail-closed reason emitted by dependency CI smoke checker evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencyCiSmokeViolationReason {
    /// Advisory input set is empty and cannot be evaluated safely.
    AdvisoryInputEmpty,
    /// Advisory severity or threshold severity is unknown.
    AdvisorySeverityUnknown,
    /// Advisory severity exceeded configured threshold.
    AdvisoryThresholdExceeded,
}

impl DependencyCiSmokeViolationReason {
    /// Returns the deterministic reason-code marker.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AdvisoryInputEmpty => "dependency_advisory_input_empty",
            Self::AdvisorySeverityUnknown => "dependency_advisory_severity_unknown",
            Self::AdvisoryThresholdExceeded => "dependency_advisory_threshold_exceeded",
        }
    }
}

/// Deterministic decision emitted by dependency CI smoke checker evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencyCiSmokeDecision {
    /// Input satisfied dependency CI smoke policy constraints.
    Allow,
    /// Input violated policy constraints and is rejected fail closed.
    Reject {
        /// Deterministic fail-closed violation reason.
        reason: DependencyCiSmokeViolationReason,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum AdvisorySeverity {
    Low,
    Moderate,
    High,
    Critical,
}

/// Returns the deterministic dependency CI smoke policy reason taxonomy version marker.
pub fn dependency_ci_smoke_reason_taxonomy_version() -> &'static str {
    DEPENDENCY_CI_SMOKE_REASON_TAXONOMY_VERSION
}

/// Returns the deterministic dependency CI smoke policy reason-code marker list.
pub fn dependency_ci_smoke_reason_codes_csv() -> &'static str {
    DEPENDENCY_CI_SMOKE_REASON_CODES_CSV
}

/// Evaluates dependency advisories against threshold policy and returns deterministic decisions.
pub fn evaluate_dependency_ci_smoke_policy(
    input: &DependencyCiSmokePolicyInput,
) -> DependencyCiSmokeDecision {
    if input.advisories.is_empty() {
        return DependencyCiSmokeDecision::Reject {
            reason: DependencyCiSmokeViolationReason::AdvisoryInputEmpty,
        };
    }

    let Some(threshold_max_severity) = parse_advisory_severity(&input.threshold_max_severity)
    else {
        return DependencyCiSmokeDecision::Reject {
            reason: DependencyCiSmokeViolationReason::AdvisorySeverityUnknown,
        };
    };

    for advisory in &input.advisories {
        let Some(advisory_severity) = parse_advisory_severity(&advisory.severity) else {
            return DependencyCiSmokeDecision::Reject {
                reason: DependencyCiSmokeViolationReason::AdvisorySeverityUnknown,
            };
        };

        if advisory_severity > threshold_max_severity {
            return DependencyCiSmokeDecision::Reject {
                reason: DependencyCiSmokeViolationReason::AdvisoryThresholdExceeded,
            };
        }
    }

    DependencyCiSmokeDecision::Allow
}

fn parse_advisory_severity(value: &str) -> Option<AdvisorySeverity> {
    match value {
        "low" => Some(AdvisorySeverity::Low),
        "moderate" => Some(AdvisorySeverity::Moderate),
        "high" => Some(AdvisorySeverity::High),
        "critical" => Some(AdvisorySeverity::Critical),
        _ => None,
    }
}
