use super::*;

pub(super) fn base_assessment(option: &ZkArchitectureOption) -> ZkOptionAssessment {
    ZkOptionAssessment {
        option_name: option.name.clone(),
        score: 100,
        feasible: false,
        trust_assumptions: trust_assumptions(option),
        risks: Vec::new(),
    }
}

pub(super) fn apply_witness_determinism(
    option: &ZkArchitectureOption,
    assessment: &mut ZkOptionAssessment,
) {
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

pub(super) fn apply_verifier_latency(
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

pub(super) fn apply_proof_size(
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

pub(super) fn apply_engineering_budget(
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

pub(super) fn apply_setup_policy(
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

pub(super) fn apply_batching_penalty(
    option: &ZkArchitectureOption,
    assessment: &mut ZkOptionAssessment,
) {
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

pub(super) fn is_feasible(option: &ZkArchitectureOption, policy: ZkEvaluationPolicy) -> bool {
    option.deterministic_witness_inputs
        && option.verifier_latency_ms <= policy.max_verifier_latency_ms
        && option.proof_size_bytes <= policy.max_proof_size_bytes
        && option.estimated_engineering_weeks <= policy.max_engineering_weeks
        && (!policy.require_transparent_setup || !option.trusted_setup_required)
}

pub(super) fn high_risk_count(assessment: &ZkOptionAssessment) -> usize {
    assessment
        .risks
        .iter()
        .filter(|risk| risk.severity == ZkRiskSeverity::High)
        .count()
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
