//! Zero-knowledge message-proof planning, admission, consensus, and watchdog projection contracts.

use crate::{AgentDid, CanonicalMessageEnvelope, MessageEnvelopeError};
use std::collections::BTreeSet;
use std::fmt;

/// Supported proof-system families considered by KAMN evaluation flows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZkProofSystem {
    /// Pairing-based Groth16 proofs.
    Groth16,
    /// Plonk-like proof systems with universal setup variants.
    Plonkish,
    /// Transparent STARK-style proofs.
    Stark,
}

/// Verification-topology choices for proof checking responsibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZkVerificationTopology {
    /// Processor performs verification inline before publication.
    ProcessorOnly,
    /// Validators re-verify across quorum path.
    ValidatorQuorum,
    /// Watchdog nodes sample and project proof-health alerts.
    WatchdogSampling,
}

/// Severity classes for architectural and runtime proof risks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ZkRiskSeverity {
    /// Informational or low-impact risk.
    Low,
    /// Material but manageable risk.
    Medium,
    /// High-impact risk requiring mitigation before adoption.
    High,
}

/// Structured risk entry emitted by option evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZkRisk {
    /// Stable risk code identifier.
    pub code: String,
    /// Severity classification.
    pub severity: ZkRiskSeverity,
    /// Human-readable risk details.
    pub detail: String,
}

/// Candidate architecture option for proof-system adoption planning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZkArchitectureOption {
    /// Option identifier.
    pub name: String,
    /// Proof-system family.
    pub proof_system: ZkProofSystem,
    /// Verification-topology model.
    pub verification_topology: ZkVerificationTopology,
    /// Whether trusted setup ceremony is required.
    pub trusted_setup_required: bool,
    /// Whether witness-generation inputs are deterministic.
    pub deterministic_witness_inputs: bool,
    /// Estimated prover latency in milliseconds.
    pub prover_latency_ms: u64,
    /// Estimated verifier latency in milliseconds.
    pub verifier_latency_ms: u64,
    /// Estimated proof size in bytes.
    pub proof_size_bytes: u64,
    /// Whether option supports proof batching.
    pub supports_batching: bool,
    /// Estimated engineering effort in weeks.
    pub estimated_engineering_weeks: u16,
}

/// Policy thresholds for option feasibility scoring.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZkEvaluationPolicy {
    /// Maximum allowed verifier latency in milliseconds.
    pub max_verifier_latency_ms: u64,
    /// Maximum allowed proof size in bytes.
    pub max_proof_size_bytes: u64,
    /// Maximum allowed engineering effort in weeks.
    pub max_engineering_weeks: u16,
    /// Whether transparent setup is mandatory.
    pub require_transparent_setup: bool,
}

impl Default for ZkEvaluationPolicy {
    fn default() -> Self {
        Self {
            max_verifier_latency_ms: 25,
            max_proof_size_bytes: 2_048,
            max_engineering_weeks: 12,
            require_transparent_setup: true,
        }
    }
}

/// Evaluation result for a single architecture option under policy constraints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZkOptionAssessment {
    /// Evaluated option name.
    pub option_name: String,
    /// Aggregate score (higher is better).
    pub score: i32,
    /// Whether option satisfies policy feasibility constraints.
    pub feasible: bool,
    /// Enumerated trust assumptions for this option.
    pub trust_assumptions: Vec<String>,
    /// Enumerated risks for this option.
    pub risks: Vec<ZkRisk>,
}

/// Phase milestone for staged proof-adoption execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZkPhaseMilestone {
    /// Phase identifier.
    pub phase: String,
    /// Phase objective.
    pub objective: String,
    /// Validation emphasis for this phase.
    pub validation_focus: String,
    /// Exit criteria that must be satisfied.
    pub exit_criteria: Vec<String>,
}

/// Recommended phased plan and ranked assessments for proof adoption.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZkPhasePlan {
    /// Recommended option identifier.
    pub recommended_option: String,
    /// Human-readable recommendation rationale.
    pub rationale: String,
    /// Ordered implementation milestones.
    pub milestones: Vec<ZkPhaseMilestone>,
    /// Full ranked option assessments.
    pub assessments: Vec<ZkOptionAssessment>,
}

/// Witness projection produced from canonical message envelope and privacy selectors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZkMessageWitness {
    /// Public commitment derived from redacted payload shape.
    pub public_commitment: String,
    /// Field names revealed in the witness output.
    pub revealed_fields: Vec<String>,
    /// Number of hidden/private fields.
    pub hidden_field_count: usize,
    /// Canonical payload byte size.
    pub payload_bytes: usize,
}

/// Proof artifact emitted by processor-level proof generation pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessorProofArtifact {
    /// Unique proof artifact identifier.
    pub artifact_id: String,
    /// Message identifier this artifact attests.
    pub message_id: String,
    /// Payload commitment associated with the proof.
    pub payload_commitment: String,
    /// Serialized proof value.
    pub proof_value: String,
}

impl ProcessorProofArtifact {
    /// Construct and validate a processor proof artifact contract.
    pub fn new(
        artifact_id: &str,
        message_id: &str,
        payload_commitment: &str,
        proof_value: &str,
    ) -> Result<Self, ZkDesignError> {
        require_non_empty_artifact_field("artifact_id", artifact_id)?;
        require_non_empty_artifact_field("message_id", message_id)?;
        require_non_empty_artifact_field("payload_commitment", payload_commitment)?;
        require_non_empty_artifact_field("proof_value", proof_value)?;
        if !payload_commitment.starts_with("fnv1a64:") {
            return Err(ZkDesignError::InvalidProofArtifact(
                "payload_commitment must start with `fnv1a64:`".to_owned(),
            ));
        }
        if payload_commitment == "fnv1a64:" {
            return Err(ZkDesignError::InvalidProofArtifact(
                "payload_commitment must include digest bytes".to_owned(),
            ));
        }
        if !proof_value.starts_with("proof:") {
            return Err(ZkDesignError::InvalidProofArtifact(
                "proof_value must start with `proof:`".to_owned(),
            ));
        }
        Ok(Self {
            artifact_id: artifact_id.to_owned(),
            message_id: message_id.to_owned(),
            payload_commitment: payload_commitment.to_owned(),
            proof_value: proof_value.to_owned(),
        })
    }
}

