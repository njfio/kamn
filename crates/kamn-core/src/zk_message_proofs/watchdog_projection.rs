use super::validator_consensus::{ValidatorProofConsensusDecision, ValidatorProofConsensusStatus};

/// Watchdog incident class derived from validator proof consensus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProofWatchdogProjectionKind {
    ConsensusAligned,
    InvalidProofConsensus,
    ReplayProofConsensus,
    ValidatorMismatch,
}

/// Operational severity attached to the projected watchdog incident.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProofWatchdogSeverity {
    Info,
    Warning,
    Critical,
}

/// Deterministic watchdog projection emitted from proof consensus state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofWatchdogProjection {
    pub incident_fingerprint: String,
    pub message_id: String,
    pub artifact_id: String,
    pub kind: ProofWatchdogProjectionKind,
    pub severity: ProofWatchdogSeverity,
    pub required_quorum: usize,
    pub validator_count: usize,
    pub valid_attestation_count: usize,
    pub invalid_attestation_count: usize,
    pub replay_attestation_count: usize,
}

/// Stateless projector from consensus decision to watchdog incident.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ProofWatchdogProjector;

impl ProofWatchdogProjector {
    pub fn new() -> Self {
        Self
    }

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
