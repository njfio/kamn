use crate::{AgentReputation, ReputationError, ReputationStore};
use std::fmt;

pub const TRUST_SCORE_ENGINE_VERSION: &str = "v1-prd-8-2";
pub const TRUST_SCORE_MIN: i32 = 0;
pub const TRUST_SCORE_MAX: i32 = 1_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrustScoreBreakdown {
    pub base_score: i32,
    pub delivery_component: i32,
    pub response_component: i32,
    pub dispute_penalty: i32,
    pub volume_bonus: i32,
    pub endorsement_bonus: i32,
    pub raw_score: i32,
    pub final_score: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TrustScoreError {
    InvalidDeliveryRate(f64),
    InvalidDisputeRate(f64),
    Reputation(ReputationError),
}

impl fmt::Display for TrustScoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDeliveryRate(value) => {
                write!(f, "delivery_rate must be within 0.0..=1.0, found {value}")
            }
            Self::InvalidDisputeRate(value) => {
                write!(f, "dispute_rate must be within 0.0..=1.0, found {value}")
            }
            Self::Reputation(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for TrustScoreError {}

impl From<ReputationError> for TrustScoreError {
    fn from(value: ReputationError) -> Self {
        Self::Reputation(value)
    }
}

pub fn calculate_trust_score(
    agent: &AgentReputation,
) -> Result<TrustScoreBreakdown, TrustScoreError> {
    if !(0.0..=1.0).contains(&agent.delivery_rate) {
        return Err(TrustScoreError::InvalidDeliveryRate(agent.delivery_rate));
    }
    if !(0.0..=1.0).contains(&agent.dispute_rate) {
        return Err(TrustScoreError::InvalidDisputeRate(agent.dispute_rate));
    }

    let base_score = 500;
    let delivery_component = ((agent.delivery_rate - 0.5) * 400.0) as i32;
    let response_component = match agent.response_time_avg_ms {
        0..=1_000 => 100,
        1_001..=5_000 => 50,
        5_001..=30_000 => 0,
        _ => -50,
    };
    let dispute_penalty = (agent.dispute_rate * 150.0) as i32;
    let volume_bonus = (agent.tasks_completed.min(1_000) as f64 * 0.1) as i32;
    let endorsement_bonus = agent.endorsements.len().min(50) as i32;

    let raw_score = base_score + delivery_component + response_component - dispute_penalty
        + volume_bonus
        + endorsement_bonus;
    let final_score = raw_score.clamp(TRUST_SCORE_MIN, TRUST_SCORE_MAX) as u32;

    Ok(TrustScoreBreakdown {
        base_score,
        delivery_component,
        response_component,
        dispute_penalty,
        volume_bonus,
        endorsement_bonus,
        raw_score,
        final_score,
    })
}

pub fn recalculate_and_persist_trust_score(
    store: &mut ReputationStore,
    agent_did: &str,
    block_height: u64,
) -> Result<TrustScoreBreakdown, TrustScoreError> {
    let agent = store
        .get_agent(agent_did)
        .cloned()
        .ok_or_else(|| ReputationError::AgentNotFound(agent_did.to_owned()))?;
    let breakdown = calculate_trust_score(&agent)?;
    store.set_trust_score(agent_did, breakdown.final_score, block_height)?;
    Ok(breakdown)
}

#[cfg(test)]
mod tests {
    use super::{calculate_trust_score, TrustScoreError, TRUST_SCORE_ENGINE_VERSION};
    use crate::AgentReputation;

    fn baseline() -> AgentReputation {
        AgentReputation {
            agent_did: "kamn:did:agent:baseline".to_owned(),
            trust_score: 500,
            delivery_rate: 0.5,
            response_time_avg_ms: 5_000,
            dispute_rate: 0.0,
            tasks_completed: 0,
            tasks_failed: 0,
            tasks_delegated: 0,
            total_earned: 0,
            total_spent: 0,
            endorsements: vec![],
            disputes: vec![],
            verified_capabilities: vec![],
            last_updated_block: 1,
            score_history: vec![],
        }
    }

    #[test]
    fn response_bucket_is_inclusive_at_1000() {
        let mut rep = baseline();
        rep.response_time_avg_ms = 1_000;
        let breakdown = calculate_trust_score(&rep).expect("score should calculate");
        assert_eq!(breakdown.response_component, 100);
    }

    #[test]
    fn engine_version_is_stable() {
        assert_eq!(TRUST_SCORE_ENGINE_VERSION, "v1-prd-8-2");
    }

    #[test]
    fn rejects_dispute_rate_above_one() {
        let mut rep = baseline();
        rep.dispute_rate = 1.4;
        assert_eq!(
            calculate_trust_score(&rep),
            Err(TrustScoreError::InvalidDisputeRate(1.4))
        );
    }
}