/// Input payload for processor proof-admission evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessorProofAdmissionInput {
    /// Message identifier expected by admission flow.
    pub message_id: String,
    /// Expected payload commitment for this message.
    pub expected_payload_commitment: String,
    /// Proof artifact presented for admission.
    pub artifact: ProcessorProofArtifact,
}

impl ProcessorProofAdmissionInput {
    /// Construct proof-admission input after required-field validation.
    pub fn new(
        message_id: &str,
        expected_payload_commitment: &str,
        artifact: ProcessorProofArtifact,
    ) -> Result<Self, ZkDesignError> {
        require_non_empty_artifact_field("message_id", message_id)?;
        require_non_empty_artifact_field(
            "expected_payload_commitment",
            expected_payload_commitment,
        )?;
        Ok(Self {
            message_id: message_id.to_owned(),
            expected_payload_commitment: expected_payload_commitment.to_owned(),
            artifact,
        })
    }
}

/// Decision emitted when processor proof artifact is admitted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessorProofAdmissionDecision {
    /// Message identifier associated with decision.
    pub message_id: String,
    /// Artifact identifier accepted by evaluator.
    pub artifact_id: String,
    /// Payload commitment bound to decision.
    pub payload_commitment: String,
}

/// Stateful evaluator for processor proof-admission replay protection.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProcessorProofAdmissionEvaluator {
    accepted_artifact_ids: BTreeSet<String>,
}

impl ProcessorProofAdmissionEvaluator {
    /// Construct an empty proof-admission evaluator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate proof artifact identity/commitment and enforce replay protection.
    pub fn evaluate(
        &mut self,
        input: ProcessorProofAdmissionInput,
    ) -> Result<ProcessorProofAdmissionDecision, ZkDesignError> {
        if input.artifact.message_id != input.message_id {
            return Err(ZkDesignError::ProofArtifactMessageMismatch {
                expected: input.message_id,
                found: input.artifact.message_id,
            });
        }

        if input.artifact.payload_commitment != input.expected_payload_commitment {
            return Err(ZkDesignError::ProofArtifactCommitmentMismatch {
                expected: input.expected_payload_commitment,
                found: input.artifact.payload_commitment,
            });
        }

        let expected_proof_value = format!("proof:ok:{}", input.artifact.artifact_id);
        if input.artifact.proof_value != expected_proof_value {
            return Err(ZkDesignError::ProofVerificationFailed {
                artifact_id: input.artifact.artifact_id,
                reason: "proof value failed deterministic verification".to_owned(),
            });
        }

        if !self
            .accepted_artifact_ids
            .insert(input.artifact.artifact_id.clone())
        {
            return Err(ZkDesignError::ProofArtifactReplay(
                input.artifact.artifact_id,
            ));
        }

        Ok(ProcessorProofAdmissionDecision {
            message_id: input.message_id,
            artifact_id: input.artifact.artifact_id,
            payload_commitment: input.expected_payload_commitment,
        })
    }
}

/// Validator attestation verdict classes for consensus evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValidatorProofVerdict {
    /// Validator verified proof as valid.
    Valid,
    /// Validator verified proof as invalid.
    Invalid,
    /// Validator detected proof/attestation replay behavior.
    Replay,
}

/// Single validator attestation over proof artifact and message identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatorProofAttestation {
    /// Unique attestation identifier.
    pub attestation_id: String,
    /// DID of attesting validator.
    pub validator_did: String,
    /// Message identifier under attestation.
    pub message_id: String,
    /// Proof artifact identifier under attestation.
    pub artifact_id: String,
    /// Validator verdict over artifact.
    pub verdict: ValidatorProofVerdict,
}

impl ValidatorProofAttestation {
    /// Construct and validate validator proof attestation payload.
    pub fn new(
        attestation_id: &str,
        validator_did: &str,
        message_id: &str,
        artifact_id: &str,
        verdict: ValidatorProofVerdict,
    ) -> Result<Self, ValidatorProofConsensusError> {
        require_non_empty_consensus_field("attestation_id", attestation_id)?;
        require_non_empty_consensus_field("validator_did", validator_did)?;
        require_non_empty_consensus_field("message_id", message_id)?;
        require_non_empty_consensus_field("artifact_id", artifact_id)?;
        AgentDid::parse(validator_did).map_err(|error| {
            ValidatorProofConsensusError::InvalidValidatorDid(error.to_string())
        })?;
        Ok(Self {
            attestation_id: attestation_id.to_owned(),
            validator_did: validator_did.to_owned(),
            message_id: message_id.to_owned(),
            artifact_id: artifact_id.to_owned(),
            verdict,
        })
    }
}

/// Consensus-evaluation input payload over validator proof attestations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatorProofConsensusInput {
    /// Message identifier expected across attestations.
    pub message_id: String,
    /// Artifact identifier expected across attestations.
    pub artifact_id: String,
    /// Collected validator attestations for quorum evaluation.
    pub attestations: Vec<ValidatorProofAttestation>,
}

