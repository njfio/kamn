//! Operator dashboard UI projection contracts for human-facing control surfaces.

use crate::{
    AgentDid, EscrowStatus, MessageStatus, OperatorActionAuditRecord, OperatorActionOutcome,
    OperatorBindingAction, OperatorDashboardSnapshot, TaskState,
};
use std::fmt;

const HUMAN_DID_PREFIX: &str = "kamn:did:human:";
const OPERATOR_DASHBOARD_UI_INVALID_AGENT_DID_REASON_CODE: &str =
    "operator_dashboard_ui_invalid_agent_did";
const OPERATOR_DASHBOARD_UI_INVALID_TASK_REQUESTER_DID_REASON_CODE: &str =
    "operator_dashboard_ui_invalid_task_requester_did";
const OPERATOR_DASHBOARD_UI_INVALID_TASK_ASSIGNEE_DID_REASON_CODE: &str =
    "operator_dashboard_ui_invalid_task_assignee_did";
const OPERATOR_DASHBOARD_UI_INVALID_MESSAGE_SENDER_DID_REASON_CODE: &str =
    "operator_dashboard_ui_invalid_message_sender_did";
const OPERATOR_DASHBOARD_UI_INVALID_MESSAGE_RECIPIENT_DID_REASON_CODE: &str =
    "operator_dashboard_ui_invalid_message_recipient_did";
const OPERATOR_DASHBOARD_UI_INVALID_ESCROW_PAYER_DID_REASON_CODE: &str =
    "operator_dashboard_ui_invalid_escrow_payer_did";
const OPERATOR_DASHBOARD_UI_INVALID_ESCROW_PAYEE_DID_REASON_CODE: &str =
    "operator_dashboard_ui_invalid_escrow_payee_did";
const OPERATOR_DASHBOARD_UI_INVALID_REPUTATION_AGENT_DID_REASON_CODE: &str =
    "operator_dashboard_ui_invalid_reputation_agent_did";
const OPERATOR_DASHBOARD_UI_INVALID_AUDIT_AGENT_DID_REASON_CODE: &str =
    "operator_dashboard_ui_invalid_audit_agent_did";
const OPERATOR_DASHBOARD_UI_INVALID_AUDIT_OPERATOR_DID_REASON_CODE: &str =
    "operator_dashboard_ui_invalid_audit_operator_did";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// UI attention severity used for triage highlighting.
