use super::*;
use crate::zk_message_proofs::errors::ZkDesignError;

pub fn phase4_baseline_options() -> Vec<ZkArchitectureOption> {
    vec![
        option(
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
        option(
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
        option(
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
    ]
}

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

pub fn recommend_phase4_plan(
    options: &[ZkArchitectureOption],
    policy: ZkEvaluationPolicy,
) -> Result<ZkPhasePlan, ZkDesignError> {
    validate_policy(policy)?;
    if options.is_empty() {
        return Err(ZkDesignError::EmptyOptionSet);
    }
    let mut ranked = build_ranked_options(options, policy)?;
    ranked.sort_by(compare_ranked_options);
    let (recommended_option, recommended_assessment) = ranked
        .first()
        .ok_or(ZkDesignError::RankingInvariantViolated)?;
    Ok(ZkPhasePlan {
        recommended_option: recommended_option.name.clone(),
        rationale: recommendation_rationale(recommended_option, recommended_assessment, policy),
        milestones: build_phase_milestones(&recommended_option.name),
        assessments: ranked
            .into_iter()
            .map(|(_, assessment)| assessment)
            .collect(),
    })
}

fn option(
    name: &str,
    proof_system: ZkProofSystem,
    topology: ZkVerificationTopology,
    trusted_setup_required: bool,
    prover_latency_ms: u64,
    verifier_latency_ms: u64,
    proof_size_bytes: u64,
    supports_batching: bool,
    estimated_engineering_weeks: u16,
) -> ZkArchitectureOption {
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

fn base_assessment(option: &ZkArchitectureOption) -> ZkOptionAssessment {
    ZkOptionAssessment {
        option_name: option.name.clone(),
        score: 100,
        feasible: false,
        trust_assumptions: trust_assumptions(option),
        risks: Vec::new(),
    }
}

fn trust_assumptions(option: &ZkArchitectureOption) -> Vec<String> {
    let mut values = vec![verification_assumption(option.verification_topology)];
    values.push(setup_assumption(option.trusted_setup_required));
    values
}

fn verification_assumption(topology: ZkVerificationTopology) -> String {
    match topology {
        ZkVerificationTopology::ProcessorOnly => {
            "single active processor enforces verification before block publication.".to_owned()
        }
        ZkVerificationTopology::ValidatorQuorum => {
            "deterministic re-execution includes verifier checks across validator quorum."
                .to_owned()
        }
        ZkVerificationTopology::WatchdogSampling => {
            "watchdog sampling confirms verification integrity.".to_owned()
        }
    }
}

fn setup_assumption(trusted_setup_required: bool) -> String {
    if trusted_setup_required {
        "trusted setup ceremony participants are honest and transcript integrity is preserved."
            .to_owned()
    } else {
        "transparent setup avoids ceremony trust assumptions.".to_owned()
    }
}

fn apply_witness_determinism(option: &ZkArchitectureOption, assessment: &mut ZkOptionAssessment) {
    if option.deterministic_witness_inputs {
        return;
    }
    assessment.score -= 45;
    assessment.risks.push(ZkRisk {
        code: "nondeterministic-witness".to_owned(),
        severity: ZkRiskSeverity::High,
        detail: "witness generation is not reproducible across validator re-execution.".to_owned(),
    });
}

fn apply_verifier_latency(
    option: &ZkArchitectureOption,
    policy: ZkEvaluationPolicy,
    assessment: &mut ZkOptionAssessment,
) {
    if option.verifier_latency_ms <= policy.max_verifier_latency_ms {
        return;
    }
    assessment.score -= 24;
    assessment.risks.push(ZkRisk {
        code: "verifier-latency".to_owned(),
        severity: ZkRiskSeverity::Medium,
        detail: format!(
            "verifier latency {}ms exceeds policy limit {}ms",
            option.verifier_latency_ms, policy.max_verifier_latency_ms
        ),
    });
}

fn apply_proof_size(
    option: &ZkArchitectureOption,
    policy: ZkEvaluationPolicy,
    assessment: &mut ZkOptionAssessment,
) {
    if option.proof_size_bytes <= policy.max_proof_size_bytes {
        return;
    }
    assessment.score -= 22;
    assessment.risks.push(ZkRisk {
        code: "proof-size".to_owned(),
        severity: ZkRiskSeverity::Medium,
        detail: format!(
            "proof size {} bytes exceeds policy limit {} bytes",
            option.proof_size_bytes, policy.max_proof_size_bytes
        ),
    });
}

fn apply_engineering_budget(
    option: &ZkArchitectureOption,
    policy: ZkEvaluationPolicy,
    assessment: &mut ZkOptionAssessment,
) {
    if option.estimated_engineering_weeks <= policy.max_engineering_weeks {
        return;
    }
    assessment.score -= 18;
    assessment.risks.push(ZkRisk {
        code: "delivery-complexity".to_owned(),
        severity: ZkRiskSeverity::Medium,
        detail: format!(
            "delivery estimate {} weeks exceeds phase budget {} weeks",
            option.estimated_engineering_weeks, policy.max_engineering_weeks
        ),
    });
}

fn apply_setup_policy(
    option: &ZkArchitectureOption,
    policy: ZkEvaluationPolicy,
    assessment: &mut ZkOptionAssessment,
) {
    if !policy.require_transparent_setup || !option.trusted_setup_required {
        return;
    }
    assessment.score -= 30;
    assessment.risks.push(ZkRisk {
        code: "trusted-setup-policy".to_owned(),
        severity: ZkRiskSeverity::High,
        detail: "policy requires transparent setup, but option depends on trusted setup ceremony."
            .to_owned(),
    });
}

fn apply_batching_penalty(option: &ZkArchitectureOption, assessment: &mut ZkOptionAssessment) {
    if option.supports_batching {
        return;
    }
    assessment.score -= 8;
    assessment.risks.push(ZkRisk {
        code: "no-batching".to_owned(),
        severity: ZkRiskSeverity::Low,
        detail: "option lacks proof batching and may cap throughput under swarm load.".to_owned(),
    });
}

fn is_feasible(option: &ZkArchitectureOption, policy: ZkEvaluationPolicy) -> bool {
    option.deterministic_witness_inputs
        && option.verifier_latency_ms <= policy.max_verifier_latency_ms
        && option.proof_size_bytes <= policy.max_proof_size_bytes
        && option.estimated_engineering_weeks <= policy.max_engineering_weeks
        && (!policy.require_transparent_setup || !option.trusted_setup_required)
}

fn build_ranked_options(
    options: &[ZkArchitectureOption],
    policy: ZkEvaluationPolicy,
) -> Result<Vec<(ZkArchitectureOption, ZkOptionAssessment)>, ZkDesignError> {
    options
        .iter()
        .map(|option| Ok((option.clone(), evaluate_zk_option(option, policy)?)))
        .collect()
}

fn compare_ranked_options(
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

fn high_risk_count(assessment: &ZkOptionAssessment) -> usize {
    assessment
        .risks
        .iter()
        .filter(|risk| risk.severity == ZkRiskSeverity::High)
        .count()
}

fn recommendation_rationale(
    option: &ZkArchitectureOption,
    assessment: &ZkOptionAssessment,
    policy: ZkEvaluationPolicy,
) -> String {
    let transparency_note = if policy.require_transparent_setup {
        "transparent setup is required and "
    } else {
        ""
    };
    if assessment.feasible {
        format!(
            "Selected `{}` because {}it satisfies verifier/proof-size budgets with score {}.",
            option.name, transparency_note, assessment.score
        )
    } else {
        format!("Selected `{}` as least-risk fallback with score {}, but follow-up risk burn-down is required.", option.name, assessment.score)
    }
}

fn build_phase_milestones(option_name: &str) -> Vec<ZkPhaseMilestone> {
    vec![
        ZkPhaseMilestone { phase: "Phase 4.0 - Feasibility harness".to_owned(), objective: format!("Implement deterministic witness harness for `{}` using canonical envelope payloads.", option_name), validation_focus: "Unit + functional validation for policy scoring and witness commitments.".to_owned(), exit_criteria: vec!["Witness commitment remains stable across repeated executions.".to_owned(), "Policy errors are explicit for invalid boundaries.".to_owned()] },
        ZkPhaseMilestone { phase: "Phase 4.1 - Processor verification pilot".to_owned(), objective: "Attach proof verification to processor transaction validation in bounded fast-lane path.".to_owned(), validation_focus: "Integration tests over message lifecycle with proof verification hooks.".to_owned(), exit_criteria: vec!["Processor rejects unverifiable proofs deterministically.".to_owned(), "Verifier runtime remains within policy budget under representative load.".to_owned()] },
        ZkPhaseMilestone { phase: "Phase 4.2 - Validator and watchdog expansion".to_owned(), objective: "Extend verification to validator quorum and watchdog sampling for abuse detection.".to_owned(), validation_focus: "Regression tests for censorship, replay, and invalid-proof propagation.".to_owned(), exit_criteria: vec!["Quorum paths align on proof validity outcomes.".to_owned(), "Watchdog alerts isolate invalid-proof mismatches without false positives.".to_owned()] },
    ]
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
        return invalid_option(&option.name, "name must not be empty");
    }
    if option.prover_latency_ms == 0 {
        return invalid_option(&option.name, "prover_latency_ms must be greater than zero");
    }
    if option.verifier_latency_ms == 0 {
        return invalid_option(
            &option.name,
            "verifier_latency_ms must be greater than zero",
        );
    }
    if option.proof_size_bytes == 0 {
        return invalid_option(&option.name, "proof_size_bytes must be greater than zero");
    }
    if option.estimated_engineering_weeks == 0 {
        return invalid_option(
            &option.name,
            "estimated_engineering_weeks must be greater than zero",
        );
    }
    Ok(())
}

fn invalid_option(option: &str, reason: &str) -> Result<(), ZkDesignError> {
    Err(ZkDesignError::InvalidOption {
        option: option.to_owned(),
        reason: reason.to_owned(),
    })
}
