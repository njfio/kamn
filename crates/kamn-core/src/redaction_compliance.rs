//! Redaction and tombstone compliance workflow contracts.
//!
//! The module models request submission, approval quorum, rejection handling, and
//! audit trail emission for content visibility protection policies.

use crate::{canonical_state_key, AgentDid};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// Action requested by a compliance redaction workflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RedactionAction {
    /// Redact the target while keeping placeholder visibility metadata.
    Redact,
    /// Tombstone the target as removed content.
    Tombstone,
}

/// Current request lifecycle state for a redaction proposal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RedactionRequestStatus {
    /// Request is waiting for additional approvals.
    PendingApproval {
        /// Unique approvals currently collected.
        approvals_collected: usize,
        /// Total approvals required to apply the request.
        approvals_required: usize,
    },
    /// Request was explicitly rejected.
    Rejected,
    /// Request reached approval quorum and was applied.
    Applied,
}

/// Visibility outcome for a protected target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RedactionVisibility {
    /// Target remains available and unprotected.
    Available,
    /// Target is redacted under the given request id.
    Redacted {
        /// Request identifier that triggered redaction.
        request_id: String,
    },
    /// Target is tombstoned under the given request id.
    Tombstoned {
        /// Request identifier that triggered tombstoning.
        request_id: String,
    },
}

/// Audit-event category emitted by redaction workflows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RedactionAuditEventKind {
    /// Request was submitted.
    Requested,
    /// Request received an approval.
    Approved,
    /// Request was rejected.
    Rejected,
    /// Request was applied after quorum.
    Applied,
}

/// Audit trail entry attached to a redaction request id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedactionAuditEvent {
    /// Request identifier associated with this audit event.
    pub request_id: String,
    /// DID of the actor that produced this event.
    pub actor: String,
    /// Event kind classification.
    pub kind: RedactionAuditEventKind,
    /// Timestamp when the event occurred.
    pub at: String,
    /// Human-readable reason or note payload.
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RedactionRequestRecord {
    id: String,
    target_namespace: String,
    target_entity_id: String,
    requester: String,
    action: RedactionAction,
    status: InternalStatus,
    approvals: BTreeSet<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InternalStatus {
    PendingApproval,
    Rejected,
    Applied,
}

/// In-memory engine for redaction request lifecycle and visibility state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedactionComplianceEngine {
    approvals_required: usize,
    requests: BTreeMap<String, RedactionRequestRecord>,
    visibility_by_target: BTreeMap<String, RedactionVisibility>,
    protected_targets: BTreeSet<String>,
    audit_events_by_request: BTreeMap<String, Vec<RedactionAuditEvent>>,
}

impl RedactionComplianceEngine {
    /// Constructs an engine with a fixed approval quorum requirement.
    pub fn new(approvals_required: usize) -> Result<Self, RedactionComplianceError> {
        if approvals_required == 0 {
            return Err(RedactionComplianceError::InvalidApprovalsRequired(
                approvals_required,
            ));
        }

        Ok(Self {
            approvals_required,
            requests: BTreeMap::new(),
            visibility_by_target: BTreeMap::new(),
            protected_targets: BTreeSet::new(),
            audit_events_by_request: BTreeMap::new(),
        })
    }

