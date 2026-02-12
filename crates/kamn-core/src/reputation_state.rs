//! Reputation state persistence, mutation, and restore contracts.

use crate::{canonical_state_key, AgentDid, StateKeyError, StateVersion, APP_STATE_VERSION};
use std::collections::HashMap;
use std::fmt;

/// Default trust score assigned to newly registered agents.
pub const DEFAULT_TRUST_SCORE: u32 = 500;
/// Maximum allowed trust score.
pub const MAX_TRUST_SCORE: u32 = 1_000;

/// Task outcome categories used for reputation accounting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReputationTaskOutcome {
    /// Task completed successfully.
    Completed,
    /// Task failed.
    Failed,
    /// Task delegated to another agent.
    Delegated,
}

/// Endorsement evidence attached to an agent reputation profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Endorsement {
    /// Endorsement identifier.
    pub endorsement_id: String,
    /// DID of endorsing agent.
    pub from_agent_did: String,
    /// Human-readable endorsement note.
    pub note: String,
    /// Block height where endorsement was anchored.
    pub block_height: u64,
}

/// Dispute record attached to an agent reputation profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisputeRecord {
    /// Dispute identifier.
    pub dispute_id: String,
    /// DID of dispute opener.
    pub opened_by: String,
    /// Dispute reason text.
    pub reason: String,
    /// Block height where dispute was opened.
    pub block_height: u64,
}

/// Capability verification evidence entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityVerification {
    /// Capability identifier.
    pub capability: String,
    /// DID of verifier.
    pub verifier_did: String,
    /// Proof reference for verification evidence.
    pub proof_ref: String,
    /// Block height where verification was recorded.
    pub block_height: u64,
}

/// Trust-score snapshot at a specific block height.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreSnapshot {
    /// Trust score value.
    pub trust_score: u32,
    /// Block height of snapshot.
    pub block_height: u64,
}

/// Canonical in-memory reputation profile for an agent.
#[derive(Debug, Clone, PartialEq)]
pub struct AgentReputation {
    /// Agent DID.
    pub agent_did: String,
    /// Current trust score.
    pub trust_score: u32,
    /// Delivery success rate for completed/failed tasks.
    pub delivery_rate: f64,
    /// Average response time across sampled outcomes.
    pub response_time_avg_ms: u64,
    /// Ratio of disputes to completed/failed tasks.
    pub dispute_rate: f64,
    /// Total completed tasks.
    pub tasks_completed: u64,
    /// Total failed tasks.
    pub tasks_failed: u64,
    /// Total delegated tasks.
    pub tasks_delegated: u64,
    /// Total earned value.
    pub total_earned: u64,
    /// Total spent value.
    pub total_spent: u64,
    /// Endorsement history.
    pub endorsements: Vec<Endorsement>,
    /// Dispute history.
    pub disputes: Vec<DisputeRecord>,
    /// Verified capability entries.
    pub verified_capabilities: Vec<CapabilityVerification>,
    /// Last block height that mutated this profile.
    pub last_updated_block: u64,
    /// Trust-score history snapshots.
    pub score_history: Vec<ScoreSnapshot>,
}

/// Persisted reputation record with canonical state-key metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct ReputationPersistedRecord {
    /// Canonical state key.
    pub state_key: String,
    /// State schema version.
    pub state_version: StateVersion,
    /// Agent reputation payload.
    pub reputation: AgentReputation,
}

/// In-memory store for agent reputation profiles.
#[derive(Debug, Clone, PartialEq)]
pub struct ReputationStore {
    state_version: StateVersion,
    agents: HashMap<String, AgentReputation>,
}

impl Default for ReputationStore {
    fn default() -> Self {
        Self {
            state_version: APP_STATE_VERSION,
            agents: HashMap::new(),
        }
    }
}

/// Error taxonomy for reputation-store validation and persistence flows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReputationError {
    /// Agent DID is invalid.
    InvalidAgentDid(String),
    /// Agent already exists in store.
    AgentAlreadyExists(String),
    /// Agent was not found in store.
    AgentNotFound(String),
    /// Required field is empty.
    EmptyField(&'static str),
    /// Block height is invalid.
    InvalidBlockHeight,
    /// Response-time sample is required for this outcome.
    MissingResponseTime {
        /// Outcome requiring response-time sample.
        outcome: ReputationTaskOutcome,
    },
    /// Endorsement id already exists for this agent.
    DuplicateEndorsementId(String),
    /// Dispute id already exists for this agent.
    DuplicateDisputeId(String),
    /// Capability verification duplicate detected.
    DuplicateCapabilityVerification {
        /// Capability value.
        capability: String,
        /// Verifier DID.
        verifier_did: String,
    },
    /// Trust score is out of allowed bounds.
    ScoreOutOfRange(u32),
    /// Canonical state-key generation/validation error.
    StateKey(StateKeyError),
    /// Persisted state key does not match canonical key.
    StateKeyMismatch {
        /// Expected canonical state key.
        expected: String,
        /// Observed persisted state key.
        actual: String,
    },
    /// Duplicate persisted state key encountered during restore.
    DuplicateStateKey(String),
    /// Persisted schema version mismatch.
    VersionMismatch {
        /// Expected state version.
        expected: StateVersion,
        /// Found state version.
        found: StateVersion,
    },
}