pub enum DashboardAttentionLevel {
    /// Informational state requiring no intervention.
    Info,
    /// Elevated state worth operator review.
    Warning,
    /// Critical state requiring operator intervention.
    Critical,
    /// Positive/completed state.
    Success,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Reputation risk tier used by operator dashboards.
pub enum ReputationRiskTier {
    /// Healthy reputation signal profile.
    Stable,
    /// Watchlist reputation signal profile.
    Watch,
    /// Critical reputation signal profile.
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Top-level counters for dashboard overview cards.
pub struct DashboardSummary {
    /// Number of registered agents in the projection.
    pub total_agents: usize,
    /// Number of tasks in non-terminal active states.
    pub active_tasks: usize,
    /// Number of blocked tasks.
    pub blocked_tasks: usize,
    /// Number of messages in failure states.
    pub failed_messages: usize,
    /// Number of escrows currently disputed.
    pub disputed_escrows: usize,
    /// Number of agents in critical reputation tier.
    pub critical_reputation_agents: usize,
    /// Number of denied operator actions in audit trail.
    pub denied_operator_actions: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// UI row projection for agent list views.
pub struct OperatorAgentListRow {
    /// Agent DID identifier.
    pub agent_did: String,
    /// Concise key hierarchy summary for display.
    pub key_summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// UI entry for task timeline displays.
pub struct OperatorTaskTimelineEntry {
    /// Task identifier.
    pub task_id: String,
    /// Effective owner (assignee or requester).
    pub owner: String,
    /// Current task state.
    pub state: TaskState,
    /// Attention level derived from task state.
    pub attention: DashboardAttentionLevel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// UI entry for message delivery trace views.
pub struct OperatorMessageTraceEntry {
    /// Message identifier.
    pub message_id: String,
    /// Sender DID.
    pub sender: String,
    /// Number of recipients for the message.
    pub recipient_count: usize,
    /// Current message status.
    pub status: MessageStatus,
    /// Attention level derived from message status.
    pub attention: DashboardAttentionLevel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// UI entry for escrow status tracking.
pub struct OperatorEscrowStatusEntry {
    /// Escrow identifier.
    pub escrow_id: String,
    /// Payer DID.
    pub payer: String,
    /// Payee DID.
    pub payee: String,
    /// Current escrow status.
    pub status: EscrowStatus,
    /// Remaining amount in base units.
    pub remaining_amount: u128,
    /// Attention level derived from escrow status.
    pub attention: DashboardAttentionLevel,
}

#[derive(Debug, Clone, PartialEq)]
/// UI entry for reputation overview tables.
pub struct OperatorReputationOverviewEntry {
    /// Agent DID identifier.
    pub agent_did: String,
    /// Trust score scalar.
    pub trust_score: u32,
    /// Delivery success rate.
    pub delivery_rate: f64,
    /// Dispute rate.
    pub dispute_rate: f64,
    /// Derived risk tier.
    pub risk_tier: ReputationRiskTier,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// UI entry for operator-action audit trace tables.
pub struct OperatorAuditTraceEntry {
    /// Subject agent DID.
    pub agent_did: String,
    /// Operator DID that initiated the action.
    pub operator_did: String,
    /// Operator binding action kind.
    pub action: OperatorBindingAction,
    /// Target capability/field.
    pub target: String,
    /// Optional action value payload.
    pub value: Option<String>,
    /// Request timestamp (epoch seconds).
    pub requested_at_unix: u64,
    /// Action outcome classification.
    pub outcome: OperatorActionOutcome,
    /// Attention level derived from outcome.
    pub attention: DashboardAttentionLevel,
}

#[derive(Debug, Clone, PartialEq)]
/// Composite UI model produced for operator dashboard rendering.
pub struct OperatorDashboardUiModel {
    /// Summary counters.
    pub summary: DashboardSummary,
    /// Agent table rows.
    pub agent_list: Vec<OperatorAgentListRow>,
    /// Task timeline entries.
    pub task_timeline: Vec<OperatorTaskTimelineEntry>,
    /// Message trace entries.
    pub message_traces: Vec<OperatorMessageTraceEntry>,
    /// Escrow status entries.
    pub escrow_status: Vec<OperatorEscrowStatusEntry>,
    /// Reputation overview entries.
    pub reputation_overview: Vec<OperatorReputationOverviewEntry>,
    /// Audit trace entries.
    pub audit_traces: Vec<OperatorAuditTraceEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
/// Stateless UI composer for operator dashboard projections.
pub struct OperatorDashboardUi;

impl OperatorDashboardUi {
    /// Creates an operator dashboard UI composer.
    pub fn new() -> Self {
        Self
    }

