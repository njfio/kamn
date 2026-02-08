use crate::{canonical_state_key, AgentDid};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DataClassificationLevel {
    Public,
    Internal,
    Sensitive,
    Restricted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WriteDomain {
    Messages,
    Tasks,
    Escrows,
    Reputation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassificationPolicy {
    pub minimum_by_domain: BTreeMap<WriteDomain, DataClassificationLevel>,
    pub required_tags_by_level: BTreeMap<DataClassificationLevel, BTreeSet<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteTag {
    pub level: DataClassificationLevel,
    pub tags: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteRequestContext {
    pub domain: WriteDomain,
    pub record_id: String,
    pub actor: String,
    pub tag: WriteTag,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassificationStatus {
    pub domain: WriteDomain,
    pub record_id: String,
    pub minimum_level: DataClassificationLevel,
    pub provided_level: DataClassificationLevel,
    pub missing_tags: Vec<String>,
    pub authorized: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataClassificationEngine {
    policy: ClassificationPolicy,
}

impl DataClassificationEngine {
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
pub enum DataClassificationError {
    InvalidPolicy(String),
    EmptyField(&'static str),
    InvalidTag(String),
    InvalidDid(String),
    MissingRequiredTags {
        level: DataClassificationLevel,
        missing: BTreeSet<String>,
    },
    ClassificationBelowDomainMinimum {
        domain: WriteDomain,
        required: DataClassificationLevel,
        provided: DataClassificationLevel,
    },
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