impl ValidatorProofConsensusInput {
    /// Construct consensus input after required-field and non-empty-attestation checks.
    pub fn new(
        message_id: &str,
        artifact_id: &str,
        attestations: Vec<ValidatorProofAttestation>,
    ) -> Result<Self, ValidatorProofConsensusError> {
        require_non_empty_consensus_field("message_id", message_id)?;
        require_non_empty_consensus_field("artifact_id", artifact_id)?;
        if attestations.is_empty() {
            return Err(ValidatorProofConsensusError::EmptyAttestations);
        }
        Ok(Self {
            message_id: message_id.to_owned(),
            artifact_id: artifact_id.to_owned(),
            attestations,
        })
    }
}

/// Terminal consensus statuses for validator proof evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidatorProofConsensusStatus {
    /// Quorum converged on valid verdict.
    ConsensusValid,
    /// Quorum converged on invalid verdict.
    ConsensusInvalid,
    /// Quorum converged on replay verdict.
    ConsensusReplay,
    /// Mixed verdict buckets indicate validator mismatch.
    ValidatorMismatch,
}

/// Consensus decision projection emitted by validator evaluator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatorProofConsensusDecision {
    /// Message identifier evaluated.
    pub message_id: String,
    /// Artifact identifier evaluated.
    pub artifact_id: String,
    /// Required quorum configured for evaluator.
    pub required_quorum: usize,
    /// Number of distinct validators observed.
    pub validator_count: usize,
    /// Sorted validator DID list.
    pub validator_dids: Vec<String>,
    /// Count of `Valid` verdict attestations.
    pub valid_attestation_count: usize,
    /// Count of `Invalid` verdict attestations.
    pub invalid_attestation_count: usize,
    /// Count of `Replay` verdict attestations.
    pub replay_attestation_count: usize,
    /// Derived consensus status.
    pub status: ValidatorProofConsensusStatus,
}

/// Errors emitted by validator proof consensus intake/evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidatorProofConsensusError {
    /// Required quorum must be positive.
    InvalidRequiredQuorum(usize),
    /// Required input field was empty.
    InvalidField {
        /// Input field name that failed validation.
        field: &'static str,
    },
    /// No attestations were provided.
    EmptyAttestations,
    /// Validator DID failed parse/validation.
    InvalidValidatorDid(String),
    /// Attestation message id did not match input message id.
    AttestationMessageMismatch {
        /// Expected message identifier.
        expected: String,
        /// Found message identifier.
        found: String,
    },
    /// Attestation artifact id did not match input artifact id.
    AttestationArtifactMismatch {
        /// Expected artifact identifier.
        expected: String,
        /// Found artifact identifier.
        found: String,
    },
    /// Duplicate validator attestation detected.
    DuplicateValidator(String),
    /// Duplicate attestation id detected in input batch.
    DuplicateAttestationId(String),
    /// Attestation id was already consumed in prior evaluation.
    AttestationReplay(String),
    /// Input count is below required quorum.
    InsufficientAttestations {
        /// Required quorum count.
        required: usize,
        /// Received attestation count.
        received: usize,
    },
}

impl fmt::Display for ValidatorProofConsensusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequiredQuorum(required_quorum) => {
                write!(
                    f,
                    "validator proof required quorum must be greater than zero, found {required_quorum}"
                )
            }
            Self::InvalidField { field } => write!(f, "{field} must not be empty"),
            Self::EmptyAttestations => {
                write!(f, "validator proof attestations must not be empty")
            }
            Self::InvalidValidatorDid(value) => {
                write!(f, "validator proof attestation DID is invalid: {value}")
            }
            Self::AttestationMessageMismatch { expected, found } => write!(
                f,
                "validator attestation message mismatch: expected {expected}, found {found}"
            ),
            Self::AttestationArtifactMismatch { expected, found } => write!(
                f,
                "validator attestation artifact mismatch: expected {expected}, found {found}"
            ),
            Self::DuplicateValidator(validator_did) => {
                write!(
                    f,
                    "validator proof attestation duplicate validator: {validator_did}"
                )
            }
            Self::DuplicateAttestationId(attestation_id) => {
                write!(
                    f,
                    "validator proof attestation id duplicated in input: {attestation_id}"
                )
            }
            Self::AttestationReplay(attestation_id) => {
                write!(
                    f,
                    "validator proof attestation replay detected: {attestation_id}"
                )
            }
            Self::InsufficientAttestations { required, received } => write!(
                f,
                "validator proof quorum insufficient attestations: required {required}, received {received}"
            ),
        }
    }
}

impl std::error::Error for ValidatorProofConsensusError {}

/// Stateful evaluator for validator proof-consensus decisions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatorProofConsensusEvaluator {
    required_quorum: usize,
    consumed_attestation_ids: BTreeSet<String>,
}

impl ValidatorProofConsensusEvaluator {
    /// Construct consensus evaluator with required quorum.
    pub fn new(required_quorum: usize) -> Result<Self, ValidatorProofConsensusError> {
        if required_quorum == 0 {
            return Err(ValidatorProofConsensusError::InvalidRequiredQuorum(
                required_quorum,
            ));
        }
        Ok(Self {
            required_quorum,
            consumed_attestation_ids: BTreeSet::new(),
        })
    }

    /// Return required quorum configured for evaluator.
    pub fn required_quorum(&self) -> usize {
        self.required_quorum
    }

