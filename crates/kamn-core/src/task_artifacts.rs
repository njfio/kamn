use crate::AgentDid;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// Persisted task artifact metadata after successful registration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskArtifactRecord {
    /// Stable artifact identifier.
    pub artifact_id: String,
    /// Task identifier the artifact belongs to.
    pub task_id: String,
    /// Creator DID for the artifact.
    pub creator: String,
    /// Creation timestamp in unix seconds.
    pub created_at_unix: u64,
    /// On-chain integrity hash for artifact verification.
    pub on_chain_hash: String,
    /// Off-chain URI where artifact content is stored.
    pub off_chain_uri: String,
    /// MIME-style content type for artifact payload.
    pub content_type: String,
}

/// Input payload used to register a new task artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskArtifactSubmission {
    /// Stable artifact identifier.
    pub artifact_id: String,
    /// Task identifier the artifact belongs to.
    pub task_id: String,
    /// Creator DID for the artifact.
    pub creator: String,
    /// Creation timestamp in unix seconds.
    pub created_at_unix: u64,
    /// On-chain integrity hash for artifact verification.
    pub on_chain_hash: String,
    /// Off-chain URI where artifact content is stored.
    pub off_chain_uri: String,
    /// MIME-style content type for artifact payload.
    pub content_type: String,
}

/// Registry for task-artifact storage keyed by artifact, task, and creator.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TaskArtifactRegistry {
    artifacts: BTreeMap<String, TaskArtifactRecord>,
    artifacts_by_task: BTreeMap<String, BTreeSet<String>>,
    artifacts_by_creator: BTreeMap<String, BTreeSet<String>>,
}

impl TaskArtifactRegistry {
    /// Creates an empty task artifact registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Computes deterministic integrity fingerprint for `(task_id, creator, off_chain_uri)`.
    pub fn integrity_fingerprint(task_id: &str, creator: &str, off_chain_uri: &str) -> String {
        let canonical = format!("{task_id}|{creator}|{off_chain_uri}");
        fnv1a_hex(&canonical)
    }

    /// Registers a new artifact after validating fields, DID, and integrity hash.
    pub fn register(
        &mut self,
        submission: TaskArtifactSubmission,
    ) -> Result<(), TaskArtifactError> {
        validate_non_empty("artifact_id", &submission.artifact_id)?;
        validate_non_empty("task_id", &submission.task_id)?;
        validate_did(&submission.creator)?;
        if submission.created_at_unix == 0 {
            return Err(TaskArtifactError::EmptyField("created_at_unix"));
        }
        validate_non_empty("on_chain_hash", &submission.on_chain_hash)?;
        validate_non_empty("off_chain_uri", &submission.off_chain_uri)?;
        validate_non_empty("content_type", &submission.content_type)?;

        if self.artifacts.contains_key(&submission.artifact_id) {
            return Err(TaskArtifactError::DuplicateArtifactId(
                submission.artifact_id.clone(),
            ));
        }

        let expected = Self::integrity_fingerprint(
            &submission.task_id,
            &submission.creator,
            &submission.off_chain_uri,
        );
        if submission.on_chain_hash != expected {
            return Err(TaskArtifactError::IntegrityMismatch {
                expected,
                provided: submission.on_chain_hash.clone(),
            });
        }

        let record = TaskArtifactRecord {
            artifact_id: submission.artifact_id.clone(),
            task_id: submission.task_id.clone(),
            creator: submission.creator.clone(),
            created_at_unix: submission.created_at_unix,
            on_chain_hash: submission.on_chain_hash.clone(),
            off_chain_uri: submission.off_chain_uri.clone(),
            content_type: submission.content_type.clone(),
        };

        self.artifacts
            .insert(submission.artifact_id.clone(), record);
        self.artifacts_by_task
            .entry(submission.task_id)
            .or_default()
            .insert(submission.artifact_id.clone());
        self.artifacts_by_creator
            .entry(submission.creator)
            .or_default()
            .insert(submission.artifact_id);
        Ok(())
    }

    /// Returns artifact record by identifier.
    pub fn artifact(&self, artifact_id: &str) -> Result<&TaskArtifactRecord, TaskArtifactError> {
        self.artifacts
            .get(artifact_id)
            .ok_or_else(|| TaskArtifactError::NotFound(artifact_id.to_owned()))
    }

