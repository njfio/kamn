use super::validator_consensus::{ValidatorProofConsensusDecision, ValidatorProofConsensusStatus};

/// Watchdog incident class derived from validator proof consensus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProofWatchdogProjectionKind {
    /// Consensus aligned variant for this public contract enum.
    ConsensusAligned,
    /// Invalid proof consensus variant for this public contract enum.
    InvalidProofConsensus,
    /// Replay proof consensus variant for this public contract enum.
    ReplayProofConsensus,
    /// Validator mismatch variant for this public contract enum.
    ValidatorMismatch,
}

/// Operational severity attached to the projected watchdog incident.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProofWatchdogSeverity {
    /// Info variant for this public contract enum.
    Info,
    /// Warning variant for this public contract enum.
    Warning,
    /// Critical variant for this public contract enum.
    Critical,
}

/// Deterministic watchdog projection emitted from proof consensus state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofWatchdogProjection {
    /// Incident fingerprint carried by this public contract model.
    pub incident_fingerprint: String,
    /// Message id carried by this public contract model.
    pub message_id: String,
    /// Artifact id carried by this public contract model.
    pub artifact_id: String,
    /// Kind carried by this public contract model.
    pub kind: ProofWatchdogProjectionKind,
    /// Severity carried by this public contract model.
    pub severity: ProofWatchdogSeverity,
    /// Required quorum carried by this public contract model.
    pub required_quorum: usize,
    /// Validator count carried by this public contract model.
    pub validator_count: usize,
    /// Valid attestation count carried by this public contract model.
    pub valid_attestation_count: usize,
    /// Invalid attestation count carried by this public contract model.
    pub invalid_attestation_count: usize,
    /// Replay attestation count carried by this public contract model.
    pub replay_attestation_count: usize,
}

/// Stateless projector from consensus decision to watchdog incident.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ProofWatchdogProjector;

impl ProofWatchdogProjector {
    /// Creates a new value for this public contract type.
    pub fn new() -> Self {
        Self
    }

    /// Runs the project contract operation.
    pub fn project(&self, decision: &ValidatorProofConsensusDecision) -> ProofWatchdogProjection {
        let (kind, severity) = projection_classification(decision.status);
        ProofWatchdogProjection {
            incident_fingerprint: incident_fingerprint(decision, kind),
            message_id: decision.message_id.clone(),
            artifact_id: decision.artifact_id.clone(),
            kind,
            severity,
            required_quorum: decision.required_quorum,
            validator_count: decision.validator_count,
            valid_attestation_count: decision.valid_attestation_count,
            invalid_attestation_count: decision.invalid_attestation_count,
            replay_attestation_count: decision.replay_attestation_count,
        }
    }
}

fn projection_classification(
    status: ValidatorProofConsensusStatus,
) -> (ProofWatchdogProjectionKind, ProofWatchdogSeverity) {
    match status {
        ValidatorProofConsensusStatus::ConsensusValid => (
            ProofWatchdogProjectionKind::ConsensusAligned,
            ProofWatchdogSeverity::Info,
        ),
        ValidatorProofConsensusStatus::ConsensusInvalid => (
            ProofWatchdogProjectionKind::InvalidProofConsensus,
            ProofWatchdogSeverity::Critical,
        ),
        ValidatorProofConsensusStatus::ConsensusReplay => (
            ProofWatchdogProjectionKind::ReplayProofConsensus,
            ProofWatchdogSeverity::Critical,
        ),
        ValidatorProofConsensusStatus::ValidatorMismatch => (
            ProofWatchdogProjectionKind::ValidatorMismatch,
            ProofWatchdogSeverity::Critical,
        ),
    }
}

fn incident_fingerprint(
    decision: &ValidatorProofConsensusDecision,
    kind: ProofWatchdogProjectionKind,
) -> String {
    format!(
        "proof-consensus:{}:{}:{}:{}:{}:{}",
        decision.message_id,
        decision.artifact_id,
        proof_watchdog_kind_code(kind),
        decision.valid_attestation_count,
        decision.invalid_attestation_count,
        decision.replay_attestation_count,
    )
}

fn proof_watchdog_kind_code(kind: ProofWatchdogProjectionKind) -> &'static str {
    match kind {
        ProofWatchdogProjectionKind::ConsensusAligned => "consensus-aligned",
        ProofWatchdogProjectionKind::InvalidProofConsensus => "invalid-proof-consensus",
        ProofWatchdogProjectionKind::ReplayProofConsensus => "replay-proof-consensus",
        ProofWatchdogProjectionKind::ValidatorMismatch => "validator-mismatch",
    }
}