    /// Evaluate validator attestations into deterministic consensus decision.
    pub fn evaluate(
        &mut self,
        input: ValidatorProofConsensusInput,
    ) -> Result<ValidatorProofConsensusDecision, ValidatorProofConsensusError> {
        let received = input.attestations.len();
        if received < self.required_quorum {
            return Err(ValidatorProofConsensusError::InsufficientAttestations {
                required: self.required_quorum,
                received,
            });
        }

        let mut validator_dids = BTreeSet::new();
        let mut local_attestation_ids = BTreeSet::new();
        let mut valid_attestation_count = 0usize;
        let mut invalid_attestation_count = 0usize;
        let mut replay_attestation_count = 0usize;

        for attestation in &input.attestations {
            if attestation.message_id != input.message_id {
                return Err(ValidatorProofConsensusError::AttestationMessageMismatch {
                    expected: input.message_id.clone(),
                    found: attestation.message_id.clone(),
                });
            }
            if attestation.artifact_id != input.artifact_id {
                return Err(ValidatorProofConsensusError::AttestationArtifactMismatch {
                    expected: input.artifact_id.clone(),
                    found: attestation.artifact_id.clone(),
                });
            }
            if !validator_dids.insert(attestation.validator_did.clone()) {
                return Err(ValidatorProofConsensusError::DuplicateValidator(
                    attestation.validator_did.clone(),
                ));
            }
            if self
                .consumed_attestation_ids
                .contains(&attestation.attestation_id)
            {
                return Err(ValidatorProofConsensusError::AttestationReplay(
                    attestation.attestation_id.clone(),
                ));
            }
            if !local_attestation_ids.insert(attestation.attestation_id.clone()) {
                return Err(ValidatorProofConsensusError::DuplicateAttestationId(
                    attestation.attestation_id.clone(),
                ));
            }

            match attestation.verdict {
                ValidatorProofVerdict::Valid => valid_attestation_count += 1,
                ValidatorProofVerdict::Invalid => invalid_attestation_count += 1,
                ValidatorProofVerdict::Replay => replay_attestation_count += 1,
            }
        }

        for attestation_id in local_attestation_ids {
            self.consumed_attestation_ids.insert(attestation_id);
        }

        let verdict_bucket_count = [
            valid_attestation_count,
            invalid_attestation_count,
            replay_attestation_count,
        ]
        .into_iter()
        .filter(|count| *count > 0)
        .count();

        let status = if verdict_bucket_count > 1 {
            ValidatorProofConsensusStatus::ValidatorMismatch
        } else if valid_attestation_count > 0 {
            ValidatorProofConsensusStatus::ConsensusValid
        } else if invalid_attestation_count > 0 {
            ValidatorProofConsensusStatus::ConsensusInvalid
        } else {
            ValidatorProofConsensusStatus::ConsensusReplay
        };

        Ok(ValidatorProofConsensusDecision {
            message_id: input.message_id,
            artifact_id: input.artifact_id,
            required_quorum: self.required_quorum,
            validator_count: validator_dids.len(),
            validator_dids: validator_dids.into_iter().collect(),
            valid_attestation_count,
            invalid_attestation_count,
            replay_attestation_count,
            status,
        })
    }
}

/// Watchdog projection kinds derived from proof-consensus status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProofWatchdogProjectionKind {
    /// Consensus aligned on valid proof.
    ConsensusAligned,
    /// Consensus aligned on invalid proof.
    InvalidProofConsensus,
    /// Consensus aligned on replay classification.
    ReplayProofConsensus,
    /// Validators disagreed across verdict classes.
    ValidatorMismatch,
}

/// Severity classes for watchdog projection outputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProofWatchdogSeverity {
    /// Informational signal.
    Info,
    /// Warning-level signal.
    Warning,
    /// Critical signal requiring operator attention.
    Critical,
}

/// Projected watchdog incident built from consensus decision details.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofWatchdogProjection {
    /// Stable incident fingerprint identifier.
    pub incident_fingerprint: String,
    /// Message identifier tied to incident.
    pub message_id: String,
    /// Artifact identifier tied to incident.
    pub artifact_id: String,
    /// Projection kind classification.
    pub kind: ProofWatchdogProjectionKind,
    /// Incident severity classification.
    pub severity: ProofWatchdogSeverity,
    /// Required consensus quorum used for decision.
    pub required_quorum: usize,
    /// Validator count observed in decision.
    pub validator_count: usize,
    /// Valid attestation count in decision.
    pub valid_attestation_count: usize,
    /// Invalid attestation count in decision.
    pub invalid_attestation_count: usize,
    /// Replay attestation count in decision.
    pub replay_attestation_count: usize,
}

/// Stateless projector converting consensus decisions into watchdog incidents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ProofWatchdogProjector;

impl ProofWatchdogProjector {
    /// Construct proof-watchdog projector.
    pub fn new() -> Self {
        Self
    }

