//! Data classification policy contracts for write authorization and status reporting.

use crate::{canonical_state_key, AgentDid};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Ordered classification level applied to domain writes.
pub enum DataClassificationLevel {
    /// Lowest-sensitivity data suitable for broad visibility.
    Public,
    /// Internal-only data that should stay within trusted operators.
    Internal,
    /// Sensitive data requiring explicit tags and tighter controls.
    Sensitive,
    /// Highest-sensitivity data requiring strict controls.
    Restricted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// Write domain used for minimum classification policy lookup.
pub enum WriteDomain {
    /// Message and channel payload domain.
    Messages,
    /// Task and workflow record domain.
    Tasks,
    /// Escrow and settlement state domain.
    Escrows,
    /// Reputation and trust signal domain.
    Reputation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Classification policy defining minimum levels and required tags.
pub struct ClassificationPolicy {
    /// Minimum classification level required per write domain.
    pub minimum_by_domain: BTreeMap<WriteDomain, DataClassificationLevel>,
    /// Required tags keyed by classification level.
    pub required_tags_by_level: BTreeMap<DataClassificationLevel, BTreeSet<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Classification label and tags attached to a write request.
pub struct WriteTag {
    /// Provided classification level for the write.
    pub level: DataClassificationLevel,
    /// Free-form governance/compliance tags attached by caller.
    pub tags: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Context describing a pending write authorization request.
pub struct WriteRequestContext {
    /// Domain receiving the write.
    pub domain: WriteDomain,
    /// Domain-local record identifier.
    pub record_id: String,
    /// Actor DID requesting the write.
    pub actor: String,
    /// Classification metadata for the write.
    pub tag: WriteTag,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Derived classification status for a given record/tag pairing.
pub struct ClassificationStatus {
    /// Domain evaluated for status.
    pub domain: WriteDomain,
    /// Domain-local record identifier.
    pub record_id: String,
    /// Policy minimum classification for the domain.
    pub minimum_level: DataClassificationLevel,
    /// Caller-provided classification level.
    pub provided_level: DataClassificationLevel,
    /// Missing required tags for the provided level.
    pub missing_tags: Vec<String>,
    /// True when policy checks authorize the write.
    pub authorized: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Engine that validates data classification constraints for writes.
pub struct DataClassificationEngine {
    policy: ClassificationPolicy,
}

impl DataClassificationEngine {
    /// Constructs a classification engine from validated policy.
    pub fn new(policy: ClassificationPolicy) -> Result<Self, DataClassificationError> {
        for domain in [
            WriteDomain::Messages,
            WriteDomain::Tasks,
            WriteDomain::Escrows,
            WriteDomain::Reputation,
        ] {
            if !policy.minimum_by_domain.contains_key(&domain) {
                return Err(DataClassificationError::InvalidPolicy(format!(
                    "missing minimum classification for domain {domain:?}"
                )));
            }
        }

        for tags in policy.required_tags_by_level.values() {
            for tag in tags {
                validate_tag(tag)?;
            }
        }

        Ok(Self { policy })
    }

    /// Authorizes a write request and returns canonical state key on success.
    pub fn authorize_write(
        &mut self,
        context: &WriteRequestContext,
    ) -> Result<String, DataClassificationError> {
        validate_non_empty("record_id", &context.record_id)?;
        validate_did(&context.actor)?;
        self.validate_write_tag(&context.record_id, &context.tag)?;

        let minimum = self.minimum_for(context.domain);
        if context.tag.level < minimum {
            return Err(DataClassificationError::ClassificationBelowDomainMinimum {
                domain: context.domain,
                required: minimum,
                provided: context.tag.level,
            });
        }

        let missing = self.missing_required_tags(&context.tag);
        if !missing.is_empty() {
            return Err(DataClassificationError::MissingRequiredTags {
                level: context.tag.level,
                missing,
            });
        }

        canonical_state_key(
            namespace_for_domain(context.domain),
            "record",
            &context.record_id,
        )
        .map_err(|error| DataClassificationError::InvalidPolicy(error.to_string()))
    }

    /// Computes authorization status for a domain/record and provided tag.
    pub fn status_for(
        &self,
        domain: WriteDomain,
        record_id: &str,
        tag: WriteTag,
    ) -> Result<ClassificationStatus, DataClassificationError> {
        validate_non_empty("record_id", record_id)?;
        self.validate_write_tag(record_id, &tag)?;

        let minimum = self.minimum_for(domain);
        let missing: Vec<String> = self.missing_required_tags(&tag).into_iter().collect();
        let authorized = tag.level >= minimum
            && !(tag.level >= DataClassificationLevel::Sensitive && tag.tags.is_empty())
            && missing.is_empty();

        Ok(ClassificationStatus {
            domain,
            record_id: record_id.to_owned(),
            minimum_level: minimum,
            provided_level: tag.level,
            missing_tags: missing,
            authorized,
        })
    }

    fn minimum_for(&self, domain: WriteDomain) -> DataClassificationLevel {
        self.policy
            .minimum_by_domain
            .get(&domain)
            .copied()
            .unwrap_or(DataClassificationLevel::Public)
    }

    fn validate_write_tag(
        &self,
        record_id: &str,
        tag: &WriteTag,
    ) -> Result<(), DataClassificationError> {
        for value in &tag.tags {
            validate_tag(value)?;
        }

        if tag.level >= DataClassificationLevel::Sensitive && tag.tags.is_empty() {
            return Err(DataClassificationError::UntaggedSensitiveWrite(
                record_id.to_owned(),
            ));
        }
        Ok(())
    }