    #[allow(clippy::too_many_arguments)]
    /// Submits a redaction or tombstone request for a target record.
    pub fn submit_request(
        &mut self,
        request_id: &str,
        target_namespace: &str,
        target_entity_id: &str,
        requester: &str,
        action: RedactionAction,
        reason: &str,
        requested_at: &str,
    ) -> Result<(), RedactionComplianceError> {
        validate_non_empty("request_id", request_id)?;
        validate_non_empty("target_namespace", target_namespace)?;
        validate_non_empty("target_entity_id", target_entity_id)?;
        validate_did(requester)?;
        validate_non_empty("reason", reason)?;
        validate_non_empty("requested_at", requested_at)?;

        if self.requests.contains_key(request_id) {
            return Err(RedactionComplianceError::DuplicateRequestId(
                request_id.to_owned(),
            ));
        }

        let key = self.target_storage_key(target_namespace, target_entity_id)?;
        if self.protected_targets.contains(&key) {
            return Err(RedactionComplianceError::TargetAlreadyProtected {
                namespace: target_namespace.to_owned(),
                entity_id: target_entity_id.to_owned(),
            });
        }

        self.requests.insert(
            request_id.to_owned(),
            RedactionRequestRecord {
                id: request_id.to_owned(),
                target_namespace: target_namespace.to_owned(),
                target_entity_id: target_entity_id.to_owned(),
                requester: requester.to_owned(),
                action,
                status: InternalStatus::PendingApproval,
                approvals: BTreeSet::new(),
            },
        );

        self.push_audit_event(RedactionAuditEvent {
            request_id: request_id.to_owned(),
            actor: requester.to_owned(),
            kind: RedactionAuditEventKind::Requested,
            at: requested_at.to_owned(),
            note: reason.to_owned(),
        });

        Ok(())
    }

    /// Approves a pending request and applies visibility protection at quorum.
    pub fn approve(
        &mut self,
        request_id: &str,
        approver: &str,
        approved_at: &str,
        note: &str,
    ) -> Result<(), RedactionComplianceError> {
        validate_non_empty("request_id", request_id)?;
        validate_did(approver)?;
        validate_non_empty("approved_at", approved_at)?;
        validate_non_empty("note", note)?;

        let approvals_required = self.approvals_required;
        let mut apply_snapshot: Option<(String, String, RedactionAction)> = None;

        {
            let record = self.request_mut(request_id)?;
            if record.status != InternalStatus::PendingApproval {
                return Err(RedactionComplianceError::RequestNotPendingApproval(
                    request_id.to_owned(),
                ));
            }

            if !record.approvals.insert(approver.to_owned()) {
                return Err(RedactionComplianceError::DuplicateApproval {
                    request_id: request_id.to_owned(),
                    approver: approver.to_owned(),
                });
            }

            if record.approvals.len() >= approvals_required {
                apply_snapshot = Some((
                    record.target_namespace.clone(),
                    record.target_entity_id.clone(),
                    record.action,
                ));
                record.status = InternalStatus::Applied;
            }
        }

        self.push_audit_event(RedactionAuditEvent {
            request_id: request_id.to_owned(),
            actor: approver.to_owned(),
            kind: RedactionAuditEventKind::Approved,
            at: approved_at.to_owned(),
            note: note.to_owned(),
        });

        if let Some((target_namespace, target_entity_id, action)) = apply_snapshot {
            let target_key = self.target_storage_key(&target_namespace, &target_entity_id)?;
            let visibility = match action {
                RedactionAction::Redact => RedactionVisibility::Redacted {
                    request_id: request_id.to_owned(),
                },
                RedactionAction::Tombstone => RedactionVisibility::Tombstoned {
                    request_id: request_id.to_owned(),
                },
            };
            self.visibility_by_target
                .insert(target_key.clone(), visibility);
            self.protected_targets.insert(target_key);

            self.push_audit_event(RedactionAuditEvent {
                request_id: request_id.to_owned(),
                actor: approver.to_owned(),
                kind: RedactionAuditEventKind::Applied,
                at: approved_at.to_owned(),
                note: "quorum reached; protection applied".to_owned(),
            });
        }

        Ok(())
    }

