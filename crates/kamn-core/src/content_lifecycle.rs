use crate::{cid_from_content_uri, content_uri_for_cid, ContentStorageError};
use std::collections::BTreeMap;
use std::fmt;

const SHORT_LIVED_RETAIN_SECS: u64 = 3_600;
const SHORT_LIVED_TOMBSTONE_SECS: u64 = 3_600;
const STANDARD_RETAIN_SECS: u64 = 86_400;
const STANDARD_TOMBSTONE_SECS: u64 = 86_400;
const COMPLIANCE_RETAIN_SECS: u64 = 31_536_000;
const COMPLIANCE_TOMBSTONE_SECS: u64 = 31_536_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentRetentionClass {
    ShortLived,
    Standard,
    Compliance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContentRetentionProfile {
    pub retain_for_secs: u64,
    pub tombstone_for_secs: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentLifecycleStatus {
    Active,
    Expired,
    Tombstoned,
    Purged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentCleanupActionKind {
    Tombstone,
    Purge,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentCleanupAction {
    pub cid: String,
    pub action: ContentCleanupActionKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentLifecycleRecord {
    pub cid: String,
    pub retention_class: ContentRetentionClass,
    pub created_at_unix: u64,
    pub expires_at_unix: u64,
    pub tombstoned_at_unix: Option<u64>,
    pub purge_after_unix: Option<u64>,
    pub purged_at_unix: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ContentLifecycleManager {
    records: BTreeMap<String, ContentLifecycleRecord>,
}

impl ContentLifecycleManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn retention_profile(class: ContentRetentionClass) -> ContentRetentionProfile {
        match class {
            ContentRetentionClass::ShortLived => ContentRetentionProfile {
                retain_for_secs: SHORT_LIVED_RETAIN_SECS,
                tombstone_for_secs: SHORT_LIVED_TOMBSTONE_SECS,
            },
            ContentRetentionClass::Standard => ContentRetentionProfile {
                retain_for_secs: STANDARD_RETAIN_SECS,
                tombstone_for_secs: STANDARD_TOMBSTONE_SECS,
            },
            ContentRetentionClass::Compliance => ContentRetentionProfile {
                retain_for_secs: COMPLIANCE_RETAIN_SECS,
                tombstone_for_secs: COMPLIANCE_TOMBSTONE_SECS,
            },
        }
    }

    pub fn register(
        &mut self,
        cid: &str,
        retention_class: ContentRetentionClass,
        created_at_unix: u64,
    ) -> Result<ContentLifecycleRecord, ContentLifecycleError> {
        validate_cid(cid)?;
        if created_at_unix == 0 {
            return Err(ContentLifecycleError::EmptyField("created_at_unix"));
        }
        if self.records.contains_key(cid) {
            return Err(ContentLifecycleError::DuplicateContent(cid.to_owned()));
        }

        let profile = Self::retention_profile(retention_class);
        let record = ContentLifecycleRecord {
            cid: cid.to_owned(),
            retention_class,
            created_at_unix,
            expires_at_unix: created_at_unix.saturating_add(profile.retain_for_secs),
            tombstoned_at_unix: None,
            purge_after_unix: None,
            purged_at_unix: None,
        };
        self.records.insert(cid.to_owned(), record.clone());
        Ok(record)
    }

    pub fn apply_tombstone(
        &mut self,
        cid: &str,
        tombstoned_at_unix: u64,
    ) -> Result<ContentLifecycleRecord, ContentLifecycleError> {
        if tombstoned_at_unix == 0 {
            return Err(ContentLifecycleError::EmptyField("tombstoned_at_unix"));
        }
        let record = self
            .records
            .get_mut(cid)
            .ok_or_else(|| ContentLifecycleError::NotFound(cid.to_owned()))?;
        if record.purged_at_unix.is_some() {
            return Err(ContentLifecycleError::Purged(cid.to_owned()));
        }
        if record.tombstoned_at_unix.is_none() {
            let profile = Self::retention_profile(record.retention_class);
            record.tombstoned_at_unix = Some(tombstoned_at_unix);
            record.purge_after_unix =
                Some(tombstoned_at_unix.saturating_add(profile.tombstone_for_secs));
        }
        Ok(record.clone())
    }

    pub fn lifecycle_status(
        &self,
        cid: &str,
        now_unix: u64,
    ) -> Result<ContentLifecycleStatus, ContentLifecycleError> {
        let record = self
            .records
            .get(cid)
            .ok_or_else(|| ContentLifecycleError::NotFound(cid.to_owned()))?;
        if record.purged_at_unix.is_some() {
            return Ok(ContentLifecycleStatus::Purged);
        }
        if record.tombstoned_at_unix.is_some() {
            return Ok(ContentLifecycleStatus::Tombstoned);
        }
        if now_unix > record.expires_at_unix {
            return Ok(ContentLifecycleStatus::Expired);
        }
        Ok(ContentLifecycleStatus::Active)
    }

    pub fn cleanup_due(&self, now_unix: u64) -> Vec<ContentCleanupAction> {
        let mut actions = Vec::new();
        for (cid, record) in &self.records {
            if record.purged_at_unix.is_some() {
                continue;
            }
            if let Some(purge_after_unix) = record.purge_after_unix {
                if now_unix > purge_after_unix {
                    actions.push(ContentCleanupAction {
                        cid: cid.clone(),
                        action: ContentCleanupActionKind::Purge,
                    });
                }
                continue;
            }
            if now_unix > record.expires_at_unix {
                actions.push(ContentCleanupAction {
                    cid: cid.clone(),
                    action: ContentCleanupActionKind::Tombstone,
                });
            }
        }
        actions
    }

    pub fn execute_cleanup(
        &mut self,
        cid: &str,
        now_unix: u64,
    ) -> Result<ContentCleanupActionKind, ContentLifecycleError> {
        let record = self
            .records
            .get_mut(cid)
            .ok_or_else(|| ContentLifecycleError::NotFound(cid.to_owned()))?;
        if record.purged_at_unix.is_some() {
            return Err(ContentLifecycleError::Purged(cid.to_owned()));
        }
        if let Some(purge_after_unix) = record.purge_after_unix {
            if now_unix > purge_after_unix {
                record.purged_at_unix = Some(now_unix);
                return Ok(ContentCleanupActionKind::Purge);
            }
            return Err(ContentLifecycleError::NoCleanupDue(cid.to_owned()));
        }
        if now_unix > record.expires_at_unix {
            let profile = Self::retention_profile(record.retention_class);
            record.tombstoned_at_unix = Some(now_unix);
            record.purge_after_unix = Some(now_unix.saturating_add(profile.tombstone_for_secs));
            return Ok(ContentCleanupActionKind::Tombstone);
        }
        Err(ContentLifecycleError::NoCleanupDue(cid.to_owned()))
    }

    pub fn assert_accessible(&self, cid: &str, now_unix: u64) -> Result<(), ContentLifecycleError> {
        match self.lifecycle_status(cid, now_unix)? {
            ContentLifecycleStatus::Active => Ok(()),
            ContentLifecycleStatus::Expired => Err(ContentLifecycleError::Expired(cid.to_owned())),
            ContentLifecycleStatus::Tombstoned => {
                Err(ContentLifecycleError::Tombstoned(cid.to_owned()))
            }
            ContentLifecycleStatus::Purged => Err(ContentLifecycleError::Purged(cid.to_owned())),
        }
    }

    pub fn assert_uri_accessible(
        &self,
        content_uri: &str,
        now_unix: u64,
    ) -> Result<String, ContentLifecycleError> {
        let cid = cid_from_content_uri(content_uri)
            .map_err(|_| ContentLifecycleError::InvalidContentUri(content_uri.to_owned()))?;
        self.assert_accessible(&cid, now_unix)?;
        Ok(cid)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentLifecycleError {
    EmptyField(&'static str),
    InvalidCid(String),
    InvalidContentUri(String),
    DuplicateContent(String),
    NotFound(String),
    NoCleanupDue(String),
    Expired(String),
    Tombstoned(String),
    Purged(String),
}

impl fmt::Display for ContentLifecycleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field) => write!(f, "field must not be empty: {field}"),
            Self::InvalidCid(cid) => write!(f, "invalid cid: {cid}"),
            Self::InvalidContentUri(uri) => write!(f, "invalid content uri: {uri}"),
            Self::DuplicateContent(cid) => write!(f, "duplicate lifecycle registration: {cid}"),
            Self::NotFound(cid) => write!(f, "content lifecycle record not found: {cid}"),
            Self::NoCleanupDue(cid) => write!(f, "no cleanup action due for {cid}"),
            Self::Expired(cid) => write!(f, "content is expired: {cid}"),
            Self::Tombstoned(cid) => write!(f, "content is tombstoned: {cid}"),
            Self::Purged(cid) => write!(f, "content is purged: {cid}"),
        }
    }
}

impl std::error::Error for ContentLifecycleError {}

fn validate_cid(cid: &str) -> Result<(), ContentLifecycleError> {
    content_uri_for_cid(cid)
        .map(|_| ())
        .map_err(|error| match error {
            ContentStorageError::InvalidCid(_) => ContentLifecycleError::InvalidCid(cid.to_owned()),
            _ => ContentLifecycleError::InvalidCid(cid.to_owned()),
        })
}

#[cfg(test)]
mod tests {
    use super::{
        ContentLifecycleError, ContentLifecycleManager, ContentLifecycleStatus,
        ContentRetentionClass,
    };

    #[test]
    fn register_rejects_duplicate_cid() {
        let mut manager = ContentLifecycleManager::new();
        manager
            .register(
                "kamn:cid:v1:aaaaaaaaaaaaaaaa",
                ContentRetentionClass::ShortLived,
                1,
            )
            .expect("first registration should succeed");
        assert_eq!(
            manager.register(
                "kamn:cid:v1:aaaaaaaaaaaaaaaa",
                ContentRetentionClass::ShortLived,
                2,
            ),
            Err(ContentLifecycleError::DuplicateContent(
                "kamn:cid:v1:aaaaaaaaaaaaaaaa".to_owned()
            ))
        );
    }

    #[test]
    fn status_reports_expired_after_ttl_boundary() {
        let mut manager = ContentLifecycleManager::new();
        let record = manager
            .register(
                "kamn:cid:v1:bbbbbbbbbbbbbbbb",
                ContentRetentionClass::ShortLived,
                10,
            )
            .expect("register should succeed");
        assert_eq!(
            manager
                .lifecycle_status("kamn:cid:v1:bbbbbbbbbbbbbbbb", record.expires_at_unix + 1)
                .expect("status should resolve"),
            ContentLifecycleStatus::Expired
        );
    }
}
