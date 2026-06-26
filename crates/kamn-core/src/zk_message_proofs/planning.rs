mod evaluation;
mod recommendation;
mod scoring;
mod validation;

pub use evaluation::{evaluate_zk_option, phase4_baseline_options};
pub use recommendation::recommend_phase4_plan;

/// Supported proof-system families considered by the phase-4 planner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZkProofSystem {
    /// Groth16 variant for this public contract enum.
    Groth16,
    /// Plonkish variant for this public contract enum.
    Plonkish,
    /// Stark variant for this public contract enum.
    Stark,
}

/// Deployment topology used to verify a proof once produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZkVerificationTopology {
    /// Processor only variant for this public contract enum.
    ProcessorOnly,
    /// Validator quorum variant for this public contract enum.
    ValidatorQuorum,
    /// Watchdog sampling variant for this public contract enum.
    WatchdogSampling,
}

/// Relative severity for evaluation risks attached to an option.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ZkRiskSeverity {
    /// Low variant for this public contract enum.
    Low,
    /// Medium variant for this public contract enum.
    Medium,
    /// High variant for this public contract enum.
    High,
}

/// A concrete risk emitted during option assessment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZkRisk {
    /// Code carried by this public contract model.
    pub code: String,
    /// Severity carried by this public contract model.
    pub severity: ZkRiskSeverity,
    /// Detail carried by this public contract model.
    pub detail: String,
}

/// A candidate architecture option scored by the planner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZkArchitectureOption {
    /// Name carried by this public contract model.
    pub name: String,
    /// Proof system carried by this public contract model.
    pub proof_system: ZkProofSystem,
    /// Verification topology carried by this public contract model.
    pub verification_topology: ZkVerificationTopology,
    /// Trusted setup required carried by this public contract model.
    pub trusted_setup_required: bool,
    /// Deterministic witness inputs carried by this public contract model.
    pub deterministic_witness_inputs: bool,
    /// Prover latency ms carried by this public contract model.
    pub prover_latency_ms: u64,
    /// Verifier latency ms carried by this public contract model.
    pub verifier_latency_ms: u64,
    /// Proof size bytes carried by this public contract model.
    pub proof_size_bytes: u64,
    /// Supports batching carried by this public contract model.
    pub supports_batching: bool,
    /// Estimated engineering weeks carried by this public contract model.
    pub estimated_engineering_weeks: u16,
}

/// Constraints applied while ranking architecture options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZkEvaluationPolicy {
    /// Max verifier latency ms carried by this public contract model.
    pub max_verifier_latency_ms: u64,
    /// Max proof size bytes carried by this public contract model.
    pub max_proof_size_bytes: u64,
    /// Max engineering weeks carried by this public contract model.
    pub max_engineering_weeks: u16,
    /// Require transparent setup carried by this public contract model.
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

/// The scored result for one architecture option.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZkOptionAssessment {
    /// Option name carried by this public contract model.
    pub option_name: String,
    /// Score carried by this public contract model.
    pub score: i32,
    /// Feasible carried by this public contract model.
    pub feasible: bool,
    /// Trust assumptions carried by this public contract model.
    pub trust_assumptions: Vec<String>,
    /// Risks carried by this public contract model.
    pub risks: Vec<ZkRisk>,
}

/// One staged milestone in the recommended rollout plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZkPhaseMilestone {
    /// Phase carried by this public contract model.
    pub phase: String,
    /// Objective carried by this public contract model.
    pub objective: String,
    /// Validation focus carried by this public contract model.
    pub validation_focus: String,
    /// Exit criteria carried by this public contract model.
    pub exit_criteria: Vec<String>,
}

/// The selected plan and supporting assessments returned by the planner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZkPhasePlan {
    /// Recommended option carried by this public contract model.
    pub recommended_option: String,
    /// Rationale carried by this public contract model.
    pub rationale: String,
    /// Milestones carried by this public contract model.
    pub milestones: Vec<ZkPhaseMilestone>,
    /// Assessments carried by this public contract model.
    pub assessments: Vec<ZkOptionAssessment>,
}
