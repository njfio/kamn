mod evaluation;

pub use evaluation::{evaluate_zk_option, phase4_baseline_options, recommend_phase4_plan};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZkProofSystem {
    Groth16,
    Plonkish,
    Stark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZkVerificationTopology {
    ProcessorOnly,
    ValidatorQuorum,
    WatchdogSampling,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ZkRiskSeverity {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZkRisk {
    pub code: String,
    pub severity: ZkRiskSeverity,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZkArchitectureOption {
    pub name: String,
    pub proof_system: ZkProofSystem,
    pub verification_topology: ZkVerificationTopology,
    pub trusted_setup_required: bool,
    pub deterministic_witness_inputs: bool,
    pub prover_latency_ms: u64,
    pub verifier_latency_ms: u64,
    pub proof_size_bytes: u64,
    pub supports_batching: bool,
    pub estimated_engineering_weeks: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZkEvaluationPolicy {
    pub max_verifier_latency_ms: u64,
    pub max_proof_size_bytes: u64,
    pub max_engineering_weeks: u16,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZkOptionAssessment {
    pub option_name: String,
    pub score: i32,
    pub feasible: bool,
    pub trust_assumptions: Vec<String>,
    pub risks: Vec<ZkRisk>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZkPhaseMilestone {
    pub phase: String,
    pub objective: String,
    pub validation_focus: String,
    pub exit_criteria: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZkPhasePlan {
    pub recommended_option: String,
    pub rationale: String,
    pub milestones: Vec<ZkPhaseMilestone>,
    pub assessments: Vec<ZkOptionAssessment>,
}