    /// Convert consensus decision into incident projection.
    pub fn project(&self, decision: &ValidatorProofConsensusDecision) -> ProofWatchdogProjection {
        let (kind, severity) = match decision.status {
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
        };

        let incident_fingerprint = format!(
            "proof-consensus:{}:{}:{}:{}:{}:{}",
            decision.message_id,
            decision.artifact_id,
            proof_watchdog_kind_code(kind),
            decision.valid_attestation_count,
            decision.invalid_attestation_count,
            decision.replay_attestation_count,
        );

        ProofWatchdogProjection {
            incident_fingerprint,
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

/// Errors emitted by proof-option evaluation, witness generation, and proof-admission flows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ZkDesignError {
    /// Evaluation policy values are invalid.
    InvalidPolicy(String),
    /// Candidate option contains invalid/inconsistent fields.
    InvalidOption {
        /// Option name.
        option: String,
        /// Validation reason.
        reason: String,
    },
    /// No options were supplied for ranking.
    EmptyOptionSet,
    /// Non-empty option set did not produce a ranked recommendation.
    RankingInvariantViolated,
    /// Private field selector is syntactically invalid.
    InvalidPrivateField(String),
    /// Requested private field selector was absent in envelope.
    MissingPrivateField(String),
    /// Processor proof artifact is invalid.
    InvalidProofArtifact(String),
    /// Artifact message id did not match expected message id.
    ProofArtifactMessageMismatch {
        /// Expected message identifier.
        expected: String,
        /// Found message identifier.
        found: String,
    },
    /// Artifact payload commitment did not match expected commitment.
    ProofArtifactCommitmentMismatch {
        /// Expected payload commitment.
        expected: String,
        /// Found payload commitment.
        found: String,
    },
    /// Artifact identifier has already been admitted.
    ProofArtifactReplay(String),
    /// Deterministic proof verification failed.
    ProofVerificationFailed {
        /// Artifact identifier.
        artifact_id: String,
        /// Verification failure reason.
        reason: String,
    },
    /// Wrapped canonical envelope validation failure.
    EnvelopeError(MessageEnvelopeError),
}

impl fmt::Display for ZkDesignError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPolicy(message) => write!(f, "invalid policy: {message}"),
            Self::InvalidOption { option, reason } => {
                write!(f, "invalid option `{option}`: {reason}")
            }
            Self::EmptyOptionSet => write!(f, "at least one architecture option is required"),
            Self::RankingInvariantViolated => {
                write!(
                    f,
                    "non-empty architecture option set did not produce a ranked recommendation"
                )
            }
            Self::InvalidPrivateField(message) => write!(f, "invalid private field: {message}"),
            Self::MissingPrivateField(field) => {
                write!(
                    f,
                    "private field `{field}` is missing from envelope body payload"
                )
            }
            Self::InvalidProofArtifact(message) => write!(f, "invalid proof artifact: {message}"),
            Self::ProofArtifactMessageMismatch { expected, found } => write!(
                f,
                "proof artifact message mismatch: expected {expected}, found {found}"
            ),
            Self::ProofArtifactCommitmentMismatch { expected, found } => write!(
                f,
                "proof artifact commitment mismatch: expected {expected}, found {found}"
            ),
            Self::ProofArtifactReplay(artifact_id) => {
                write!(f, "proof artifact replay detected: {artifact_id}")
            }
            Self::ProofVerificationFailed {
                artifact_id,
                reason,
            } => write!(
                f,
                "proof verification failed for artifact {artifact_id}: {reason}"
            ),
            Self::EnvelopeError(error) => write!(f, "invalid canonical envelope: {error}"),
        }
    }
}

impl std::error::Error for ZkDesignError {}

/// Return baseline phase-4 architecture options for policy evaluation.
pub fn phase4_baseline_options() -> Vec<ZkArchitectureOption> {
    vec![
        ZkArchitectureOption {
            name: "groth16-processor-only".to_owned(),
            proof_system: ZkProofSystem::Groth16,
            verification_topology: ZkVerificationTopology::ProcessorOnly,
            trusted_setup_required: true,
            deterministic_witness_inputs: true,
            prover_latency_ms: 120,
            verifier_latency_ms: 4,
            proof_size_bytes: 192,
            supports_batching: false,
            estimated_engineering_weeks: 7,
        },
        ZkArchitectureOption {
            name: "plonkish-batched-envelope".to_owned(),
            proof_system: ZkProofSystem::Plonkish,
            verification_topology: ZkVerificationTopology::ValidatorQuorum,
            trusted_setup_required: false,
            deterministic_witness_inputs: true,
            prover_latency_ms: 180,
            verifier_latency_ms: 15,
            proof_size_bytes: 896,
            supports_batching: true,
            estimated_engineering_weeks: 10,
        },
        ZkArchitectureOption {
            name: "stark-recursive-watchdog".to_owned(),
            proof_system: ZkProofSystem::Stark,
            verification_topology: ZkVerificationTopology::WatchdogSampling,
            trusted_setup_required: false,
            deterministic_witness_inputs: true,
            prover_latency_ms: 360,
            verifier_latency_ms: 45,
            proof_size_bytes: 4_608,
            supports_batching: true,
            estimated_engineering_weeks: 14,
        },
    ]
}

/// Evaluate one architecture option against policy thresholds and risk rules.
pub fn evaluate_zk_option(
    option: &ZkArchitectureOption,
    policy: ZkEvaluationPolicy,
) -> Result<ZkOptionAssessment, ZkDesignError> {
    validate_policy(policy)?;
    validate_option(option)?;

    let mut score = 100_i32;
    let mut risks = Vec::new();
    let mut trust_assumptions = Vec::new();

    match option.verification_topology {
        ZkVerificationTopology::ProcessorOnly => trust_assumptions.push(
            "single active processor enforces verification before block publication.".to_owned(),
        ),
        ZkVerificationTopology::ValidatorQuorum => trust_assumptions.push(
            "deterministic re-execution includes verifier checks across validator quorum."
                .to_owned(),
        ),
        ZkVerificationTopology::WatchdogSampling => {
            trust_assumptions.push("watchdog sampling confirms verification integrity.".to_owned())
        }
    }

    if option.trusted_setup_required {
        trust_assumptions.push(
            "trusted setup ceremony participants are honest and transcript integrity is preserved."
                .to_owned(),
        );
    } else {
        trust_assumptions.push("transparent setup avoids ceremony trust assumptions.".to_owned());
    }

    if !option.deterministic_witness_inputs {
        score -= 45;
        risks.push(ZkRisk {
            code: "nondeterministic-witness".to_owned(),
            severity: ZkRiskSeverity::High,
            detail: "witness generation is not reproducible across validator re-execution."
                .to_owned(),
        });
    }

    if option.verifier_latency_ms > policy.max_verifier_latency_ms {
        score -= 24;
        risks.push(ZkRisk {
            code: "verifier-latency".to_owned(),
            severity: ZkRiskSeverity::Medium,
            detail: format!(
                "verifier latency {}ms exceeds policy limit {}ms",
                option.verifier_latency_ms, policy.max_verifier_latency_ms
            ),
        });
    }

    if option.proof_size_bytes > policy.max_proof_size_bytes {
        score -= 22;
        risks.push(ZkRisk {
            code: "proof-size".to_owned(),
            severity: ZkRiskSeverity::Medium,
            detail: format!(
                "proof size {} bytes exceeds policy limit {} bytes",
                option.proof_size_bytes, policy.max_proof_size_bytes
            ),
        });
    }

    if option.estimated_engineering_weeks > policy.max_engineering_weeks {
        score -= 18;
        risks.push(ZkRisk {
            code: "delivery-complexity".to_owned(),
            severity: ZkRiskSeverity::Medium,
            detail: format!(
                "delivery estimate {} weeks exceeds phase budget {} weeks",
                option.estimated_engineering_weeks, policy.max_engineering_weeks
            ),
        });
    }

    if policy.require_transparent_setup && option.trusted_setup_required {
        score -= 30;
        risks.push(ZkRisk {
            code: "trusted-setup-policy".to_owned(),
            severity: ZkRiskSeverity::High,
            detail:
                "policy requires transparent setup, but option depends on trusted setup ceremony."
                    .to_owned(),
        });
    }

    if !option.supports_batching {
        score -= 8;
        risks.push(ZkRisk {
            code: "no-batching".to_owned(),
            severity: ZkRiskSeverity::Low,
            detail: "option lacks proof batching and may cap throughput under swarm load."
                .to_owned(),
        });
    }

    score = score.max(0);

    let feasible = option.deterministic_witness_inputs
        && option.verifier_latency_ms <= policy.max_verifier_latency_ms
        && option.proof_size_bytes <= policy.max_proof_size_bytes
        && option.estimated_engineering_weeks <= policy.max_engineering_weeks
        && (!policy.require_transparent_setup || !option.trusted_setup_required);

    Ok(ZkOptionAssessment {
        option_name: option.name.clone(),
        score,
        feasible,
        trust_assumptions,
        risks,
    })
}

