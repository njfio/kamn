//! Upgrade proposal orchestration, activation controls, rollback handling, and audit contracts.

use crate::{AgentDid, GovernanceProposalStatus};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

const UPGRADE_ORCHESTRATION_INVALID_PROPOSED_BY_DID_REASON_CODE: &str =
    "upgrade_orchestration_invalid_proposed_by_did";
const UPGRADE_ORCHESTRATION_INVALID_VALIDATOR_DID_REASON_CODE: &str =
    "upgrade_orchestration_invalid_validator_did";
const UPGRADE_ORCHESTRATION_INVALID_ACTIVATED_BY_DID_REASON_CODE: &str =
    "upgrade_orchestration_invalid_activated_by_did";
const UPGRADE_ORCHESTRATION_INVALID_ROLLED_BACK_BY_DID_REASON_CODE: &str =
    "upgrade_orchestration_invalid_rolled_back_by_did";

/// Upgrade orchestration audit event categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpgradeAuditEventKind {
    /// Upgrade proposal created.
    Proposed,
    /// Validator approval recorded.
    Approved,
    /// Governance status synchronized into the local proposal record.
    GovernanceStatusSynced,
    /// Upgrade activation completed.
    Activated,
    /// Rollback completed.
    RolledBack,
}

/// Structured audit event emitted by upgrade orchestration operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpgradeAuditEvent {
    /// Proposal identifier.
    pub proposal_id: String,
    /// Target version associated with the event.
    pub target_version: String,
    /// Actor DID responsible for the event.
    pub actor_did: String,
    /// Event timestamp in Unix seconds.
    pub event_at_unix: u64,
    /// Event kind.
    pub kind: UpgradeAuditEventKind,
    /// Optional supplemental note.
    pub note: Option<String>,
}

/// Audit view snapshot containing current version and event history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionUpgradeAuditView {
    /// Currently active version.
    pub current_version: String,
    /// Recorded audit events in insertion order.
    pub events: Vec<UpgradeAuditEvent>,
}

/// Upgrade proposal lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpgradeProposalState {
    /// Proposal exists and awaits activation.
    Pending,
    /// Proposal was activated.
    Activated,
    /// Proposal was rolled back after activation.
    RolledBack,
}

/// Mutable upgrade proposal record tracked by the orchestrator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpgradeProposalRecord {
    /// Proposal identifier.
    pub proposal_id: String,
    /// Requested target version.
    pub target_version: String,
    /// DID of the proposer.
    pub proposed_by: String,
    /// Proposal timestamp in Unix seconds.
    pub proposed_at_unix: u64,
    /// Required validator approval quorum.
    pub required_quorum: usize,
    /// Validator DIDs that approved the proposal.
    pub approvals: BTreeSet<String>,
    /// Latest governance status for the proposal.
    pub governance_status: GovernanceProposalStatus,
    /// Current proposal lifecycle state.
    pub state: UpgradeProposalState,
    /// Activation timestamp when state is `Activated`.
    pub activated_at_unix: Option<u64>,
}

/// Stateful orchestrator that manages version upgrade proposals and audits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionUpgradeOrchestrator {
    current_version: String,
    proposals: BTreeMap<String, UpgradeProposalRecord>,
    events: Vec<UpgradeAuditEvent>,
}

impl VersionUpgradeOrchestrator {
    /// Creates a new orchestrator for the provided current version.
    pub fn new(current_version: &str) -> Result<Self, UpgradeOrchestrationError> {
        validate_version_format(current_version)?;
        Ok(Self {
            current_version: current_version.to_owned(),
            proposals: BTreeMap::new(),
            events: Vec::new(),
        })
    }

