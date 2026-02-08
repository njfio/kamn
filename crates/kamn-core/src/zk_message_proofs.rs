use crate::{CanonicalMessageEnvelope, MessageEnvelopeError};
use std::collections::BTreeSet;
use std::fmt;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZkMessageWitness {
    pub public_commitment: String,
    pub revealed_fields: Vec<String>,
    pub hidden_field_count: usize,
    pub payload_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ZkDesignError {
    InvalidPolicy(String),
    InvalidOption { option: String, reason: String },
    EmptyOptionSet,
    InvalidPrivateField(String),
    MissingPrivateField(String),
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
            Self::InvalidPrivateField(message) => write!(f, "invalid private field: {message}"),
            Self::MissingPrivateField(field) => {
                write!(
                    f,
                    "private field `{field}` is missing from envelope body payload"
                )
            }
            Self::EnvelopeError(error) => write!(f, "invalid canonical envelope: {error}"),
        }
    }
}

impl std::error::Error for ZkDesignError {}

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
        .expect("non-empty option set should produce ranked list");
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
        evaluate_zk_option, phase4_baseline_options, ZkArchitectureOption, ZkDesignError,
        ZkEvaluationPolicy, ZkProofSystem, ZkVerificationTopology,
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
}
