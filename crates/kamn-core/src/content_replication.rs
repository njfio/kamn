use crate::{ContentStorageAdapter, ContentStorageError};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentReplicationPolicy {
    pub minimum_replicas: u16,
    pub target_replicas: u16,
    pub max_repair_attempts: u8,
}

impl ContentReplicationPolicy {
    pub fn new(
        minimum_replicas: u16,
        target_replicas: u16,
        max_repair_attempts: u8,
    ) -> Result<Self, ContentReplicationError> {
        if minimum_replicas == 0 {
            return Err(ContentReplicationError::InvalidPolicy("minimum_replicas"));
        }
        if target_replicas < minimum_replicas {
            return Err(ContentReplicationError::InvalidPolicy(
                "target_replicas must be >= minimum_replicas",
            ));
        }
        if max_repair_attempts == 0 {
            return Err(ContentReplicationError::InvalidPolicy(
                "max_repair_attempts",
            ));
        }

        Ok(Self {
            minimum_replicas,
            target_replicas,
            max_repair_attempts,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentAvailabilityHealth {
    Healthy,
    Degraded,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentAvailabilitySnapshot {
    pub cid: String,
    pub health: ContentAvailabilityHealth,
    pub available_replicas: u16,
    pub minimum_replicas: u16,
    pub target_replicas: u16,
    pub repair_attempts: u8,
    pub last_checked_unix: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentAvailabilityAlert {
    pub cid: String,
    pub health: ContentAvailabilityHealth,
    pub available_replicas: u16,
    pub minimum_replicas: u16,
    pub target_replicas: u16,
    pub repair_attempts: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentRepairReason {
    UnderReplicated,
    MissingContent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentRepairAction {
    pub cid: String,
    pub missing_replicas: u16,
    pub reason: ContentRepairReason,
    pub attempt: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentReplicationManager {
    policy: ContentReplicationPolicy,
    records: BTreeMap<String, ContentReplicationRecord>,
}

impl ContentReplicationManager {
    pub fn new(policy: ContentReplicationPolicy) -> Self {
        Self {
            policy,
            records: BTreeMap::new(),
        }
    }

    pub fn register_content<A: ContentStorageAdapter>(
        &mut self,
        adapter: &A,
        cid: &str,
        replica_nodes: &[&str],
        checked_at_unix: u64,
    ) -> Result<ContentAvailabilitySnapshot, ContentReplicationError> {
        if checked_at_unix == 0 {
            return Err(ContentReplicationError::EmptyField("checked_at_unix"));
        }

        adapter
            .head(cid)
            .map_err(ContentReplicationError::Storage)?;
        adapter
            .verify(cid)
            .map_err(ContentReplicationError::Storage)?;

        let replicas = normalize_replica_nodes(replica_nodes)?;
        let previous = self.records.get(cid).cloned();
        let mut record = previous.unwrap_or_default();
        record.replicas = replicas;
        record.last_checked_unix = checked_at_unix;
        record.pending_repair = None;

        self.records.insert(cid.to_owned(), record);
        self.availability(cid)
    }

    pub fn availability(
        &self,
        cid: &str,
    ) -> Result<ContentAvailabilitySnapshot, ContentReplicationError> {
        let record = self
            .records
            .get(cid)
            .ok_or_else(|| ContentReplicationError::UnknownContent(cid.to_owned()))?;
        Ok(build_snapshot(cid, record, &self.policy))
    }

    pub fn availability_alerts(&self) -> Vec<ContentAvailabilityAlert> {
        self.records
            .iter()
            .filter_map(|(cid, record)| {
                let available_replicas = record.replicas.len() as u16;
                let health = health_for_replicas(self.policy.minimum_replicas, available_replicas);
                match health {
                    ContentAvailabilityHealth::Healthy => None,
                    _ => Some(ContentAvailabilityAlert {
                        cid: cid.clone(),
                        health,
                        available_replicas,
                        minimum_replicas: self.policy.minimum_replicas,
                        target_replicas: self.policy.target_replicas,
                        repair_attempts: record.repair_attempts,
                    }),
                }
            })
            .collect()
    }

    pub fn plan_repairs(&mut self) -> Vec<ContentRepairAction> {
        let mut actions = Vec::new();

        for (cid, record) in &mut self.records {
            let available_replicas = record.replicas.len() as u16;
            if available_replicas >= self.policy.target_replicas {
                record.pending_repair = None;
                continue;
            }
            if record.repair_attempts >= self.policy.max_repair_attempts {
                record.pending_repair = None;
                continue;
            }
            if record.pending_repair.is_some() {
                continue;
            }

            let reason = if available_replicas == 0 {
                ContentRepairReason::MissingContent
            } else {
                ContentRepairReason::UnderReplicated
            };
            let action = ContentRepairAction {
                cid: cid.clone(),
                missing_replicas: self
                    .policy
                    .target_replicas
                    .saturating_sub(available_replicas),
                reason: reason.clone(),
                attempt: record.repair_attempts + 1,
            };
            record.pending_repair = Some(reason);
            actions.push(action);
        }

        actions
    }

    pub fn apply_repair_success(
        &mut self,
        cid: &str,
        replica_node: &str,
        checked_at_unix: u64,
    ) -> Result<ContentAvailabilitySnapshot, ContentReplicationError> {
        if replica_node.trim().is_empty() {
            return Err(ContentReplicationError::EmptyField("replica_node"));
        }
        if checked_at_unix == 0 {
            return Err(ContentReplicationError::EmptyField("checked_at_unix"));
        }

        let record = self
            .records
            .get_mut(cid)
            .ok_or_else(|| ContentReplicationError::UnknownContent(cid.to_owned()))?;
        record.replicas.insert(replica_node.to_owned());
        record.last_checked_unix = checked_at_unix;
        record.repair_attempts = 0;
        record.pending_repair = None;
        Ok(build_snapshot(cid, record, &self.policy))
    }

    pub fn apply_repair_failure(
        &mut self,
        cid: &str,
        checked_at_unix: u64,
    ) -> Result<(), ContentReplicationError> {
        if checked_at_unix == 0 {
            return Err(ContentReplicationError::EmptyField("checked_at_unix"));
        }

        let record = self
            .records
            .get_mut(cid)
            .ok_or_else(|| ContentReplicationError::UnknownContent(cid.to_owned()))?;

        if record.repair_attempts >= self.policy.max_repair_attempts {
            return Err(ContentReplicationError::RepairAttemptsExceeded {
                cid: cid.to_owned(),
                max_attempts: self.policy.max_repair_attempts,
            });
        }

        record.repair_attempts += 1;
        record.last_checked_unix = checked_at_unix;
        record.pending_repair = None;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContentReplicationError {
    InvalidPolicy(&'static str),
    EmptyField(&'static str),
    UnknownContent(String),
    RepairAttemptsExceeded { cid: String, max_attempts: u8 },
    Storage(ContentStorageError),
}

impl fmt::Display for ContentReplicationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPolicy(field) => write!(f, "invalid replication policy: {field}"),
            Self::EmptyField(field) => write!(f, "field must not be empty: {field}"),
            Self::UnknownContent(cid) => write!(f, "content not tracked for replication: {cid}"),
            Self::RepairAttemptsExceeded { cid, max_attempts } => write!(
                f,
                "repair attempts exceeded for {cid}; maximum configured attempts: {max_attempts}"
            ),
            Self::Storage(error) => write!(f, "storage validation failed: {error}"),
        }
    }
}

impl std::error::Error for ContentReplicationError {}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct ContentReplicationRecord {
    replicas: BTreeSet<String>,
    last_checked_unix: u64,
    repair_attempts: u8,
    pending_repair: Option<ContentRepairReason>,
}

fn normalize_replica_nodes(
    replica_nodes: &[&str],
) -> Result<BTreeSet<String>, ContentReplicationError> {
    let mut replicas = BTreeSet::new();
    for node in replica_nodes {
        if node.trim().is_empty() {
            return Err(ContentReplicationError::EmptyField("replica_node"));
        }
        replicas.insert((*node).to_owned());
    }
    Ok(replicas)
}

fn health_for_replicas(
    minimum_replicas: u16,
    available_replicas: u16,
) -> ContentAvailabilityHealth {
    if available_replicas == 0 {
        return ContentAvailabilityHealth::Unavailable;
    }
    if available_replicas < minimum_replicas {
        return ContentAvailabilityHealth::Degraded;
    }
    ContentAvailabilityHealth::Healthy
}

fn build_snapshot(
    cid: &str,
    record: &ContentReplicationRecord,
    policy: &ContentReplicationPolicy,
) -> ContentAvailabilitySnapshot {
    let available_replicas = record.replicas.len() as u16;
    ContentAvailabilitySnapshot {
        cid: cid.to_owned(),
        health: health_for_replicas(policy.minimum_replicas, available_replicas),
        available_replicas,
        minimum_replicas: policy.minimum_replicas,
        target_replicas: policy.target_replicas,
        repair_attempts: record.repair_attempts,
        last_checked_unix: record.last_checked_unix,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        health_for_replicas, normalize_replica_nodes, ContentAvailabilityHealth,
        ContentReplicationError, ContentReplicationPolicy,
    };

    #[test]
    fn policy_rejects_target_below_minimum() {
        assert_eq!(
            ContentReplicationPolicy::new(2, 1, 1),
            Err(ContentReplicationError::InvalidPolicy(
                "target_replicas must be >= minimum_replicas"
            ))
        );
    }

    #[test]
    fn normalize_replica_nodes_rejects_empty_node_id() {
        assert_eq!(
            normalize_replica_nodes(&["node-a", ""]),
            Err(ContentReplicationError::EmptyField("replica_node"))
        );
    }

    #[test]
    fn health_classification_marks_unavailable_when_zero_replicas() {
        assert_eq!(
            health_for_replicas(2, 0),
            ContentAvailabilityHealth::Unavailable
        );
    }
}