    /// Registers a new upgrade proposal.
    pub fn propose_upgrade(
        &mut self,
        proposal_id: &str,
        target_version: &str,
        proposed_by: &str,
        required_quorum: usize,
        proposed_at_unix: u64,
    ) -> Result<(), UpgradeOrchestrationError> {
        require_non_empty("proposal_id", proposal_id)?;
        validate_did(
            proposed_by,
            "proposed_by",
            UPGRADE_ORCHESTRATION_INVALID_PROPOSED_BY_DID_REASON_CODE,
        )?;
        validate_timestamp("proposed_at_unix", proposed_at_unix)?;
        validate_version_format(target_version)?;
        if required_quorum == 0 {
            return Err(UpgradeOrchestrationError::InvalidRequiredQuorum(0));
        }
        if self.proposals.contains_key(proposal_id) {
            return Err(UpgradeOrchestrationError::DuplicateProposal(
                proposal_id.to_owned(),
            ));
        }
        if !is_version_advance(&self.current_version, target_version)? {
            return Err(UpgradeOrchestrationError::InvalidTargetVersionTransition {
                current_version: self.current_version.clone(),
                target_version: target_version.to_owned(),
            });
        }

        self.proposals.insert(
            proposal_id.to_owned(),
            UpgradeProposalRecord {
                proposal_id: proposal_id.to_owned(),
                target_version: target_version.to_owned(),
                proposed_by: proposed_by.to_owned(),
                proposed_at_unix,
                required_quorum,
                approvals: BTreeSet::new(),
                governance_status: GovernanceProposalStatus::Voting,
                state: UpgradeProposalState::Pending,
                activated_at_unix: None,
            },
        );
        self.events.push(UpgradeAuditEvent {
            proposal_id: proposal_id.to_owned(),
            target_version: target_version.to_owned(),
            actor_did: proposed_by.to_owned(),
            event_at_unix: proposed_at_unix,
            kind: UpgradeAuditEventKind::Proposed,
            note: None,
        });
        Ok(())
    }

    /// Records validator approval for an existing proposal.
    pub fn approve_upgrade(
        &mut self,
        proposal_id: &str,
        validator_did: &str,
        approved_at_unix: u64,
    ) -> Result<(), UpgradeOrchestrationError> {
        validate_did(
            validator_did,
            "validator_did",
            UPGRADE_ORCHESTRATION_INVALID_VALIDATOR_DID_REASON_CODE,
        )?;
        validate_timestamp("approved_at_unix", approved_at_unix)?;
        let proposal = self
            .proposals
            .get_mut(proposal_id)
            .ok_or_else(|| UpgradeOrchestrationError::ProposalNotFound(proposal_id.to_owned()))?;

        if !proposal.approvals.insert(validator_did.to_owned()) {
            return Err(UpgradeOrchestrationError::DuplicateApproval {
                proposal_id: proposal_id.to_owned(),
                validator_did: validator_did.to_owned(),
            });
        }

        self.events.push(UpgradeAuditEvent {
            proposal_id: proposal_id.to_owned(),
            target_version: proposal.target_version.clone(),
            actor_did: validator_did.to_owned(),
            event_at_unix: approved_at_unix,
            kind: UpgradeAuditEventKind::Approved,
            note: Some("validator approval registered".to_owned()),
        });
        Ok(())
    }

    /// Updates governance status for a proposal.
    pub fn mark_governance_status(
        &mut self,
        proposal_id: &str,
        status: GovernanceProposalStatus,
        updated_at_unix: u64,
    ) -> Result<(), UpgradeOrchestrationError> {
        validate_timestamp("updated_at_unix", updated_at_unix)?;
        let proposal = self
            .proposals
            .get_mut(proposal_id)
            .ok_or_else(|| UpgradeOrchestrationError::ProposalNotFound(proposal_id.to_owned()))?;
        proposal.governance_status = status;

        self.events.push(UpgradeAuditEvent {
            proposal_id: proposal_id.to_owned(),
            target_version: proposal.target_version.clone(),
            actor_did: proposal.proposed_by.clone(),
            event_at_unix: updated_at_unix,
            kind: UpgradeAuditEventKind::GovernanceStatusSynced,
            note: Some(format!("governance status updated to {status:?}")),
        });
        Ok(())
    }

    /// Activates a proposal once governance and quorum requirements are met.
    pub fn activate_upgrade(
        &mut self,
        proposal_id: &str,
        activated_by: &str,
        activated_at_unix: u64,
    ) -> Result<(), UpgradeOrchestrationError> {
        validate_did(
            activated_by,
            "activated_by",
            UPGRADE_ORCHESTRATION_INVALID_ACTIVATED_BY_DID_REASON_CODE,
        )?;
        validate_timestamp("activated_at_unix", activated_at_unix)?;

        let proposal = self
            .proposals
            .get_mut(proposal_id)
            .ok_or_else(|| UpgradeOrchestrationError::ProposalNotFound(proposal_id.to_owned()))?;
        if proposal.state == UpgradeProposalState::Activated {
            return Err(UpgradeOrchestrationError::AlreadyActivated(
                proposal_id.to_owned(),
            ));
        }
        if proposal.governance_status != GovernanceProposalStatus::Approved {
            return Err(UpgradeOrchestrationError::GovernanceNotApproved {
                proposal_id: proposal_id.to_owned(),
                status: proposal.governance_status,
            });
        }
        if proposal.approvals.len() < proposal.required_quorum {
            return Err(UpgradeOrchestrationError::InsufficientApprovals {
                required: proposal.required_quorum,
                provided: proposal.approvals.len(),
            });
        }
        if !is_version_advance(&self.current_version, &proposal.target_version)? {
            return Err(UpgradeOrchestrationError::InvalidTargetVersionTransition {
                current_version: self.current_version.clone(),
                target_version: proposal.target_version.clone(),
            });
        }

        self.current_version = proposal.target_version.clone();
        proposal.state = UpgradeProposalState::Activated;
        proposal.activated_at_unix = Some(activated_at_unix);
        self.events.push(UpgradeAuditEvent {
            proposal_id: proposal_id.to_owned(),
            target_version: proposal.target_version.clone(),
            actor_did: activated_by.to_owned(),
            event_at_unix: activated_at_unix,
            kind: UpgradeAuditEventKind::Activated,
            note: None,
        });
        Ok(())
    }