    /// Lists artifact ids associated with the provided task id.
    pub fn artifacts_for_task(&self, task_id: &str) -> Vec<String> {
        self.artifacts_by_task
            .get(task_id)
            .map(|values| values.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Lists artifact ids associated with the provided creator DID.
    pub fn artifacts_for_creator(&self, creator: &str) -> Vec<String> {
        self.artifacts_by_creator
            .get(creator)
            .map(|values| values.iter().cloned().collect())
            .unwrap_or_default()
    }
}

/// Errors returned by task artifact registration and lookup flows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskArtifactError {
    /// Required field was empty.
    EmptyField(&'static str),
    /// Creator DID failed validation.
    InvalidDid(String),
    /// Artifact id already exists.
    DuplicateArtifactId(String),
    /// Integrity hash did not match expected fingerprint.
    IntegrityMismatch {
        /// Expected deterministic fingerprint.
        expected: String,
        /// Provided on-chain hash value.
        provided: String,
    },
    /// Artifact id does not exist.
    NotFound(String),
}

impl fmt::Display for TaskArtifactError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field) => write!(f, "field must not be empty: {field}"),
            Self::InvalidDid(value) => write!(f, "invalid did: {value}"),
            Self::DuplicateArtifactId(value) => write!(f, "duplicate artifact id: {value}"),
            Self::IntegrityMismatch { expected, provided } => {
                write!(
                    f,
                    "artifact integrity mismatch, expected {expected}, got {provided}"
                )
            }
            Self::NotFound(value) => write!(f, "artifact not found: {value}"),
        }
    }
}

impl std::error::Error for TaskArtifactError {}

fn validate_non_empty(field: &'static str, value: &str) -> Result<(), TaskArtifactError> {
    if value.trim().is_empty() {
        return Err(TaskArtifactError::EmptyField(field));
    }
    Ok(())
}

fn validate_did(value: &str) -> Result<(), TaskArtifactError> {
    AgentDid::parse(value).map_err(|error| TaskArtifactError::InvalidDid(error.to_string()))?;
    Ok(())
}

fn fnv1a_hex(value: &str) -> String {
    let mut acc: u64 = 0xcbf29ce484222325;
    for byte in value.bytes() {
        acc = acc.wrapping_mul(0x00000100000001B3);
        acc ^= u64::from(byte);
    }
    format!("{acc:016x}")
}

#[cfg(test)]
mod tests {
    use super::{TaskArtifactError, TaskArtifactRegistry, TaskArtifactSubmission};

    #[test]
    fn integrity_fingerprint_is_deterministic() {
        let first = TaskArtifactRegistry::integrity_fingerprint(
            "task-1",
            "kamn:did:agent:builder-1",
            "ipfs://artifact",
        );
        let second = TaskArtifactRegistry::integrity_fingerprint(
            "task-1",
            "kamn:did:agent:builder-1",
            "ipfs://artifact",
        );

        assert_eq!(first, second);
    }

    #[test]
    fn register_rejects_invalid_did() {
        let mut registry = TaskArtifactRegistry::new();
        let hash = TaskArtifactRegistry::integrity_fingerprint("task-1", "bad-did", "uri");
        assert_eq!(
            registry.register(TaskArtifactSubmission {
                artifact_id: "artifact-1".to_owned(),
                task_id: "task-1".to_owned(),
                creator: "bad-did".to_owned(),
                created_at_unix: 1,
                on_chain_hash: hash,
                off_chain_uri: "uri".to_owned(),
                content_type: "application/json".to_owned(),
            }),
            Err(TaskArtifactError::InvalidDid(
                "invalid agent did prefix: bad-did".to_owned()
            ))
        );
    }

    #[test]
    fn register_rejects_zero_timestamp() {
        let mut registry = TaskArtifactRegistry::new();
        let hash = TaskArtifactRegistry::integrity_fingerprint(
            "task-1",
            "kamn:did:agent:builder-1",
            "uri",
        );
        assert_eq!(
            registry.register(TaskArtifactSubmission {
                artifact_id: "artifact-1".to_owned(),
                task_id: "task-1".to_owned(),
                creator: "kamn:did:agent:builder-1".to_owned(),
                created_at_unix: 0,
                on_chain_hash: hash,
                off_chain_uri: "uri".to_owned(),
                content_type: "application/json".to_owned(),
            }),
            Err(TaskArtifactError::EmptyField("created_at_unix"))
        );
    }
}
