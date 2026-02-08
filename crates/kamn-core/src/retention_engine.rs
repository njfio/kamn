use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RetentionDomain {
    Messages,
    Tasks,
    Escrows,
    Reputation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RetentionClass {
    MaxAgeSeconds(u64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetentionEnginePolicy {
    pub default_class: RetentionClass,
    pub overrides: BTreeMap<RetentionDomain, RetentionClass>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetentionRecord {
    pub domain: RetentionDomain,
    pub record_id: String,
    pub created_at_secs: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetentionStatus {
    pub domain: RetentionDomain,
    pub record_id: String,
    pub class: RetentionClass,
    pub expires_at_secs: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetentionEvaluation {
    pub expired_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetentionPolicyEngine {
    policy: RetentionEnginePolicy,
    blocked_ids: BTreeSet<String>,
}

impl RetentionPolicyEngine {
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
pub enum RetentionPolicyError {
    InvalidRetentionClass(u64),
    EmptyField(&'static str),
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