    /// Rolls back an activated proposal to a specified version.
    pub fn rollback_upgrade(
        &mut self,
        proposal_id: &str,
        rollback_version: &str,
        rolled_back_by: &str,
        rolled_back_at_unix: u64,
        reason: &str,
    ) -> Result<(), UpgradeOrchestrationError> {
        validate_did(
            rolled_back_by,
            "rolled_back_by",
            UPGRADE_ORCHESTRATION_INVALID_ROLLED_BACK_BY_DID_REASON_CODE,
        )?;
        validate_timestamp("rolled_back_at_unix", rolled_back_at_unix)?;
        validate_version_format(rollback_version)?;
        require_non_empty("reason", reason)?;

        let proposal = self
            .proposals
            .get_mut(proposal_id)
            .ok_or_else(|| UpgradeOrchestrationError::ProposalNotFound(proposal_id.to_owned()))?;
        if proposal.state != UpgradeProposalState::Activated {
            return Err(UpgradeOrchestrationError::RollbackNotAllowed(
                proposal_id.to_owned(),
            ));
        }

        self.current_version = rollback_version.to_owned();
        proposal.state = UpgradeProposalState::RolledBack;
        self.events.push(UpgradeAuditEvent {
            proposal_id: proposal_id.to_owned(),
            target_version: rollback_version.to_owned(),
            actor_did: rolled_back_by.to_owned(),
            event_at_unix: rolled_back_at_unix,
            kind: UpgradeAuditEventKind::RolledBack,
            note: Some(reason.to_owned()),
        });
        Ok(())
    }

    /// Returns a cloned proposal record by identifier.
    pub fn proposal(&self, proposal_id: &str) -> Option<UpgradeProposalRecord> {
        self.proposals.get(proposal_id).cloned()
    }

    /// Returns the current audit view snapshot.
    pub fn audit_view(&self) -> VersionUpgradeAuditView {
        VersionUpgradeAuditView {
            current_version: self.current_version.clone(),
            events: self.events.clone(),
        }
    }
}

