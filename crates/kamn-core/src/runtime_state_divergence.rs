use super::is_valid_kamn_did;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// State divergence status.
pub enum StateDivergenceStatus {
    /// In sync.
    InSync,
    /// Diverged.
    Diverged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// State divergence severity.
pub enum StateDivergenceSeverity {
    /// Info.
    Info,
    /// Critical.
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// State divergence evidence.
pub struct StateDivergenceEvidence {
    /// Peer id.
    pub peer_id: String,
    /// Expected state version.
    pub expected_state_version: u64,
    /// Observed state version.
    pub observed_state_version: u64,
    /// Expected state hash.
    pub expected_state_hash: String,
    /// Observed state hash.
    pub observed_state_hash: String,
    /// Observed at tick.
    pub observed_at_tick: u64,
}

impl StateDivergenceEvidence {
    /// Handles new.
    pub fn new(
        peer_id: &str,
        expected_state_version: u64,
        observed_state_version: u64,
        expected_state_hash: &str,
        observed_state_hash: &str,
        observed_at_tick: u64,
    ) -> Result<Self, StateDivergenceError> {
        if !is_valid_kamn_did(peer_id) {
            return Err(StateDivergenceError::InvalidPeerDid);
        }
        if expected_state_version == 0 {
            return Err(StateDivergenceError::InvalidStateVersion {
                field: "expected_state_version",
                value: expected_state_version,
            });
        }
        if observed_state_version == 0 {
            return Err(StateDivergenceError::InvalidStateVersion {
                field: "observed_state_version",
                value: observed_state_version,
            });
        }
        if expected_state_hash.trim().is_empty() {
            return Err(StateDivergenceError::IncompleteEvidenceField {
                field: "expected_state_hash",
            });
        }
        if observed_state_hash.trim().is_empty() {
            return Err(StateDivergenceError::IncompleteEvidenceField {
                field: "observed_state_hash",
            });
        }
        if observed_at_tick == 0 {
            return Err(StateDivergenceError::InvalidObservedTick {
                tick: observed_at_tick,
            });
        }

        Ok(Self {
            peer_id: peer_id.to_owned(),
            expected_state_version,
            observed_state_version,
            expected_state_hash: expected_state_hash.to_owned(),
            observed_state_hash: observed_state_hash.to_owned(),
            observed_at_tick,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// State divergence watch input.
pub struct StateDivergenceWatchInput {
    evidence: StateDivergenceEvidence,
}

impl StateDivergenceWatchInput {
    /// Handles new.
    pub fn new(
        peer_id: &str,
        expected_state_version: u64,
        observed_state_version: u64,
        expected_state_hash: &str,
        observed_state_hash: &str,
        observed_at_tick: u64,
    ) -> Result<Self, StateDivergenceError> {
        let evidence = StateDivergenceEvidence::new(
            peer_id,
            expected_state_version,
            observed_state_version,
            expected_state_hash,
            observed_state_hash,
            observed_at_tick,
        )?;
        Ok(Self { evidence })
    }

    /// Handles evidence.
    pub fn evidence(&self) -> &StateDivergenceEvidence {
        &self.evidence
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// State divergence report.
pub struct StateDivergenceReport {
    /// Status.
    pub status: StateDivergenceStatus,
    /// Severity.
    pub severity: StateDivergenceSeverity,
    /// Incident fingerprint.
    pub incident_fingerprint: String,
    /// Evidence.
    pub evidence: StateDivergenceEvidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// State divergence error.
pub enum StateDivergenceError {
    /// Invalid peer did.
    InvalidPeerDid,
    /// Invalid state version.
    InvalidStateVersion {
        /// Field name associated with the invalid value.
        field: &'static str,
        /// Provided state version value.
        value: u64,
    },
    /// Incomplete evidence field.
    IncompleteEvidenceField {
        /// Missing or empty evidence field name.
        field: &'static str,
    },
    /// Invalid observed tick.
    InvalidObservedTick {
        /// Observed daemon tick value.
        tick: u64,
    },
}

impl Display for StateDivergenceError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPeerDid => write!(f, "state divergence peer did is invalid"),
            Self::InvalidStateVersion { field, value } => {
                write!(
                    f,
                    "state divergence {field} must be positive, found {value}"
                )
            }
            Self::IncompleteEvidenceField { field } => {
                write!(
                    f,
                    "state divergence evidence field cannot be empty: {field}"
                )
            }
            Self::InvalidObservedTick { tick } => {
                write!(
                    f,
                    "state divergence observed tick must be positive, found {tick}"
                )
            }
        }
    }
}

impl Error for StateDivergenceError {}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// State divergence evaluator.
pub struct StateDivergenceEvaluator;

impl StateDivergenceEvaluator {
    /// Handles evaluate.
    pub fn evaluate(
        &self,
        input: StateDivergenceWatchInput,
    ) -> Result<StateDivergenceReport, StateDivergenceError> {
        let evidence = input.evidence;
        let diverged = evidence.expected_state_version != evidence.observed_state_version
            || evidence.expected_state_hash != evidence.observed_state_hash;

        let status = if diverged {
            StateDivergenceStatus::Diverged
        } else {
            StateDivergenceStatus::InSync
        };
        let severity = if diverged {
            StateDivergenceSeverity::Critical
        } else {
            StateDivergenceSeverity::Info
        };
        let incident_fingerprint = format!(
            "state-divergence:{}:{}:{}:{}:{}",
            evidence.peer_id,
            evidence.expected_state_version,
            evidence.observed_state_version,
            evidence.expected_state_hash,
            evidence.observed_state_hash
        );

        Ok(StateDivergenceReport {
            status,
            severity,
            incident_fingerprint,
            evidence,
        })
    }
}

/// Handles evaluate daemon state divergence.
pub fn evaluate_daemon_state_divergence(
    evaluator: &StateDivergenceEvaluator,
    input: StateDivergenceWatchInput,
) -> Result<StateDivergenceReport, StateDivergenceError> {
    evaluator.evaluate(input)
}