    /// Rejects a pending request and records an audit event.
    pub fn reject(
        &mut self,
        request_id: &str,
        actor: &str,
        rejected_at: &str,
        reason: &str,
    ) -> Result<(), RedactionComplianceError> {
        validate_non_empty("request_id", request_id)?;
        validate_did(actor)?;
        validate_non_empty("rejected_at", rejected_at)?;
        validate_non_empty("reason", reason)?;

        let record = self.request_mut(request_id)?;
        if record.status != InternalStatus::PendingApproval {
            return Err(RedactionComplianceError::RequestNotPendingApproval(
                request_id.to_owned(),
            ));
        }
        record.status = InternalStatus::Rejected;

        self.push_audit_event(RedactionAuditEvent {
            request_id: request_id.to_owned(),
            actor: actor.to_owned(),
            kind: RedactionAuditEventKind::Rejected,
            at: rejected_at.to_owned(),
            note: reason.to_owned(),
        });

        Ok(())
    }

    /// Returns lifecycle status for the given request id.
    pub fn request_status(
        &self,
        request_id: &str,
    ) -> Result<RedactionRequestStatus, RedactionComplianceError> {
        let record = self
            .requests
            .get(request_id)
            .ok_or_else(|| RedactionComplianceError::NotFound(request_id.to_owned()))?;

        Ok(match record.status {
            InternalStatus::PendingApproval => RedactionRequestStatus::PendingApproval {
                approvals_collected: record.approvals.len(),
                approvals_required: self.approvals_required,
            },
            InternalStatus::Rejected => RedactionRequestStatus::Rejected,
            InternalStatus::Applied => RedactionRequestStatus::Applied,
        })
    }

    /// Returns current visibility state for a target namespace/entity pair.
    pub fn retrieve_visibility(
        &self,
        target_namespace: &str,
        target_entity_id: &str,
    ) -> Result<RedactionVisibility, RedactionComplianceError> {
        let key = self.target_storage_key(target_namespace, target_entity_id)?;
        Ok(self
            .visibility_by_target
            .get(&key)
            .cloned()
            .unwrap_or(RedactionVisibility::Available))
    }

    /// Builds the canonical storage key used to address a target record.
    pub fn target_storage_key(
        &self,
        target_namespace: &str,
        target_entity_id: &str,
    ) -> Result<String, RedactionComplianceError> {
        canonical_state_key(target_namespace, "record", target_entity_id)
            .map_err(|error| RedactionComplianceError::InvalidTarget(error.to_string()))
    }

    /// Returns audit events recorded for a request id.
    pub fn audit_events(
        &self,
        request_id: &str,
    ) -> Result<Vec<RedactionAuditEvent>, RedactionComplianceError> {
        if !self.requests.contains_key(request_id) {
            return Err(RedactionComplianceError::NotFound(request_id.to_owned()));
        }

        Ok(self
            .audit_events_by_request
            .get(request_id)
            .cloned()
            .unwrap_or_default())
    }

    fn request_mut(
        &mut self,
        request_id: &str,
    ) -> Result<&mut RedactionRequestRecord, RedactionComplianceError> {
        self.requests
            .get_mut(request_id)
            .ok_or_else(|| RedactionComplianceError::NotFound(request_id.to_owned()))
    }

    fn push_audit_event(&mut self, event: RedactionAuditEvent) {
        self.audit_events_by_request
            .entry(event.request_id.clone())
            .or_default()
            .push(event);
    }
}

/// Error surface for redaction workflow validation and state transitions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RedactionComplianceError {
    /// `approvals_required` was zero.
    InvalidApprovalsRequired(usize),
    /// Required field was empty.
    EmptyField(&'static str),
    /// DID value failed validation.
    InvalidDid(String),
    /// Target namespace/entity key could not be canonicalized.
    InvalidTarget(String),
    /// Request id already exists.
    DuplicateRequestId(String),
    /// Target already has active protection from a prior request.
    TargetAlreadyProtected {
        /// Target namespace.
        namespace: String,
        /// Target entity identifier.
        entity_id: String,
    },
    /// Request id was not found.
    NotFound(String),
    /// Request is not in pending-approval state.
    RequestNotPendingApproval(String),
    /// Approver duplicated an existing approval for the request.
    DuplicateApproval {
        /// Request identifier.
        request_id: String,
        /// Approver DID.
        approver: String,
    },
}

