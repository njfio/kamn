use super::error::GovernanceWorkflowError;
use super::models::GovernanceParameterChangeDraft;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct SemanticVersion {
    major: u64,
    minor: u64,
    patch: u64,
}

impl SemanticVersion {
    fn parse(value: &str) -> Option<Self> {
        let mut parts = value.split('.');
        let major = parts.next()?.parse().ok()?;
        let minor = parts.next()?.parse().ok()?;
        let patch = parts.next()?.parse().ok()?;
        if parts.next().is_some() {
            return None;
        }
        Some(Self {
            major,
            minor,
            patch,
        })
    }

    fn canonical_string(self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ParameterPolicySpec {
    key: &'static str,
    min_value: u64,
    max_value: u64,
    min_supported_version: SemanticVersion,
}

const PARAMETER_POLICY_CATALOG: [ParameterPolicySpec; 3] = [
    ParameterPolicySpec {
        key: "listener.quorum",
        min_value: 1,
        max_value: 7,
        min_supported_version: SemanticVersion {
            major: 1,
            minor: 0,
            patch: 0,
        },
    },
    ParameterPolicySpec {
        key: "approver.required_approvals",
        min_value: 1,
        max_value: 7,
        min_supported_version: SemanticVersion {
            major: 1,
            minor: 0,
            patch: 0,
        },
    },
    ParameterPolicySpec {
        key: "watchdog.delivery_ratio_bps",
        min_value: 9000,
        max_value: 9999,
        min_supported_version: SemanticVersion {
            major: 1,
            minor: 1,
            patch: 0,
        },
    },
];

pub(super) fn require_non_empty(
    field: &'static str,
    value: &str,
) -> Result<(), GovernanceWorkflowError> {
    if value.trim().is_empty() {
        return Err(GovernanceWorkflowError::EmptyField(field));
    }
    Ok(())
}

pub(super) fn validate_parameter_change(
    parameter_change: &GovernanceParameterChangeDraft,
) -> Result<(), GovernanceWorkflowError> {
    require_non_empty("parameter_change.key", &parameter_change.key)?;
    require_non_empty(
        "parameter_change.target_version",
        &parameter_change.target_version,
    )?;
    let target_version =
        SemanticVersion::parse(&parameter_change.target_version).ok_or_else(|| {
            GovernanceWorkflowError::InvalidParameterTargetVersion(
                parameter_change.target_version.clone(),
            )
        })?;
    if parameter_change.min_value > parameter_change.max_value {
        return Err(GovernanceWorkflowError::InvalidParameterRange {
            key: parameter_change.key.clone(),
            min_value: parameter_change.min_value,
            max_value: parameter_change.max_value,
        });
    }
    let policy = parameter_policy_for_key(&parameter_change.key).ok_or_else(|| {
        GovernanceWorkflowError::UnknownParameterKey(parameter_change.key.clone())
    })?;
    if target_version < policy.min_supported_version {
        return Err(GovernanceWorkflowError::ParameterUnsupportedForVersion {
            key: parameter_change.key.clone(),
            target_version: parameter_change.target_version.clone(),
            min_supported_version: policy.min_supported_version.canonical_string(),
        });
    }
    if parameter_change.min_value < policy.min_value
        || parameter_change.max_value > policy.max_value
    {
        return Err(GovernanceWorkflowError::ParameterRangeOutsidePolicy {
            key: parameter_change.key.clone(),
            min_value: parameter_change.min_value,
            max_value: parameter_change.max_value,
            policy_min_value: policy.min_value,
            policy_max_value: policy.max_value,
        });
    }
    if parameter_change.proposed_value < parameter_change.min_value
        || parameter_change.proposed_value > parameter_change.max_value
    {
        return Err(GovernanceWorkflowError::ParameterOutOfBounds {
            key: parameter_change.key.clone(),
            proposed_value: parameter_change.proposed_value,
            min_value: parameter_change.min_value,
            max_value: parameter_change.max_value,
        });
    }
    Ok(())
}

fn parameter_policy_for_key(key: &str) -> Option<&'static ParameterPolicySpec> {
    PARAMETER_POLICY_CATALOG
        .iter()
        .find(|policy| policy.key == key)
}
