use super::evaluation::{build_ranked_options, compare_ranked_options};
use super::validation::validate_policy;
use super::*;
use crate::zk_message_proofs::errors::ZkDesignError;

/// Recommend the current phase-4 plan from the evaluated architecture options.
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
        return format!(
            "Selected `{}` because {}it satisfies verifier/proof-size budgets with score {}.",
            option.name, transparency_note, assessment.score
        );
    }
    format!(
        "Selected `{}` as least-risk fallback with score {}, but follow-up risk burn-down is required.",
        option.name, assessment.score
    )
}

fn build_phase_milestones(option_name: &str) -> Vec<ZkPhaseMilestone> {
    vec![
        feasibility_milestone(option_name),
        processor_milestone(),
        validator_milestone(),
    ]
}

fn feasibility_milestone(option_name: &str) -> ZkPhaseMilestone {
    ZkPhaseMilestone {
        phase: "Phase 4.0 - Feasibility harness".to_owned(),
        objective: format!(
            "Implement deterministic witness harness for `{option_name}` using canonical envelope payloads."
        ),
        validation_focus:
            "Unit + functional validation for policy scoring and witness commitments.".to_owned(),
        exit_criteria: vec![
            "Witness commitment remains stable across repeated executions.".to_owned(),
            "Policy errors are explicit for invalid boundaries.".to_owned(),
        ],
    }
}

fn processor_milestone() -> ZkPhaseMilestone {
    ZkPhaseMilestone {
        phase: "Phase 4.1 - Processor verification pilot".to_owned(),
        objective:
            "Attach proof verification to processor transaction validation in bounded fast-lane path."
                .to_owned(),
        validation_focus:
            "Integration tests over message lifecycle with proof verification hooks.".to_owned(),
        exit_criteria: vec![
            "Processor rejects unverifiable proofs deterministically.".to_owned(),
            "Verifier runtime remains within policy budget under representative load.".to_owned(),
        ],
    }
}

fn validator_milestone() -> ZkPhaseMilestone {
    ZkPhaseMilestone {
        phase: "Phase 4.2 - Validator and watchdog expansion".to_owned(),
        objective:
            "Extend verification to validator quorum and watchdog sampling for abuse detection."
                .to_owned(),
        validation_focus: "Regression tests for censorship, replay, and invalid-proof propagation."
            .to_owned(),
        exit_criteria: vec![
            "Quorum paths align on proof validity outcomes.".to_owned(),
            "Watchdog alerts isolate invalid-proof mismatches without false positives.".to_owned(),
        ],
    }
}