impl fmt::Display for ReputationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAgentDid(message) => write!(f, "invalid agent did: {message}"),
            Self::AgentAlreadyExists(agent_did) => {
                write!(f, "agent already exists: {agent_did}")
            }
            Self::AgentNotFound(agent_did) => write!(f, "agent not found: {agent_did}"),
            Self::EmptyField(field) => write!(f, "{field} must not be empty"),
            Self::InvalidBlockHeight => write!(f, "block_height must be greater than zero"),
            Self::MissingResponseTime { outcome } => {
                write!(f, "response_time_ms is required for outcome {outcome:?}")
            }
            Self::DuplicateEndorsementId(endorsement_id) => {
                write!(f, "duplicate endorsement_id: {endorsement_id}")
            }
            Self::DuplicateDisputeId(dispute_id) => {
                write!(f, "duplicate dispute_id: {dispute_id}")
            }
            Self::DuplicateCapabilityVerification {
                capability,
                verifier_did,
            } => write!(
                f,
                "duplicate capability verification for capability `{capability}` by `{verifier_did}`"
            ),
            Self::ScoreOutOfRange(score) => {
                write!(f, "trust_score must be between 0 and {MAX_TRUST_SCORE}, found {score}")
            }
            Self::StateKey(error) => write!(f, "state key error: {error}"),
            Self::StateKeyMismatch { expected, actual } => write!(
                f,
                "state key mismatch, expected `{expected}` but found `{actual}`"
            ),
            Self::DuplicateStateKey(state_key) => write!(f, "duplicate state key: {state_key}"),
            Self::VersionMismatch { expected, found } => {
                write!(
                    f,
                    "state version mismatch, expected {}, found {}",
                    expected.0, found.0
                )
            }
        }
    }
}

impl std::error::Error for ReputationError {}

impl ReputationStore {
    /// Returns the store schema version.
    pub fn state_version(&self) -> StateVersion {
        self.state_version
    }

    /// Returns an immutable view of an agent profile.
    pub fn get_agent(&self, agent_did: &str) -> Option<&AgentReputation> {
        self.agents.get(agent_did)
    }

    /// Registers a new agent with default reputation values.
    pub fn register_agent(
        &mut self,
        agent_did: &str,
        block_height: u64,
    ) -> Result<(), ReputationError> {
        validate_block_height(block_height)?;
        validate_agent_did(agent_did)?;

        if self.agents.contains_key(agent_did) {
            return Err(ReputationError::AgentAlreadyExists(agent_did.to_owned()));
        }

        self.agents.insert(
            agent_did.to_owned(),
            AgentReputation {
                agent_did: agent_did.to_owned(),
                trust_score: DEFAULT_TRUST_SCORE,
                delivery_rate: 0.0,
                response_time_avg_ms: 0,
                dispute_rate: 0.0,
                tasks_completed: 0,
                tasks_failed: 0,
                tasks_delegated: 0,
                total_earned: 0,
                total_spent: 0,
                endorsements: Vec::new(),
                disputes: Vec::new(),
                verified_capabilities: Vec::new(),
                last_updated_block: block_height,
                score_history: vec![ScoreSnapshot {
                    trust_score: DEFAULT_TRUST_SCORE,
                    block_height,
                }],
            },
        );
        Ok(())
    }

