use crate::{AgentReputation, ReputationError, ReputationStore};
use std::fmt;

pub const TRUST_SCORE_ENGINE_VERSION: &str = "v2-prd-8-2-anti-gaming";
pub const TRUST_SCORE_MIN: i32 = 0;
pub const TRUST_SCORE_MAX: i32 = 1_000;
const DECAY_WINDOW_RECENT_BLOCKS: u64 = 128;
const DECAY_WINDOW_MID_BLOCKS: u64 = 512;
const DECAY_MIN_BPS: i32 = 500;
const DECAY_MAX_BPS: i32 = 1_000;
const ABUSE_PENALTY_RECIPROCITY_RING: i32 = 80;
const ABUSE_PENALTY_BURST_SPAM: i32 = 70;
const ABUSE_PENALTY_CHURN_SPIKE: i32 = 60;
const ABUSE_PENALTY_COMPOUND: i32 = 140;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbusePenaltyKind {
    None,
    ReciprocityRing,
    BurstSpam,
    ChurnSpike,
    Compound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrustScoreBreakdown {
    pub base_score: i32,
    pub delivery_component: i32,
    pub response_component: i32,
    pub dispute_penalty: i32,
    pub volume_bonus: i32,
    pub endorsement_bonus: i32,
    pub decay_multiplier_bps: u16,
    pub decayed_volume_bonus: i32,
    pub decayed_endorsement_bonus: i32,
    pub abuse_penalty_kind: AbusePenaltyKind,
    pub abuse_penalty_points: i32,
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
    let decay_multiplier_bps = calculate_decay_multiplier_bps(agent);
    let decayed_volume_bonus = apply_decay_multiplier(volume_bonus, decay_multiplier_bps);
    let decayed_endorsement_bonus = apply_decay_multiplier(endorsement_bonus, decay_multiplier_bps);
    let (abuse_penalty_kind, abuse_penalty_points) = classify_abuse_penalty(agent);

    let raw_score = base_score + delivery_component + response_component - dispute_penalty
        + decayed_volume_bonus
        + decayed_endorsement_bonus
        - abuse_penalty_points;
    let final_score = raw_score.clamp(TRUST_SCORE_MIN, TRUST_SCORE_MAX) as u32;

    Ok(TrustScoreBreakdown {
        base_score,
        delivery_component,
        response_component,
        dispute_penalty,
        volume_bonus,
        endorsement_bonus,
        decay_multiplier_bps,
        decayed_volume_bonus,
        decayed_endorsement_bonus,
        abuse_penalty_kind,
        abuse_penalty_points,
        raw_score,
        final_score,
    })
}

fn calculate_decay_multiplier_bps(agent: &AgentReputation) -> u16 {
    let mut multiplier_bps = if agent.score_history.is_empty() {
        950
    } else {
        let mut recent = 0usize;
        let mut mid = 0usize;
        let mut stale = 0usize;
        for snapshot in &agent.score_history {
            let age = agent
                .last_updated_block
                .saturating_sub(snapshot.block_height);
            if age <= DECAY_WINDOW_RECENT_BLOCKS {
                recent += 1;
            } else if age <= DECAY_WINDOW_MID_BLOCKS {
                mid += 1;
            } else {
                stale += 1;
            }
        }

        550 + (recent.min(3) as i32 * 120) + (mid.min(4) as i32 * 40) + (stale.min(8) as i32 * 10)
    };

    let total_activity = agent.tasks_completed + agent.tasks_failed + agent.tasks_delegated;
    if total_activity < 25 {
        multiplier_bps -= 50;
    }
    if agent.dispute_rate > 0.2 {
        multiplier_bps -= 80;
    }

    multiplier_bps.clamp(DECAY_MIN_BPS, DECAY_MAX_BPS) as u16
}

fn apply_decay_multiplier(points: i32, multiplier_bps: u16) -> i32 {
    points.saturating_mul(multiplier_bps as i32) / 1_000
}

fn classify_abuse_penalty(agent: &AgentReputation) -> (AbusePenaltyKind, i32) {
    let completed = agent.tasks_completed as f64;
    let delegated = agent.tasks_delegated as f64;
    let failed = agent.tasks_failed as f64;
    let disputes = agent.disputes.len() as f64;

    let delegation_ratio = delegated / completed.max(1.0);
    let failure_ratio = failed / (completed + failed).max(1.0);
    let churn_ratio = disputes / (completed + failed).max(1.0);

    let reciprocity_ring = delegation_ratio >= 0.60;
    let burst_spam = failure_ratio >= 0.45 && agent.tasks_failed >= 10;
    let churn_spike = churn_ratio >= 0.20 && agent.disputes.len() >= 5;

    let triggered = [reciprocity_ring, burst_spam, churn_spike]
        .iter()
        .filter(|value| **value)
        .count();

    if triggered > 1 {
        (AbusePenaltyKind::Compound, ABUSE_PENALTY_COMPOUND)
    } else if reciprocity_ring {
        (
            AbusePenaltyKind::ReciprocityRing,
            ABUSE_PENALTY_RECIPROCITY_RING,
        )
    } else if burst_spam {
        (AbusePenaltyKind::BurstSpam, ABUSE_PENALTY_BURST_SPAM)
    } else if churn_spike {
        (AbusePenaltyKind::ChurnSpike, ABUSE_PENALTY_CHURN_SPIKE)
    } else {
        (AbusePenaltyKind::None, 0)
    }
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
    use super::{
        calculate_trust_score, AbusePenaltyKind, TrustScoreError, ABUSE_PENALTY_COMPOUND,
        TRUST_SCORE_ENGINE_VERSION,
    };
    use crate::{AgentReputation, DisputeRecord, ScoreSnapshot};

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
        assert_eq!(TRUST_SCORE_ENGINE_VERSION, "v2-prd-8-2-anti-gaming");
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

    #[test]
    fn weighted_decay_prefers_recent_score_history() {
        let mut recent = baseline();
        recent.last_updated_block = 1_000;
        recent.tasks_completed = 200;
        recent.endorsements = vec![
            crate::Endorsement {
                endorsement_id: "endorse-r1".to_owned(),
                from_agent_did: "kamn:did:agent:r1".to_owned(),
                note: "recent".to_owned(),
                block_height: 990,
            },
            crate::Endorsement {
                endorsement_id: "endorse-r2".to_owned(),
                from_agent_did: "kamn:did:agent:r2".to_owned(),
                note: "recent".to_owned(),
                block_height: 995,
            },
        ];
        recent.score_history = vec![
            ScoreSnapshot {
                trust_score: 500,
                block_height: 995,
            },
            ScoreSnapshot {
                trust_score: 500,
                block_height: 980,
            },
            ScoreSnapshot {
                trust_score: 500,
                block_height: 970,
            },
        ];

        let mut stale = recent.clone();
        stale.score_history = vec![
            ScoreSnapshot {
                trust_score: 500,
                block_height: 100,
            },
            ScoreSnapshot {
                trust_score: 500,
                block_height: 90,
            },
            ScoreSnapshot {
                trust_score: 500,
                block_height: 80,
            },
        ];

        let recent_breakdown =
            calculate_trust_score(&recent).expect("recent score should calculate");
        let stale_breakdown = calculate_trust_score(&stale).expect("stale score should calculate");

        assert!(recent_breakdown.decay_multiplier_bps > stale_breakdown.decay_multiplier_bps);
        assert!(
            recent_breakdown.decayed_endorsement_bonus >= stale_breakdown.decayed_endorsement_bonus
        );
    }

    #[test]
    fn abuse_thresholds_map_to_compound_penalty() {
        let mut rep = baseline();
        rep.tasks_completed = 10;
        rep.tasks_failed = 12;
        rep.tasks_delegated = 8;
        rep.disputes = (0..5)
            .map(|index| DisputeRecord {
                dispute_id: format!("dispute-{index}"),
                opened_by: "kamn:did:agent:requester".to_owned(),
                reason: "abuse".to_owned(),
                block_height: 100 + index as u64,
            })
            .collect();

        let breakdown = calculate_trust_score(&rep).expect("score should calculate");
        assert_eq!(breakdown.abuse_penalty_kind, AbusePenaltyKind::Compound);
        assert_eq!(breakdown.abuse_penalty_points, ABUSE_PENALTY_COMPOUND);
    }
}