    /// Composes a presentation-ready UI model from runtime snapshot and audit log.
    pub fn compose(
        &self,
        snapshot: &OperatorDashboardSnapshot,
        audit_log: &[OperatorActionAuditRecord],
    ) -> Result<OperatorDashboardUiModel, OperatorDashboardUiError> {
        let mut agent_list = Vec::with_capacity(snapshot.agents.items.len());
        for agent in &snapshot.agents.items {
            let agent_did = parse_agent_did(
                agent.agent_did.as_str(),
                "snapshot.agents[].agent_did",
                OPERATOR_DASHBOARD_UI_INVALID_AGENT_DID_REASON_CODE,
            )?;
            if agent.identity_key_id.trim().is_empty() {
                return Err(OperatorDashboardUiError::EmptyAgentKey {
                    agent_did: agent.agent_did.clone(),
                    key_role: "identity",
                });
            }
            if agent.signing_key_id.trim().is_empty() {
                return Err(OperatorDashboardUiError::EmptyAgentKey {
                    agent_did: agent.agent_did.clone(),
                    key_role: "signing",
                });
            }
            if agent.agreement_key_id.trim().is_empty() {
                return Err(OperatorDashboardUiError::EmptyAgentKey {
                    agent_did: agent.agent_did.clone(),
                    key_role: "agreement",
                });
            }

            agent_list.push(OperatorAgentListRow {
                agent_did: agent_did.as_str().to_owned(),
                key_summary: format!(
                    "{}/{}/{}",
                    agent.identity_key_id, agent.signing_key_id, agent.agreement_key_id
                ),
            });
        }
        agent_list.sort_by(|left, right| left.agent_did.cmp(&right.agent_did));

        let mut task_timeline = Vec::with_capacity(snapshot.tasks.items.len());
        for task in &snapshot.tasks.items {
            let requester = parse_agent_did(
                task.requester.as_str(),
                "snapshot.tasks[].requester",
                OPERATOR_DASHBOARD_UI_INVALID_TASK_REQUESTER_DID_REASON_CODE,
            )?;
            let assignee = task
                .assignee
                .as_deref()
                .map(|value| {
                    parse_agent_did(
                        value,
                        "snapshot.tasks[].assignee",
                        OPERATOR_DASHBOARD_UI_INVALID_TASK_ASSIGNEE_DID_REASON_CODE,
                    )
                    .map(|did| did.as_str().to_owned())
                })
                .transpose()?;
            task_timeline.push(OperatorTaskTimelineEntry {
                task_id: task.task_id.clone(),
                owner: assignee.unwrap_or_else(|| requester.as_str().to_owned()),
                state: task.state,
                attention: task_attention(task.state),
            });
        }
        task_timeline.sort_by(|left, right| left.task_id.cmp(&right.task_id));

        let mut message_traces = Vec::with_capacity(snapshot.messages.items.len());
        for message in &snapshot.messages.items {
            if message.recipients.is_empty() {
                return Err(OperatorDashboardUiError::EmptyMessageRecipients(
                    message.message_id.clone(),
                ));
            }
            let sender = parse_agent_did(
                message.sender.as_str(),
                "snapshot.messages[].sender",
                OPERATOR_DASHBOARD_UI_INVALID_MESSAGE_SENDER_DID_REASON_CODE,
            )?;
            for recipient in &message.recipients {
                parse_agent_did(
                    recipient.as_str(),
                    "snapshot.messages[].recipients[]",
                    OPERATOR_DASHBOARD_UI_INVALID_MESSAGE_RECIPIENT_DID_REASON_CODE,
                )?;
            }

            message_traces.push(OperatorMessageTraceEntry {
                message_id: message.message_id.clone(),
                sender: sender.as_str().to_owned(),
                recipient_count: message.recipients.len(),
                status: message.status,
                attention: message_attention(message.status),
            });
        }
        message_traces.sort_by(|left, right| left.message_id.cmp(&right.message_id));

        let mut escrow_status = Vec::with_capacity(snapshot.escrows.items.len());
        for escrow in &snapshot.escrows.items {
            let payer = parse_agent_did(
                escrow.payer.as_str(),
                "snapshot.escrows[].payer",
                OPERATOR_DASHBOARD_UI_INVALID_ESCROW_PAYER_DID_REASON_CODE,
            )?;
            let payee = parse_agent_did(
                escrow.payee.as_str(),
                "snapshot.escrows[].payee",
                OPERATOR_DASHBOARD_UI_INVALID_ESCROW_PAYEE_DID_REASON_CODE,
            )?;
            escrow_status.push(OperatorEscrowStatusEntry {
                escrow_id: escrow.escrow_id.clone(),
                payer: payer.as_str().to_owned(),
                payee: payee.as_str().to_owned(),
                status: escrow.status.clone(),
                remaining_amount: escrow.remaining_amount,
                attention: escrow_attention(&escrow.status),
            });
        }
        escrow_status.sort_by(|left, right| left.escrow_id.cmp(&right.escrow_id));

        let mut reputation_overview = Vec::with_capacity(snapshot.reputation.items.len());
        for reputation in &snapshot.reputation.items {
            let agent_did = parse_agent_did(
                reputation.agent_did.as_str(),
                "snapshot.reputation[].agent_did",
                OPERATOR_DASHBOARD_UI_INVALID_REPUTATION_AGENT_DID_REASON_CODE,
            )?;
            validate_rate(
                "delivery_rate",
                agent_did.as_str(),
                reputation.delivery_rate,
            )?;
            validate_rate("dispute_rate", agent_did.as_str(), reputation.dispute_rate)?;
            reputation_overview.push(OperatorReputationOverviewEntry {
                agent_did: agent_did.as_str().to_owned(),
                trust_score: reputation.trust_score,
                delivery_rate: reputation.delivery_rate,
                dispute_rate: reputation.dispute_rate,
                risk_tier: reputation_risk_tier(reputation.trust_score, reputation.dispute_rate),
            });
        }
        reputation_overview.sort_by(|left, right| left.agent_did.cmp(&right.agent_did));

        let mut audit_traces = Vec::with_capacity(audit_log.len());
        for record in audit_log {
            let agent_did = parse_agent_did(
                record.agent_did.as_str(),
                "audit_log[].agent_did",
                OPERATOR_DASHBOARD_UI_INVALID_AUDIT_AGENT_DID_REASON_CODE,
            )?;
            let operator_did = parse_operator_did(
                record.operator_did.as_str(),
                "audit_log[].operator_did",
                OPERATOR_DASHBOARD_UI_INVALID_AUDIT_OPERATOR_DID_REASON_CODE,
            )?;
            if record.requested_at_unix == 0 {
                return Err(OperatorDashboardUiError::InvalidAuditTimestamp {
                    operator_did: operator_did.clone(),
                });
            }

            audit_traces.push(OperatorAuditTraceEntry {
                agent_did: agent_did.as_str().to_owned(),
                operator_did: operator_did.clone(),
                action: record.action,
                target: record.target.clone(),
                value: record.value.clone(),
                requested_at_unix: record.requested_at_unix,
                outcome: record.outcome.clone(),
                attention: match record.outcome {
                    OperatorActionOutcome::Denied => DashboardAttentionLevel::Critical,
                    OperatorActionOutcome::Allowed => DashboardAttentionLevel::Info,
                },
            });
        }
        audit_traces.sort_by(|left, right| {
            right
                .requested_at_unix
                .cmp(&left.requested_at_unix)
                .then_with(|| left.operator_did.cmp(&right.operator_did))
                .then_with(|| left.target.cmp(&right.target))
        });

        let summary = DashboardSummary {
            total_agents: agent_list.len(),
            active_tasks: task_timeline
                .iter()
                .filter(|task| {
                    !matches!(
                        task.state,
                        TaskState::Completed | TaskState::Failed | TaskState::Cancelled
                    )
                })
                .count(),
            blocked_tasks: task_timeline
                .iter()
                .filter(|task| matches!(task.state, TaskState::Blocked))
                .count(),
            failed_messages: message_traces
                .iter()
                .filter(|message| {
                    matches!(
                        message.status,
                        MessageStatus::Rejected | MessageStatus::Expired
                    )
                })
                .count(),
            disputed_escrows: escrow_status
                .iter()
                .filter(|escrow| matches!(escrow.status, EscrowStatus::Disputed))
                .count(),
            critical_reputation_agents: reputation_overview
                .iter()
                .filter(|entry| matches!(entry.risk_tier, ReputationRiskTier::Critical))
                .count(),
            denied_operator_actions: audit_traces
                .iter()
                .filter(|trace| matches!(trace.outcome, OperatorActionOutcome::Denied))
                .count(),
        };

        Ok(OperatorDashboardUiModel {
            summary,
            agent_list,
            task_timeline,
            message_traces,
            escrow_status,
            reputation_overview,
            audit_traces,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
/// Error taxonomy for dashboard UI projection validation.
pub enum OperatorDashboardUiError {
    /// Agent key role is empty.
    EmptyAgentKey {
        /// Agent DID with missing key.
        agent_did: String,
        /// Missing key role label.
        key_role: &'static str,
    },
    /// Message projection is missing recipients.
    EmptyMessageRecipients(String),
    /// DID value is invalid.
    InvalidDid {
        /// Input field carrying the DID value.
        field: &'static str,
        /// Stable reason marker.
        reason_code: &'static str,
        /// Canonical parser detail.
        detail: String,
    },
    /// Reputation rate is outside the supported range.
    InvalidReputationRate {
        /// Agent DID associated with invalid rate.
        agent_did: String,
        /// Invalid field name.
        field: &'static str,
        /// Invalid value.
        value: f64,
    },
    /// Audit record timestamp is invalid.
    InvalidAuditTimestamp {
        /// Operator DID for the invalid record.
        operator_did: String,
    },
}

impl fmt::Display for OperatorDashboardUiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyAgentKey {
                agent_did,
                key_role,
            } => {
                write!(f, "agent key role {key_role} is empty for {agent_did}")
            }
            Self::EmptyMessageRecipients(message_id) => {
                write!(
                    f,
                    "message trace must include at least one recipient: {message_id}"
                )
            }
            Self::InvalidDid {
                field,
                reason_code,
                detail,
            } => write!(f, "invalid did field {field}: {reason_code} ({detail})"),
            Self::InvalidReputationRate {
                agent_did,
                field,
                value,
            } => write!(
                f,
                "invalid reputation rate {field} for {agent_did}: expected 0..=1, found {value}"
            ),
            Self::InvalidAuditTimestamp { operator_did } => {
                write!(
                    f,
                    "invalid audit timestamp: operator action timestamp must be > 0 for {operator_did}"
                )
            }
        }
    }
}

impl std::error::Error for OperatorDashboardUiError {}

fn validate_rate(
    field: &'static str,
    agent_did: &str,
    value: f64,
) -> Result<(), OperatorDashboardUiError> {
    if !(0.0..=1.0).contains(&value) {
        return Err(OperatorDashboardUiError::InvalidReputationRate {
            agent_did: agent_did.to_owned(),
            field,
            value,
        });
    }
    Ok(())
}

fn parse_agent_did(
    value: &str,
    field: &'static str,
    reason_code: &'static str,
) -> Result<AgentDid, OperatorDashboardUiError> {
    AgentDid::parse(value).map_err(|error| OperatorDashboardUiError::InvalidDid {
        field,
        reason_code,
        detail: error.to_string(),
    })
}

fn parse_operator_did(
    value: &str,
    field: &'static str,
    reason_code: &'static str,
) -> Result<String, OperatorDashboardUiError> {
    if !value.starts_with(HUMAN_DID_PREFIX) {
        return Err(OperatorDashboardUiError::InvalidDid {
            field,
            reason_code,
            detail: format!("invalid human did prefix: {value}"),
        });
    }
    let method_specific_id = &value[HUMAN_DID_PREFIX.len()..];
    if method_specific_id.is_empty() {
        return Err(OperatorDashboardUiError::InvalidDid {
            field,
            reason_code,
            detail: "human did method-specific id must not be empty".to_owned(),
        });
    }
    if !method_specific_id
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_')
    {
        return Err(OperatorDashboardUiError::InvalidDid {
            field,
            reason_code,
            detail: format!("human did has invalid characters: {method_specific_id}"),
        });
    }
    Ok(value.to_owned())
}

fn task_attention(state: TaskState) -> DashboardAttentionLevel {
    match state {
        TaskState::Blocked | TaskState::Failed => DashboardAttentionLevel::Critical,
        TaskState::Cancelled | TaskState::Delegated | TaskState::InputRequired => {
            DashboardAttentionLevel::Warning
        }
        TaskState::Completed => DashboardAttentionLevel::Success,
        TaskState::Submitted | TaskState::Accepted | TaskState::InProgress => {
            DashboardAttentionLevel::Info
        }
    }
}

fn message_attention(state: MessageStatus) -> DashboardAttentionLevel {
    match state {
        MessageStatus::Rejected | MessageStatus::Expired => DashboardAttentionLevel::Critical,
        MessageStatus::Created
        | MessageStatus::Signed
        | MessageStatus::Broadcast
        | MessageStatus::Included
        | MessageStatus::Delivered => DashboardAttentionLevel::Warning,
        MessageStatus::Validated => DashboardAttentionLevel::Success,
    }
}

fn escrow_attention(state: &EscrowStatus) -> DashboardAttentionLevel {
    match state {
        EscrowStatus::Disputed => DashboardAttentionLevel::Critical,
        EscrowStatus::Refunded | EscrowStatus::PartiallyReleased { .. } => {
            DashboardAttentionLevel::Warning
        }
        EscrowStatus::Released | EscrowStatus::Resolved { .. } => DashboardAttentionLevel::Success,
        EscrowStatus::Funded => DashboardAttentionLevel::Info,
    }
}

fn reputation_risk_tier(trust_score: u32, dispute_rate: f64) -> ReputationRiskTier {
    if trust_score < 300 || dispute_rate > 0.30 {
        ReputationRiskTier::Critical
    } else if trust_score < 600 || dispute_rate > 0.15 {
        ReputationRiskTier::Watch
    } else {
        ReputationRiskTier::Stable
    }
}

#[cfg(test)]
mod tests {
    use super::{OperatorDashboardUiError, ReputationRiskTier};

    #[test]
    fn reputation_tier_marks_critical_for_high_dispute_rate() {
        assert_eq!(
            super::reputation_risk_tier(800, 0.45),
            ReputationRiskTier::Critical
        );
    }

    #[test]
    fn validate_rate_rejects_out_of_range_values() {
        assert_eq!(
            super::validate_rate("delivery_rate", "kamn:did:agent:ops-1", 1.5),
            Err(OperatorDashboardUiError::InvalidReputationRate {
                agent_did: "kamn:did:agent:ops-1".to_owned(),
                field: "delivery_rate",
                value: 1.5
            })
        );
    }
}
