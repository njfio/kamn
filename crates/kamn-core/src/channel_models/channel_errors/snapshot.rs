use super::*;

/// Errors emitted while validating/restoring snapshots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelSnapshotError {
    /// Snapshot schema version mismatched runtime expectation.
    SnapshotVersionMismatch {
        /// Expected schema version.
        expected: u16,
        /// Schema version found in snapshot payload.
        found: u16,
    },
    /// Duplicate channel identifier was found in snapshot records.
    DuplicateChannelId(String),
    /// Snapshot payload was malformed or semantically invalid.
    InvalidSnapshot(String),
    /// Snapshot record failed normal channel-model validation.
    Model(ChannelModelError),
}

impl fmt::Display for ChannelSnapshotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SnapshotVersionMismatch { expected, found } => {
                write!(
                    f,
                    "channel snapshot version mismatch: expected {expected}, found {found}"
                )
            }
            Self::DuplicateChannelId(value) => {
                write!(f, "duplicate channel id in snapshot: {value}")
            }
            Self::InvalidSnapshot(value) => write!(f, "invalid channel snapshot: {value}"),
            Self::Model(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for ChannelSnapshotError {}