/// Error taxonomy for upgrade orchestration validation and lifecycle failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpgradeOrchestrationError {
    /// Required field is empty.
    EmptyField(&'static str),
    /// Timestamp is invalid.
    InvalidTimestamp(&'static str),
    /// DID parsing failed.
    InvalidDid {
        /// Input field carrying invalid DID.
        field: &'static str,
        /// Stable deterministic reason marker.
        reason_code: &'static str,
        /// Canonical parser detail.
        detail: String,
    },
    /// Version format is invalid.
    InvalidVersionFormat(String),
    /// Target version is not a valid forward transition from current version.
    InvalidTargetVersionTransition {
        /// Current active version.
        current_version: String,
        /// Proposed target version.
        target_version: String,
    },
    /// Required quorum value is invalid.
    InvalidRequiredQuorum(usize),
    /// Duplicate proposal identifier was submitted.
    DuplicateProposal(String),
    /// Proposal identifier was not found.
    ProposalNotFound(String),
    /// Duplicate approval from the same validator.
    DuplicateApproval {
        /// Proposal identifier.
        proposal_id: String,
        /// Validator DID that duplicated approval.
        validator_did: String,
    },
    /// Governance status does not allow activation.
    GovernanceNotApproved {
        /// Proposal identifier.
        proposal_id: String,
        /// Current governance status.
        status: GovernanceProposalStatus,
    },
    /// Approval set does not satisfy required quorum.
    InsufficientApprovals {
        /// Required quorum count.
        required: usize,
        /// Provided approval count.
        provided: usize,
    },
    /// Proposal is already activated.
    AlreadyActivated(String),
    /// Rollback attempted from an invalid proposal state.
    RollbackNotAllowed(String),
}

impl fmt::Display for UpgradeOrchestrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField(field) => write!(f, "field must not be empty: {field}"),
            Self::InvalidTimestamp(field) => write!(f, "timestamp must be > 0: {field}"),
            Self::InvalidDid {
                field,
                reason_code,
                detail,
            } => write!(f, "invalid did field {field}: {reason_code} ({detail})"),
            Self::InvalidVersionFormat(value) => {
                write!(f, "invalid version format, expected vX.Y.Z: {value}")
            }
            Self::InvalidTargetVersionTransition {
                current_version,
                target_version,
            } => write!(
                f,
                "invalid target version transition: current={current_version}, target={target_version}"
            ),
            Self::InvalidRequiredQuorum(value) => write!(f, "invalid required quorum: {value}"),
            Self::DuplicateProposal(proposal_id) => {
                write!(f, "duplicate upgrade proposal id: {proposal_id}")
            }
            Self::ProposalNotFound(proposal_id) => {
                write!(f, "upgrade proposal not found: {proposal_id}")
            }
            Self::DuplicateApproval {
                proposal_id,
                validator_did,
            } => write!(
                f,
                "duplicate upgrade approval: proposal={proposal_id}, validator={validator_did}"
            ),
            Self::GovernanceNotApproved {
                proposal_id,
                status,
            } => write!(
                f,
                "governance not approved for upgrade activation: proposal={proposal_id}, status={status:?}"
            ),
            Self::InsufficientApprovals { required, provided } => write!(
                f,
                "insufficient approvals for upgrade activation: required {required}, provided {provided}"
            ),
            Self::AlreadyActivated(proposal_id) => {
                write!(f, "upgrade proposal already activated: {proposal_id}")
            }
            Self::RollbackNotAllowed(proposal_id) => {
                write!(f, "upgrade rollback not allowed for proposal: {proposal_id}")
            }
        }
    }
}

impl std::error::Error for UpgradeOrchestrationError {}

fn validate_timestamp(field: &'static str, value: u64) -> Result<(), UpgradeOrchestrationError> {
    if value == 0 {
        return Err(UpgradeOrchestrationError::InvalidTimestamp(field));
    }
    Ok(())
}

fn validate_did(
    value: &str,
    field: &'static str,
    reason_code: &'static str,
) -> Result<AgentDid, UpgradeOrchestrationError> {
    AgentDid::parse(value).map_err(|error| UpgradeOrchestrationError::InvalidDid {
        field,
        reason_code,
        detail: error.to_string(),
    })
}

fn require_non_empty(field: &'static str, value: &str) -> Result<(), UpgradeOrchestrationError> {
    if value.trim().is_empty() {
        return Err(UpgradeOrchestrationError::EmptyField(field));
    }
    Ok(())
}

fn parse_version(value: &str) -> Result<(u64, u64, u64), UpgradeOrchestrationError> {
    let normalized = value.strip_prefix('v').unwrap_or(value);
    let parts: Vec<&str> = normalized.split('.').collect();
    if parts.len() != 3 {
        return Err(UpgradeOrchestrationError::InvalidVersionFormat(
            value.to_owned(),
        ));
    }
    let major = parts[0]
        .parse::<u64>()
        .map_err(|_| UpgradeOrchestrationError::InvalidVersionFormat(value.to_owned()))?;
    let minor = parts[1]
        .parse::<u64>()
        .map_err(|_| UpgradeOrchestrationError::InvalidVersionFormat(value.to_owned()))?;
    let patch = parts[2]
        .parse::<u64>()
        .map_err(|_| UpgradeOrchestrationError::InvalidVersionFormat(value.to_owned()))?;
    Ok((major, minor, patch))
}

fn validate_version_format(value: &str) -> Result<(), UpgradeOrchestrationError> {
    parse_version(value).map(|_| ())
}

fn is_version_advance(
    current_version: &str,
    target_version: &str,
) -> Result<bool, UpgradeOrchestrationError> {
    Ok(parse_version(target_version)? > parse_version(current_version)?)
}

#[cfg(test)]
mod tests {
    use super::{is_version_advance, UpgradeOrchestrationError, VersionUpgradeOrchestrator};

    #[test]
    fn parse_version_requires_three_segments() {
        assert_eq!(
            VersionUpgradeOrchestrator::new("v0.1"),
            Err(UpgradeOrchestrationError::InvalidVersionFormat(
                "v0.1".to_owned()
            ))
        );
    }

    #[test]
    fn version_advance_is_semver_ordered() {
        assert!(
            is_version_advance("v0.9.9", "v1.0.0").expect("comparison should succeed"),
            "major version advance should be valid"
        );
    }
}
