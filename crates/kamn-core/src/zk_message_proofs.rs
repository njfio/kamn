//! Zero-knowledge message-proof planning, admission, consensus, and watchdog projection contracts.

mod errors;
mod planning;
mod processor_admission;
mod validator_consensus;
mod watchdog_projection;
mod witness;

pub use errors::ZkDesignError;
pub use planning::{
    evaluate_zk_option, phase4_baseline_options, recommend_phase4_plan, ZkArchitectureOption,
    ZkEvaluationPolicy, ZkOptionAssessment, ZkPhaseMilestone, ZkPhasePlan, ZkProofSystem, ZkRisk,
    ZkRiskSeverity, ZkVerificationTopology,
};
pub use processor_admission::{
    ProcessorProofAdmissionDecision, ProcessorProofAdmissionEvaluator,
    ProcessorProofAdmissionInput, ProcessorProofArtifact,
};
pub use validator_consensus::{
    ValidatorProofAttestation, ValidatorProofConsensusDecision, ValidatorProofConsensusError,
    ValidatorProofConsensusEvaluator, ValidatorProofConsensusInput, ValidatorProofConsensusStatus,
    ValidatorProofVerdict,
};
pub use watchdog_projection::{
    ProofWatchdogProjection, ProofWatchdogProjectionKind, ProofWatchdogProjector,
    ProofWatchdogSeverity,
};
pub use witness::{build_message_witness, ZkMessageWitness};

#[cfg(test)]
mod tests;