impl fmt::Display for RedactionComplianceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidApprovalsRequired(value) => {
                write!(f, "approvals_required must be greater than zero: {value}")
            }
            Self::EmptyField(field) => write!(f, "field must not be empty: {field}"),
            Self::InvalidDid(value) => write!(f, "invalid did: {value}"),
            Self::InvalidTarget(value) => write!(f, "invalid target reference: {value}"),
            Self::DuplicateRequestId(value) => write!(f, "duplicate request id: {value}"),
            Self::TargetAlreadyProtected {
                namespace,
                entity_id,
            } => {
                write!(f, "target already protected: {namespace}/{entity_id}")
            }
            Self::NotFound(value) => write!(f, "redaction request not found: {value}"),
            Self::RequestNotPendingApproval(value) => {
                write!(f, "redaction request is not pending approval: {value}")
            }
            Self::DuplicateApproval {
                request_id,
                approver,
            } => write!(
                f,
                "duplicate approval for request {request_id} from approver {approver}"
            ),
        }
    }
}

impl std::error::Error for RedactionComplianceError {}

fn validate_non_empty(field: &'static str, value: &str) -> Result<(), RedactionComplianceError> {
    if value.trim().is_empty() {
        return Err(RedactionComplianceError::EmptyField(field));
    }
    Ok(())
}

fn validate_did(value: &str) -> Result<(), RedactionComplianceError> {
    AgentDid::parse(value)
        .map_err(|error| RedactionComplianceError::InvalidDid(error.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        RedactionAction, RedactionComplianceEngine, RedactionComplianceError,
        RedactionRequestStatus, RedactionVisibility,
    };

    #[test]
    fn constructor_rejects_zero_approvals() {
        assert_eq!(
            RedactionComplianceEngine::new(0),
            Err(RedactionComplianceError::InvalidApprovalsRequired(0))
        );
    }

    #[test]
    fn duplicate_approval_is_rejected() {
        let mut engine = RedactionComplianceEngine::new(2).expect("engine should construct");
        engine
            .submit_request(
                "req-dup-1",
                "kamn.messages",
                "msg-1",
                "kamn:did:agent:owner-1",
                RedactionAction::Redact,
                "request",
                "2026-02-08T10:00:00Z",
            )
            .expect("request should be accepted");
        engine
            .approve(
                "req-dup-1",
                "kamn:did:agent:approver-1",
                "2026-02-08T10:01:00Z",
                "first",
            )
            .expect("first approval should succeed");

        assert_eq!(
            engine.approve(
                "req-dup-1",
                "kamn:did:agent:approver-1",
                "2026-02-08T10:02:00Z",
                "duplicate",
            ),
            Err(RedactionComplianceError::DuplicateApproval {
                request_id: "req-dup-1".to_owned(),
                approver: "kamn:did:agent:approver-1".to_owned(),
            })
        );
    }

    #[test]
    fn rejected_request_stays_available() {
        let mut engine = RedactionComplianceEngine::new(1).expect("engine should construct");
        engine
            .submit_request(
                "req-reject-1",
                "kamn.tasks",
                "task-1",
                "kamn:did:agent:owner-1",
                RedactionAction::Tombstone,
                "request",
                "2026-02-08T10:00:00Z",
            )
            .expect("request should be accepted");
        engine
            .reject(
                "req-reject-1",
                "kamn:did:agent:approver-9",
                "2026-02-08T10:01:00Z",
                "insufficient grounds",
            )
            .expect("reject should succeed");

        assert_eq!(
            engine
                .request_status("req-reject-1")
                .expect("status should resolve"),
            RedactionRequestStatus::Rejected
        );
        assert_eq!(
            engine
                .retrieve_visibility("kamn.tasks", "task-1")
                .expect("visibility should resolve"),
            RedactionVisibility::Available
        );
    }
}