/// Recommend phase-4 proof adoption plan from ranked architecture options.
pub fn recommend_phase4_plan(
    options: &[ZkArchitectureOption],
    policy: ZkEvaluationPolicy,
) -> Result<ZkPhasePlan, ZkDesignError> {
    validate_policy(policy)?;
    if options.is_empty() {
        return Err(ZkDesignError::EmptyOptionSet);
    }

    let mut ranked = Vec::with_capacity(options.len());
    for option in options {
        let assessment = evaluate_zk_option(option, policy)?;
        ranked.push((option.clone(), assessment));
    }

    ranked.sort_by(
        |(left_option, left_assessment), (right_option, right_assessment)| {
            right_assessment
                .score
                .cmp(&left_assessment.score)
                .then_with(|| {
                    left_high_risk_count(left_assessment)
                        .cmp(&left_high_risk_count(right_assessment))
                })
                .then_with(|| {
                    left_option
                        .verifier_latency_ms
                        .cmp(&right_option.verifier_latency_ms)
                })
                .then_with(|| {
                    left_option
                        .proof_size_bytes
                        .cmp(&right_option.proof_size_bytes)
                })
                .then_with(|| {
                    left_option
                        .estimated_engineering_weeks
                        .cmp(&right_option.estimated_engineering_weeks)
                })
        },
    );

    let (recommended_option, recommended_assessment) = ranked
        .first()
        .ok_or(ZkDesignError::RankingInvariantViolated)?;
    let recommended_option_name = recommended_option.name.clone();
    let recommended_score = recommended_assessment.score;
    let recommended_feasible = recommended_assessment.feasible;

    let transparency_note = if policy.require_transparent_setup {
        "transparent setup is required and "
    } else {
        ""
    };

    let rationale = if recommended_feasible {
        format!(
            "Selected `{}` because {}it satisfies verifier/proof-size budgets with score {}.",
            recommended_option_name, transparency_note, recommended_score
        )
    } else {
        format!(
            "Selected `{}` as least-risk fallback with score {}, but follow-up risk burn-down is required.",
            recommended_option_name, recommended_score
        )
    };

    let milestones = vec![
        ZkPhaseMilestone {
            phase: "Phase 4.0 - Feasibility harness".to_owned(),
            objective: format!(
                "Implement deterministic witness harness for `{}` using canonical envelope payloads.",
                recommended_option_name
            ),
            validation_focus:
                "Unit + functional validation for policy scoring and witness commitments."
                    .to_owned(),
            exit_criteria: vec![
                "Witness commitment remains stable across repeated executions.".to_owned(),
                "Policy errors are explicit for invalid boundaries.".to_owned(),
            ],
        },
        ZkPhaseMilestone {
            phase: "Phase 4.1 - Processor verification pilot".to_owned(),
            objective:
                "Attach proof verification to processor transaction validation in bounded fast-lane path."
                    .to_owned(),
            validation_focus:
                "Integration tests over message lifecycle with proof verification hooks.".to_owned(),
            exit_criteria: vec![
                "Processor rejects unverifiable proofs deterministically.".to_owned(),
                "Verifier runtime remains within policy budget under representative load."
                    .to_owned(),
            ],
        },
        ZkPhaseMilestone {
            phase: "Phase 4.2 - Validator and watchdog expansion".to_owned(),
            objective:
                "Extend verification to validator quorum and watchdog sampling for abuse detection."
                    .to_owned(),
            validation_focus:
                "Regression tests for censorship, replay, and invalid-proof propagation."
                    .to_owned(),
            exit_criteria: vec![
                "Quorum paths align on proof validity outcomes.".to_owned(),
                "Watchdog alerts isolate invalid-proof mismatches without false positives."
                    .to_owned(),
            ],
        },
    ];

    let assessments = ranked
        .iter()
        .map(|(_, assessment)| assessment.clone())
        .collect::<Vec<_>>();

    Ok(ZkPhasePlan {
        recommended_option: recommended_option_name,
        rationale,
        milestones,
        assessments,
    })
}

