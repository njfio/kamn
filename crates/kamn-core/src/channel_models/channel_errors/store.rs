use super::*;

/// Errors emitted by snapshot-store read/write and recovery operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelSnapshotStoreError {
    /// Filesystem I/O operation failed.
    Io(String),
    /// Snapshot payload encoding/format was invalid.
    InvalidPayload(String),
    /// Snapshot payload failed semantic validation.
    Snapshot(ChannelSnapshotError),
}

impl fmt::Display for ChannelSnapshotStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(value) => write!(f, "channel snapshot store I/O error: {value}"),
            Self::InvalidPayload(value) => {
                write!(f, "channel snapshot store invalid payload: {value}")
            }
            Self::Snapshot(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for ChannelSnapshotStoreError {}
