use super::scoring::{
    apply_batching_penalty, apply_engineering_budget, apply_proof_size, apply_setup_policy,
    apply_verifier_latency, apply_witness_determinism, base_assessment, high_risk_count,
    is_feasible,
};
use super::validation::{validate_option, validate_policy};
use super::*;
use crate::zk_message_proofs::errors::ZkDesignError;

type BaselineSpec = (
    &'static str,
    ZkProofSystem,
    ZkVerificationTopology,
    bool,
    u64,
    u64,
    u64,
    bool,
    u16,
);

const PHASE4_BASELINE_SPECS: [BaselineSpec; 3] = [
    (
        "groth16-processor-only",
        ZkProofSystem::Groth16,
        ZkVerificationTopology::ProcessorOnly,
        true,
        120,
        4,
        192,
        false,
        7,
    ),
    (
        "plonkish-batched-envelope",
        ZkProofSystem::Plonkish,
        ZkVerificationTopology::ValidatorQuorum,
        false,
        180,
        15,
        896,
        true,
        10,
    ),
    (
        "stark-recursive-watchdog",
        ZkProofSystem::Stark,
        ZkVerificationTopology::WatchdogSampling,
        false,
        360,
        45,
        4_608,
        true,
        14,
    ),
];

/// Runs the phase4 baseline options contract helper.
pub fn phase4_baseline_options() -> Vec<ZkArchitectureOption> {
    PHASE4_BASELINE_SPECS
        .into_iter()
        .map(option_from_spec)
        .collect()
}

/// Runs the evaluate zk option contract helper.
pub fn evaluate_zk_option(
    option: &ZkArchitectureOption,
    policy: ZkEvaluationPolicy,
) -> Result<ZkOptionAssessment, ZkDesignError> {
    validate_policy(policy)?;
    validate_option(option)?;
    let mut assessment = base_assessment(option);
    apply_witness_determinism(option, &mut assessment);
    apply_verifier_latency(option, policy, &mut assessment);
    apply_proof_size(option, policy, &mut assessment);
    apply_engineering_budget(option, policy, &mut assessment);
    apply_setup_policy(option, policy, &mut assessment);
    apply_batching_penalty(option, &mut assessment);
    assessment.score = assessment.score.max(0);
    assessment.feasible = is_feasible(option, policy);
    Ok(assessment)
}

fn option_from_spec(spec: BaselineSpec) -> ZkArchitectureOption {
    let (
        name,
        proof_system,
        topology,
        trusted_setup_required,
        prover_latency_ms,
        verifier_latency_ms,
        proof_size_bytes,
        supports_batching,
        estimated_engineering_weeks,
    ) = spec;
    ZkArchitectureOption {
        name: name.to_owned(),
        proof_system,
        verification_topology: topology,
        trusted_setup_required,
        deterministic_witness_inputs: true,
        prover_latency_ms,
        verifier_latency_ms,
        proof_size_bytes,
        supports_batching,
        estimated_engineering_weeks,
    }
}

pub(super) fn build_ranked_options(
    options: &[ZkArchitectureOption],
    policy: ZkEvaluationPolicy,
) -> Result<Vec<(ZkArchitectureOption, ZkOptionAssessment)>, ZkDesignError> {
    options
        .iter()
        .map(|option| Ok((option.clone(), evaluate_zk_option(option, policy)?)))
        .collect()
}

pub(super) fn compare_ranked_options(
    (left_option, left_assessment): &(ZkArchitectureOption, ZkOptionAssessment),
    (right_option, right_assessment): &(ZkArchitectureOption, ZkOptionAssessment),
) -> std::cmp::Ordering {
    right_assessment
        .score
        .cmp(&left_assessment.score)
        .then_with(|| high_risk_count(left_assessment).cmp(&high_risk_count(right_assessment)))
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
}