/// Build zero-knowledge witness projection from canonical envelope and private-field selectors.
pub fn build_message_witness(
    envelope: &CanonicalMessageEnvelope,
    private_fields: &[&str],
) -> Result<ZkMessageWitness, ZkDesignError> {
    envelope.validate().map_err(ZkDesignError::EnvelopeError)?;

    let mut hidden = BTreeSet::new();
    for field in private_fields {
        if field.trim().is_empty() {
            return Err(ZkDesignError::InvalidPrivateField(
                "private field names must not be empty".to_owned(),
            ));
        }
        if !is_valid_private_field_selector(field) {
            return Err(ZkDesignError::InvalidPrivateField(format!(
                "private field selector `{field}` must contain only [A-Za-z0-9_.-] and no empty path segments"
            )));
        }
        if !envelope.body.contains_key(*field) {
            return Err(ZkDesignError::MissingPrivateField((*field).to_owned()));
        }
        hidden.insert((*field).to_owned());
    }

    let canonical_payload = envelope.canonical_payload();
    let mut redacted_body = String::new();
    let mut revealed_fields = Vec::new();
    for (key, value) in &envelope.body {
        redacted_body.push_str(key);
        redacted_body.push('=');
        if hidden.contains(key) {
            redacted_body.push_str("<hidden>");
        } else {
            redacted_body.push_str(value);
            revealed_fields.push(key.clone());
        }
        redacted_body.push(';');
    }

    let hidden_list = hidden.into_iter().collect::<Vec<_>>().join(",");
    let commitment_input =
        format!("{canonical_payload}|redacted:{redacted_body}|hidden:{hidden_list}");
    let public_commitment = format!("fnv1a64:{:016x}", fnv1a_64(commitment_input.as_bytes()));

    Ok(ZkMessageWitness {
        public_commitment,
        revealed_fields,
        hidden_field_count: private_fields
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            .len(),
        payload_bytes: canonical_payload.len(),
    })
}

fn left_high_risk_count(assessment: &ZkOptionAssessment) -> usize {
    assessment
        .risks
        .iter()
        .filter(|risk| risk.severity == ZkRiskSeverity::High)
        .count()
}

fn require_non_empty_artifact_field(field: &str, value: &str) -> Result<(), ZkDesignError> {
    if value.trim().is_empty() {
        return Err(ZkDesignError::InvalidProofArtifact(format!(
            "{field} must not be empty"
        )));
    }
    Ok(())
}

fn is_valid_private_field_selector(selector: &str) -> bool {
    let trimmed = selector.trim();
    if trimmed.is_empty() || trimmed.starts_with('.') || trimmed.ends_with('.') {
        return false;
    }
    if trimmed.contains("..") {
        return false;
    }
    trimmed
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' || ch == '.')
}

fn require_non_empty_consensus_field(
    field: &'static str,
    value: &str,
) -> Result<(), ValidatorProofConsensusError> {
    if value.trim().is_empty() {
        return Err(ValidatorProofConsensusError::InvalidField { field });
    }
    Ok(())
}

fn proof_watchdog_kind_code(kind: ProofWatchdogProjectionKind) -> &'static str {
    match kind {
        ProofWatchdogProjectionKind::ConsensusAligned => "consensus-aligned",
        ProofWatchdogProjectionKind::InvalidProofConsensus => "invalid-proof-consensus",
        ProofWatchdogProjectionKind::ReplayProofConsensus => "replay-proof-consensus",
        ProofWatchdogProjectionKind::ValidatorMismatch => "validator-mismatch",
    }
}

fn validate_policy(policy: ZkEvaluationPolicy) -> Result<(), ZkDesignError> {
    if policy.max_verifier_latency_ms == 0 {
        return Err(ZkDesignError::InvalidPolicy(
            "max_verifier_latency_ms must be greater than zero".to_owned(),
        ));
    }
    if policy.max_proof_size_bytes == 0 {
        return Err(ZkDesignError::InvalidPolicy(
            "max_proof_size_bytes must be greater than zero".to_owned(),
        ));
    }
    if policy.max_engineering_weeks == 0 {
        return Err(ZkDesignError::InvalidPolicy(
            "max_engineering_weeks must be greater than zero".to_owned(),
        ));
    }
    Ok(())
}

fn validate_option(option: &ZkArchitectureOption) -> Result<(), ZkDesignError> {
    if option.name.trim().is_empty() {
        return Err(ZkDesignError::InvalidOption {
            option: option.name.clone(),
            reason: "name must not be empty".to_owned(),
        });
    }
    if option.prover_latency_ms == 0 {
        return Err(ZkDesignError::InvalidOption {
            option: option.name.clone(),
            reason: "prover_latency_ms must be greater than zero".to_owned(),
        });
    }
    if option.verifier_latency_ms == 0 {
        return Err(ZkDesignError::InvalidOption {
            option: option.name.clone(),
            reason: "verifier_latency_ms must be greater than zero".to_owned(),
        });
    }
    if option.proof_size_bytes == 0 {
        return Err(ZkDesignError::InvalidOption {
            option: option.name.clone(),
            reason: "proof_size_bytes must be greater than zero".to_owned(),
        });
    }
    if option.estimated_engineering_weeks == 0 {
        return Err(ZkDesignError::InvalidOption {
            option: option.name.clone(),
            reason: "estimated_engineering_weeks must be greater than zero".to_owned(),
        });
    }

    Ok(())
}