    fn missing_required_tags(&self, tag: &WriteTag) -> BTreeSet<String> {
        match self.policy.required_tags_by_level.get(&tag.level) {
            Some(required) => required.difference(&tag.tags).cloned().collect(),
            None => BTreeSet::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Data classification engine error taxonomy.
pub enum DataClassificationError {
    /// Policy configuration failed validation.
    InvalidPolicy(String),
    /// Required field value was empty.
    EmptyField(&'static str),
    /// Tag value failed validation.
    InvalidTag(String),
    /// Actor DID failed parse/validation.
    InvalidDid(String),
    /// Caller omitted one or more required tags for classification level.
    MissingRequiredTags {
        /// Classification level with required tag policy.
        level: DataClassificationLevel,
        /// Tags missing from caller payload.
        missing: BTreeSet<String>,
    },
    /// Caller provided classification level below domain minimum.
    ClassificationBelowDomainMinimum {
        /// Domain enforcing the minimum.
        domain: WriteDomain,
        /// Required minimum level from policy.
        required: DataClassificationLevel,
        /// Caller-provided level.
        provided: DataClassificationLevel,
    },
    /// Sensitive/restricted write was submitted without tags.
    UntaggedSensitiveWrite(String),
}

impl fmt::Display for DataClassificationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPolicy(value) => write!(f, "invalid classification policy: {value}"),
            Self::EmptyField(field) => write!(f, "field must not be empty: {field}"),
            Self::InvalidTag(value) => write!(f, "invalid tag: {value}"),
            Self::InvalidDid(value) => write!(f, "invalid did: {value}"),
            Self::MissingRequiredTags { level, missing } => write!(
                f,
                "missing required tags for level {level:?}: {}",
                missing.iter().cloned().collect::<Vec<_>>().join(", ")
            ),
            Self::ClassificationBelowDomainMinimum {
                domain,
                required,
                provided,
            } => write!(
                f,
                "classification level {provided:?} is below minimum {required:?} for domain {domain:?}"
            ),
            Self::UntaggedSensitiveWrite(value) => {
                write!(f, "sensitive/restricted write is missing tags: {value}")
            }
        }
    }
}

impl std::error::Error for DataClassificationError {}

fn validate_non_empty(field: &'static str, value: &str) -> Result<(), DataClassificationError> {
    if value.trim().is_empty() {
        return Err(DataClassificationError::EmptyField(field));
    }
    Ok(())
}

fn validate_tag(value: &str) -> Result<(), DataClassificationError> {
    if value.trim().is_empty() {
        return Err(DataClassificationError::InvalidTag(value.to_owned()));
    }
    Ok(())
}

fn validate_did(value: &str) -> Result<(), DataClassificationError> {
    AgentDid::parse(value)
        .map_err(|error| DataClassificationError::InvalidDid(error.to_string()))?;
    Ok(())
}

fn namespace_for_domain(domain: WriteDomain) -> &'static str {
    match domain {
        WriteDomain::Messages => "kamn.messages",
        WriteDomain::Tasks => "kamn.tasks",
        WriteDomain::Escrows => "kamn.escrows",
        WriteDomain::Reputation => "kamn.reputation",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ClassificationPolicy, DataClassificationEngine, DataClassificationError,
        DataClassificationLevel, WriteDomain, WriteRequestContext, WriteTag,
    };
    use std::collections::BTreeMap;

    fn policy() -> ClassificationPolicy {
        let mut minimum_by_domain = BTreeMap::new();
        minimum_by_domain.insert(WriteDomain::Messages, DataClassificationLevel::Internal);
        minimum_by_domain.insert(WriteDomain::Tasks, DataClassificationLevel::Internal);
        minimum_by_domain.insert(WriteDomain::Escrows, DataClassificationLevel::Sensitive);
        minimum_by_domain.insert(WriteDomain::Reputation, DataClassificationLevel::Public);

        ClassificationPolicy {
            minimum_by_domain,
            required_tags_by_level: BTreeMap::new(),
        }
    }

    #[test]
    fn constructor_requires_domain_minimums() {
        assert_eq!(
            DataClassificationEngine::new(ClassificationPolicy {
                minimum_by_domain: BTreeMap::new(),
                required_tags_by_level: BTreeMap::new(),
            }),
            Err(DataClassificationError::InvalidPolicy(
                "missing minimum classification for domain Messages".to_owned()
            ))
        );
    }

    #[test]
    fn authorize_rejects_invalid_did() {
        let mut engine = DataClassificationEngine::new(policy()).expect("engine should construct");

        assert_eq!(
            engine.authorize_write(&WriteRequestContext {
                domain: WriteDomain::Messages,
                record_id: "msg-1".to_owned(),
                actor: "bad-did".to_owned(),
                tag: WriteTag {
                    level: DataClassificationLevel::Internal,
                    tags: Default::default(),
                },
            }),
            Err(DataClassificationError::InvalidDid(
                "invalid agent did prefix: bad-did".to_owned()
            ))
        );
    }

    #[test]
    fn status_marks_authorized_when_all_controls_pass() {
        let engine = DataClassificationEngine::new(policy()).expect("engine should construct");
        let status = engine
            .status_for(
                WriteDomain::Tasks,
                "task-1",
                WriteTag {
                    level: DataClassificationLevel::Internal,
                    tags: Default::default(),
                },
            )
            .expect("status should resolve");
        assert!(status.authorized);
        assert!(status.missing_tags.is_empty());
    }
}