    /// Records a task outcome and updates derived reputation metrics.
    pub fn record_task_outcome(
        &mut self,
        agent_did: &str,
        outcome: ReputationTaskOutcome,
        response_time_ms: Option<u64>,
        earned_delta: u64,
        spent_delta: u64,
        block_height: u64,
    ) -> Result<(), ReputationError> {
        validate_block_height(block_height)?;
        let agent = self.agent_mut(agent_did)?;
        let prior_samples = agent.tasks_completed.saturating_add(agent.tasks_failed);

        match outcome {
            ReputationTaskOutcome::Completed => {
                agent.tasks_completed = agent.tasks_completed.saturating_add(1);
            }
            ReputationTaskOutcome::Failed => {
                agent.tasks_failed = agent.tasks_failed.saturating_add(1);
            }
            ReputationTaskOutcome::Delegated => {
                agent.tasks_delegated = agent.tasks_delegated.saturating_add(1);
            }
        }

        if matches!(
            outcome,
            ReputationTaskOutcome::Completed | ReputationTaskOutcome::Failed
        ) {
            let response =
                response_time_ms.ok_or(ReputationError::MissingResponseTime { outcome })?;
            let weighted_total = u128::from(agent.response_time_avg_ms) * u128::from(prior_samples)
                + u128::from(response);
            let sample_count = prior_samples.saturating_add(1);
            agent.response_time_avg_ms = (weighted_total / u128::from(sample_count)) as u64;
        }

        agent.total_earned = agent.total_earned.saturating_add(earned_delta);
        agent.total_spent = agent.total_spent.saturating_add(spent_delta);

        let completed_and_failed = agent.tasks_completed.saturating_add(agent.tasks_failed);
        agent.delivery_rate = if completed_and_failed == 0 {
            0.0
        } else {
            agent.tasks_completed as f64 / completed_and_failed as f64
        };

        agent.dispute_rate =
            calculate_dispute_rate(agent.disputes.len() as u64, completed_and_failed);
        agent.last_updated_block = block_height;
        Ok(())
    }

    /// Records endorsement evidence for an agent.
    pub fn record_endorsement(
        &mut self,
        agent_did: &str,
        endorsement: Endorsement,
    ) -> Result<(), ReputationError> {
        validate_block_height(endorsement.block_height)?;
        validate_agent_did(&endorsement.from_agent_did)?;
        require_non_empty("endorsement.endorsement_id", &endorsement.endorsement_id)?;
        require_non_empty("endorsement.note", &endorsement.note)?;

        let agent = self.agent_mut(agent_did)?;
        if agent
            .endorsements
            .iter()
            .any(|entry| entry.endorsement_id == endorsement.endorsement_id)
        {
            return Err(ReputationError::DuplicateEndorsementId(
                endorsement.endorsement_id,
            ));
        }
        agent.last_updated_block = endorsement.block_height;
        agent.endorsements.push(endorsement);
        Ok(())
    }

    /// Records a dispute for an agent and refreshes dispute rate.
    pub fn record_dispute(
        &mut self,
        agent_did: &str,
        dispute: DisputeRecord,
    ) -> Result<(), ReputationError> {
        validate_block_height(dispute.block_height)?;
        validate_agent_did(&dispute.opened_by)?;
        require_non_empty("dispute.dispute_id", &dispute.dispute_id)?;
        require_non_empty("dispute.reason", &dispute.reason)?;

        let agent = self.agent_mut(agent_did)?;
        if agent
            .disputes
            .iter()
            .any(|entry| entry.dispute_id == dispute.dispute_id)
        {
            return Err(ReputationError::DuplicateDisputeId(dispute.dispute_id));
        }

        agent.disputes.push(dispute);
        let completed_and_failed = agent.tasks_completed.saturating_add(agent.tasks_failed);
        agent.dispute_rate =
            calculate_dispute_rate(agent.disputes.len() as u64, completed_and_failed);
        agent.last_updated_block = agent
            .disputes
            .last()
            .map(|entry| entry.block_height)
            .unwrap_or(agent.last_updated_block);
        Ok(())
    }

    /// Records capability verification evidence for an agent.
    pub fn record_capability_verification(
        &mut self,
        agent_did: &str,
        verification: CapabilityVerification,
    ) -> Result<(), ReputationError> {
        validate_block_height(verification.block_height)?;
        validate_agent_did(&verification.verifier_did)?;
        require_non_empty("verification.capability", &verification.capability)?;
        require_non_empty("verification.proof_ref", &verification.proof_ref)?;

        let agent = self.agent_mut(agent_did)?;
        if agent.verified_capabilities.iter().any(|entry| {
            entry.capability == verification.capability
                && entry.verifier_did == verification.verifier_did
        }) {
            return Err(ReputationError::DuplicateCapabilityVerification {
                capability: verification.capability,
                verifier_did: verification.verifier_did,
            });
        }
        agent.last_updated_block = verification.block_height;
        agent.verified_capabilities.push(verification);
        Ok(())
    }

    /// Sets trust score and appends a score-history snapshot.
    pub fn set_trust_score(
        &mut self,
        agent_did: &str,
        trust_score: u32,
        block_height: u64,
    ) -> Result<(), ReputationError> {
        validate_block_height(block_height)?;
        if trust_score > MAX_TRUST_SCORE {
            return Err(ReputationError::ScoreOutOfRange(trust_score));
        }

        let agent = self.agent_mut(agent_did)?;
        agent.trust_score = trust_score;
        agent.last_updated_block = block_height;
        agent.score_history.push(ScoreSnapshot {
            trust_score,
            block_height,
        });
        Ok(())
    }

