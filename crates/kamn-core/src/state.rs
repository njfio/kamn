//! Canonical state schema/versioning and deterministic state-key construction.

use std::fmt;

use crate::namespaces::StateNamespaces;

/// Monotonic schema version marker for persisted state layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct StateVersion(
    /// Integer representation of the schema version.
    pub u32,
);

/// Current application state schema version.
pub const APP_STATE_VERSION: StateVersion = StateVersion(1);
/// Maximum allowed length for each canonical state-key part.
pub const MAX_STATE_KEY_PART_LEN: usize = 96;

/// Canonical state schema metadata used by bootstrap and migration planning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppStateSchema {
    /// Schema version this node expects for persisted state.
    pub version: StateVersion,
    /// Canonical namespace registry used for state-key composition.
    pub namespaces: StateNamespaces,
}

impl Default for AppStateSchema {
    fn default() -> Self {
        Self {
            version: APP_STATE_VERSION,
            namespaces: StateNamespaces::default(),
        }
    }
}

/// Builds a deterministic canonical state key `<namespace>:<entity>:<id>`.
pub fn canonical_state_key(
    namespace: &str,
    entity: &str,
    id: &str,
) -> Result<String, StateKeyError> {
    validate_namespace(namespace)?;
    validate_key_part("entity", entity)?;
    validate_key_part("id", id)?;
    Ok(format!("{namespace}:{entity}:{id}"))
}

fn validate_namespace(namespace: &str) -> Result<(), StateKeyError> {
    if namespace.trim().is_empty() {
        return Err(StateKeyError::EmptyPart("namespace"));
    }
    if !namespace.starts_with("kamn.") {
        return Err(StateKeyError::NamespacePrefix(namespace.to_owned()));
    }
    if namespace.len() > MAX_STATE_KEY_PART_LEN {
        return Err(StateKeyError::PartTooLong {
            part: "namespace",
            max: MAX_STATE_KEY_PART_LEN,
            actual: namespace.len(),
        });
    }
    if !namespace
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '.' || ch == '_')
    {
        return Err(StateKeyError::InvalidCharacter {
            part: "namespace",
            value: namespace.to_owned(),
        });
    }
    Ok(())
}

fn validate_key_part(part: &'static str, value: &str) -> Result<(), StateKeyError> {
    if value.trim().is_empty() {
        return Err(StateKeyError::EmptyPart(part));
    }
    if value.len() > MAX_STATE_KEY_PART_LEN {
        return Err(StateKeyError::PartTooLong {
            part,
            max: MAX_STATE_KEY_PART_LEN,
            actual: value.len(),
        });
    }
    if !value
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_')
    {
        return Err(StateKeyError::InvalidCharacter {
            part,
            value: value.to_owned(),
        });
    }
    Ok(())
}

/// Validation error for canonical state-key construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateKeyError {
    /// Required key part is empty.
    EmptyPart(&'static str),
    /// One key part exceeds the configured max length.
    PartTooLong {
        /// Part label that violated the constraint.
        part: &'static str,
        /// Maximum supported character length.
        max: usize,
        /// Observed character length.
        actual: usize,
    },
    /// One key part contains unsupported characters.
    InvalidCharacter {
        /// Part label that violated character rules.
        part: &'static str,
        /// Raw value that failed validation.
        value: String,
    },
    /// Namespace does not use the required `kamn.` prefix.
    NamespacePrefix(String),
}

impl fmt::Display for StateKeyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPart(part) => write!(f, "{part} must not be empty"),
            Self::PartTooLong { part, max, actual } => {
                write!(f, "{part} exceeds max length {max}: {actual}")
            }
            Self::InvalidCharacter { part, value } => {
                write!(f, "invalid characters for {part}: {value}")
            }
            Self::NamespacePrefix(value) => {
                write!(f, "namespace must use kamn. prefix: {value}")
            }
        }
    }
}

impl std::error::Error for StateKeyError {}

#[cfg(test)]
mod tests {
    use super::{
        canonical_state_key, AppStateSchema, StateKeyError, StateVersion, APP_STATE_VERSION,
    };

    #[test]
    fn state_version_orders() {
        assert!(StateVersion(1) < StateVersion(2));
    }

    #[test]
    fn schema_default_matches_current_version() {
        let schema = AppStateSchema::default();
        assert_eq!(schema.version, APP_STATE_VERSION);
        assert!(schema.namespaces.all_unique());
    }

    #[test]
    fn canonical_key_is_deterministic() {
        let key = canonical_state_key("kamn.tasks.state", "assignment", "task_123")
            .expect("state key must be canonical");
        assert_eq!(key, "kamn.tasks.state:assignment:task_123");
    }

    #[test]
    fn canonical_key_rejects_invalid_namespace_prefix() {
        let key = canonical_state_key("tasks.state", "assignment", "task_123");
        assert_eq!(
            key,
            Err(StateKeyError::NamespacePrefix("tasks.state".to_owned()))
        );
    }

    #[test]
    fn canonical_key_rejects_invalid_characters() {
        let key = canonical_state_key("kamn.tasks.state", "Assignment", "task_123");
        assert_eq!(
            key,
            Err(StateKeyError::InvalidCharacter {
                part: "entity",
                value: "Assignment".to_owned()
            })
        );
    }

    #[test]
    fn canonical_key_rejects_empty_id() {
        // Regression: #17
        let key = canonical_state_key("kamn.tasks.state", "assignment", "");
        assert_eq!(key, Err(StateKeyError::EmptyPart("id")));
    }
}
