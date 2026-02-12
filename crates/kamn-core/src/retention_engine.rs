//! Retention policy contracts and expiration evaluation helpers.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Logical domain used to select retention rules for a record.
pub enum RetentionDomain {
    /// Message domain records.
    Messages,
    /// Task domain records.
    Tasks,
    /// Escrow domain records.
    Escrows,
    /// Reputation domain records.
    Reputation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// Retention rule class applied to a record.
pub enum RetentionClass {
    /// Maximum record age in seconds.
    MaxAgeSeconds(u64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Retention policy with default and per-domain class overrides.
pub struct RetentionEnginePolicy {
    /// Default class used when no domain override exists.
    pub default_class: RetentionClass,
    /// Domain-specific retention class overrides.
    pub overrides: BTreeMap<RetentionDomain, RetentionClass>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Canonical retention record input evaluated by the policy engine.
pub struct RetentionRecord {
    /// Domain the record belongs to.
    pub domain: RetentionDomain,
    /// Stable identifier of the retained record.
    pub record_id: String,
    /// Record creation timestamp (epoch seconds).
    pub created_at_secs: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Retention status projection for a specific record.
pub struct RetentionStatus {
    /// Domain of the evaluated record.
    pub domain: RetentionDomain,
    /// Identifier of the evaluated record.
    pub record_id: String,
    /// Effective retention class used for evaluation.
    pub class: RetentionClass,
    /// Expiration timestamp (epoch seconds).
    pub expires_at_secs: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Batch evaluation output containing expired record identifiers.
pub struct RetentionEvaluation {
    /// Record IDs that expired during this evaluation cycle.
    pub expired_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// In-memory retention policy engine with resurfaced-record protection.
pub struct RetentionPolicyEngine {
    policy: RetentionEnginePolicy,
    blocked_ids: BTreeSet<String>,
}

impl RetentionPolicyEngine {
    /// Builds a retention policy engine after validating all classes.
    pub fn new(policy: RetentionEnginePolicy) -> Result<Self, RetentionPolicyError> {
        validate_class(policy.default_class)?;
        for class in policy.overrides.values() {
            validate_class(*class)?;
        }

        Ok(Self {
            policy,
            blocked_ids: BTreeSet::new(),
        })
    }

    /// Evaluates records at `now_secs` and returns records that expired.
    pub fn evaluate(
        &mut self,
        now_secs: u64,
        mut records: Vec<RetentionRecord>,
    ) -> Result<RetentionEvaluation, RetentionPolicyError> {
        records.sort_by(|left, right| {
            left.domain
                .cmp(&right.domain)
                .then_with(|| left.record_id.cmp(&right.record_id))
                .then_with(|| left.created_at_secs.cmp(&right.created_at_secs))
        });

        let mut expired_ids = Vec::new();

        for record in records {
            if self.blocked_ids.contains(&record.record_id) {
                return Err(RetentionPolicyError::ResurfacedExpiredRecord(
                    record.record_id,
                ));
            }

            let status = self.status_for(&record)?;
            if now_secs > status.expires_at_secs {
                self.blocked_ids.insert(record.record_id.clone());
                expired_ids.push(record.record_id);
            }
        }

        Ok(RetentionEvaluation { expired_ids })
    }

    /// Computes retention status for a single record.
    pub fn status_for(
        &self,
        record: &RetentionRecord,
    ) -> Result<RetentionStatus, RetentionPolicyError> {
        validate_non_empty("record_id", &record.record_id)?;

        let class = self
            .policy
            .overrides
            .get(&record.domain)
            .copied()
            .unwrap_or(self.policy.default_class);
        let max_age = class_max_age_secs(class)?;

        Ok(RetentionStatus {
            domain: record.domain,
            record_id: record.record_id.clone(),
            class,
            expires_at_secs: record.created_at_secs.saturating_add(max_age),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Error taxonomy for retention policy validation and evaluation failures.
pub enum RetentionPolicyError {
    /// Retention class has an invalid maximum age.
    InvalidRetentionClass(u64),
    /// Required field was empty after trimming.
    EmptyField(&'static str),
    /// Previously expired record resurfaced in a later evaluation.
    ResurfacedExpiredRecord(String),
}

impl fmt::Display for RetentionPolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRetentionClass(value) => {
                write!(f, "retention max age must be greater than zero: {value}")
            }
            Self::EmptyField(field) => write!(f, "field must not be empty: {field}"),
            Self::ResurfacedExpiredRecord(value) => {
                write!(f, "expired record attempted to resurface: {value}")
            }
        }
    }
}

impl std::error::Error for RetentionPolicyError {}

fn validate_class(class: RetentionClass) -> Result<(), RetentionPolicyError> {
    let max_age = class_max_age_secs(class)?;
    if max_age == 0 {
        return Err(RetentionPolicyError::InvalidRetentionClass(max_age));
    }
    Ok(())
}

fn class_max_age_secs(class: RetentionClass) -> Result<u64, RetentionPolicyError> {
    match class {
        RetentionClass::MaxAgeSeconds(value) => {
            if value == 0 {
                return Err(RetentionPolicyError::InvalidRetentionClass(value));
            }
            Ok(value)
        }
    }
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<(), RetentionPolicyError> {
    if value.trim().is_empty() {
        return Err(RetentionPolicyError::EmptyField(field));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        RetentionClass, RetentionDomain, RetentionEnginePolicy, RetentionPolicyEngine,
        RetentionPolicyError, RetentionRecord,
    };
    use std::collections::BTreeMap;

    fn base_policy() -> RetentionEnginePolicy {
        RetentionEnginePolicy {
            default_class: RetentionClass::MaxAgeSeconds(300),
            overrides: BTreeMap::new(),
        }
    }

    #[test]
    fn rejects_zero_age_class() {
        assert_eq!(
            RetentionPolicyEngine::new(RetentionEnginePolicy {
                default_class: RetentionClass::MaxAgeSeconds(0),
                overrides: BTreeMap::new(),
            }),
            Err(RetentionPolicyError::InvalidRetentionClass(0))
        );
    }

    #[test]
    fn empty_record_id_is_rejected() {
        let engine = RetentionPolicyEngine::new(base_policy()).expect("engine should construct");
        assert_eq!(
            engine.status_for(&RetentionRecord {
                domain: RetentionDomain::Tasks,
                record_id: "".to_owned(),
                created_at_secs: 1,
            }),
            Err(RetentionPolicyError::EmptyField("record_id"))
        );
    }

    #[test]
    fn expired_record_is_blocked_on_resurface() {
        let mut engine =
            RetentionPolicyEngine::new(base_policy()).expect("engine should construct");

        let first = engine
            .evaluate(
                1_000,
                vec![RetentionRecord {
                    domain: RetentionDomain::Tasks,
                    record_id: "task-1".to_owned(),
                    created_at_secs: 1,
                }],
            )
            .expect("first evaluation should succeed");
        assert_eq!(first.expired_ids, vec!["task-1".to_owned()]);

        assert_eq!(
            engine.evaluate(
                1_001,
                vec![RetentionRecord {
                    domain: RetentionDomain::Tasks,
                    record_id: "task-1".to_owned(),
                    created_at_secs: 999,
                }],
            ),
            Err(RetentionPolicyError::ResurfacedExpiredRecord(
                "task-1".to_owned()
            ))
        );
    }
}