    /// Exports canonical persisted records sorted by state key.
    pub fn export_records(&self) -> Vec<ReputationPersistedRecord> {
        let mut records = self
            .agents
            .values()
            .filter_map(|agent| {
                agent_state_key(&agent.agent_did)
                    .ok()
                    .map(|state_key| ReputationPersistedRecord {
                        state_key,
                        state_version: self.state_version,
                        reputation: agent.clone(),
                    })
            })
            .collect::<Vec<_>>();
        records.sort_by(|left, right| left.state_key.cmp(&right.state_key));
        records
    }

    /// Restores a store from persisted records with strict validation.
    pub fn restore_from_records(
        records: &[ReputationPersistedRecord],
    ) -> Result<Self, ReputationError> {
        let mut store = ReputationStore::default();
        let mut seen_keys = HashMap::new();

        for record in records {
            if record.state_version != APP_STATE_VERSION {
                return Err(ReputationError::VersionMismatch {
                    expected: APP_STATE_VERSION,
                    found: record.state_version,
                });
            }

            let expected_key = agent_state_key(&record.reputation.agent_did)?;
            if record.state_key != expected_key {
                return Err(ReputationError::StateKeyMismatch {
                    expected: expected_key,
                    actual: record.state_key.clone(),
                });
            }

            if seen_keys.insert(record.state_key.clone(), ()).is_some() {
                return Err(ReputationError::DuplicateStateKey(record.state_key.clone()));
            }

            if store
                .agents
                .insert(
                    record.reputation.agent_did.clone(),
                    record.reputation.clone(),
                )
                .is_some()
            {
                return Err(ReputationError::AgentAlreadyExists(
                    record.reputation.agent_did.clone(),
                ));
            }
        }

        Ok(store)
    }

    fn agent_mut(&mut self, agent_did: &str) -> Result<&mut AgentReputation, ReputationError> {
        self.agents
            .get_mut(agent_did)
            .ok_or_else(|| ReputationError::AgentNotFound(agent_did.to_owned()))
    }
}

/// Builds the canonical reputation state key for an agent DID.
pub fn agent_state_key(agent_did: &str) -> Result<String, ReputationError> {
    let did = AgentDid::parse(agent_did)
        .map_err(|error| ReputationError::InvalidAgentDid(error.to_string()))?;
    canonical_state_key("kamn.reputation.scores", "agent", did.method_specific_id())
        .map_err(ReputationError::StateKey)
}

fn calculate_dispute_rate(disputes: u64, completed_and_failed: u64) -> f64 {
    if completed_and_failed == 0 {
        if disputes == 0 {
            0.0
        } else {
            1.0
        }
    } else {
        disputes as f64 / completed_and_failed as f64
    }
}

fn validate_agent_did(agent_did: &str) -> Result<(), ReputationError> {
    AgentDid::parse(agent_did)
        .map_err(|error| ReputationError::InvalidAgentDid(error.to_string()))?;
    Ok(())
}

fn validate_block_height(block_height: u64) -> Result<(), ReputationError> {
    if block_height == 0 {
        return Err(ReputationError::InvalidBlockHeight);
    }
    Ok(())
}

fn require_non_empty(field: &'static str, value: &str) -> Result<(), ReputationError> {
    if value.trim().is_empty() {
        return Err(ReputationError::EmptyField(field));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{agent_state_key, ReputationError, ReputationStore};

    #[test]
    fn state_key_uses_reputation_namespace() {
        let key = agent_state_key("kamn:did:agent:agent-1").expect("state key should build");
        assert_eq!(key, "kamn.reputation.scores:agent:agent-1");
    }

    #[test]
    fn export_skips_invalid_agents() {
        let mut store = ReputationStore::default();
        store
            .register_agent("kamn:did:agent:agent-1", 1)
            .expect("registration should succeed");
        assert_eq!(store.export_records().len(), 1);
    }

    #[test]
    fn restore_rejects_version_mismatch() {
        let mut store = ReputationStore::default();
        store
            .register_agent("kamn:did:agent:agent-1", 1)
            .expect("registration should succeed");
        let mut records = store.export_records();
        records[0].state_version = super::StateVersion(99);

        let result = ReputationStore::restore_from_records(&records);
        assert_eq!(
            result,
            Err(ReputationError::VersionMismatch {
                expected: super::APP_STATE_VERSION,
                found: super::StateVersion(99),
            })
        );
    }
}