fn fnv1a_64(input: &[u8]) -> u64 {
    const OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;

    let mut hash = OFFSET_BASIS;
    for byte in input {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::{
        evaluate_zk_option, phase4_baseline_options, ProcessorProofAdmissionEvaluator,
        ProcessorProofAdmissionInput, ProcessorProofArtifact, ProofWatchdogProjectionKind,
        ProofWatchdogProjector, ProofWatchdogSeverity, ValidatorProofAttestation,
        ValidatorProofConsensusError, ValidatorProofConsensusEvaluator,
        ValidatorProofConsensusInput, ValidatorProofConsensusStatus, ValidatorProofVerdict,
        ZkArchitectureOption, ZkDesignError, ZkEvaluationPolicy, ZkProofSystem,
        ZkVerificationTopology,
    };

    #[test]
    fn transparent_policy_penalizes_trusted_setup_options() {
        let options = phase4_baseline_options();
        let result = evaluate_zk_option(&options[0], ZkEvaluationPolicy::default())
            .expect("evaluation should succeed");
        assert!(!result.feasible);
        assert!(result
            .risks
            .iter()
            .any(|risk| risk.code == "trusted-setup-policy"));
    }

    #[test]
    fn option_validation_rejects_zero_proof_size() {
        let option = ZkArchitectureOption {
            name: "invalid".to_owned(),
            proof_system: ZkProofSystem::Plonkish,
            verification_topology: ZkVerificationTopology::ValidatorQuorum,
            trusted_setup_required: false,
            deterministic_witness_inputs: true,
            prover_latency_ms: 10,
            verifier_latency_ms: 10,
            proof_size_bytes: 0,
            supports_batching: true,
            estimated_engineering_weeks: 2,
        };

        let result = evaluate_zk_option(&option, ZkEvaluationPolicy::default());
        assert_eq!(
            result,
            Err(ZkDesignError::InvalidOption {
                option: "invalid".to_owned(),
                reason: "proof_size_bytes must be greater than zero".to_owned(),
            })
        );
    }

    #[test]
    fn processor_proof_artifact_rejects_empty_artifact_id() {
        assert_eq!(
            ProcessorProofArtifact::new(
                "",
                "urn:uuid:message-1",
                "fnv1a64:abc",
                "proof:ok:artifact-1",
            ),
            Err(ZkDesignError::InvalidProofArtifact(
                "artifact_id must not be empty".to_owned()
            ))
        );
    }

    #[test]
    fn processor_admission_rejects_invalid_proof_value() {
        let artifact = ProcessorProofArtifact::new(
            "artifact-1",
            "urn:uuid:message-1",
            "fnv1a64:abc",
            "proof:tampered:artifact-1",
        )
        .expect("artifact should parse");
        let input =
            ProcessorProofAdmissionInput::new("urn:uuid:message-1", "fnv1a64:abc", artifact)
                .expect("input should parse");
        let mut evaluator = ProcessorProofAdmissionEvaluator::new();

        assert_eq!(
            evaluator.evaluate(input),
            Err(ZkDesignError::ProofVerificationFailed {
                artifact_id: "artifact-1".to_owned(),
                reason: "proof value failed deterministic verification".to_owned(),
            })
        );
    }

    #[test]
    fn validator_attestation_rejects_invalid_did() {
        let error = ValidatorProofAttestation::new(
            "attestation-1",
            "validator-a",
            "urn:uuid:message-1",
            "artifact-1",
            ValidatorProofVerdict::Valid,
        )
        .expect_err("invalid validator did should be rejected");
        assert!(matches!(
            error,
            ValidatorProofConsensusError::InvalidValidatorDid(_)
        ));
    }

    #[test]
    fn validator_consensus_rejects_duplicate_validator_attestations() {
        let mut evaluator =
            ValidatorProofConsensusEvaluator::new(2).expect("valid quorum should build");
        let input = ValidatorProofConsensusInput::new(
            "urn:uuid:message-1",
            "artifact-1",
            vec![
                ValidatorProofAttestation::new(
                    "attestation-1",
                    "kamn:did:agent:validator-a",
                    "urn:uuid:message-1",
                    "artifact-1",
                    ValidatorProofVerdict::Valid,
                )
                .expect("valid attestation"),
                ValidatorProofAttestation::new(
                    "attestation-2",
                    "kamn:did:agent:validator-a",
                    "urn:uuid:message-1",
                    "artifact-1",
                    ValidatorProofVerdict::Valid,
                )
                .expect("valid attestation"),
            ],
        )
        .expect("input should parse");

        assert_eq!(
            evaluator.evaluate(input),
            Err(ValidatorProofConsensusError::DuplicateValidator(
                "kamn:did:agent:validator-a".to_owned()
            ))
        );
    }

    #[test]
    fn watchdog_projection_is_nominal_for_aligned_valid_consensus() {
        let mut evaluator =
            ValidatorProofConsensusEvaluator::new(2).expect("valid quorum should build");
        let input = ValidatorProofConsensusInput::new(
            "urn:uuid:message-1",
            "artifact-1",
            vec![
                ValidatorProofAttestation::new(
                    "attestation-1",
                    "kamn:did:agent:validator-z",
                    "urn:uuid:message-1",
                    "artifact-1",
                    ValidatorProofVerdict::Valid,
                )
                .expect("valid attestation"),
                ValidatorProofAttestation::new(
                    "attestation-2",
                    "kamn:did:agent:validator-a",
                    "urn:uuid:message-1",
                    "artifact-1",
                    ValidatorProofVerdict::Valid,
                )
                .expect("valid attestation"),
            ],
        )
        .expect("input should parse");
        let decision = evaluator
            .evaluate(input)
            .expect("aligned valid consensus should succeed");
        let projection = ProofWatchdogProjector::new().project(&decision);

        assert_eq!(
            decision.status,
            ValidatorProofConsensusStatus::ConsensusValid
        );
        assert_eq!(
            decision.validator_dids,
            vec![
                "kamn:did:agent:validator-a".to_owned(),
                "kamn:did:agent:validator-z".to_owned()
            ]
        );
        assert_eq!(
            projection.kind,
            ProofWatchdogProjectionKind::ConsensusAligned
        );
        assert_eq!(projection.severity, ProofWatchdogSeverity::Info);
    }
}
