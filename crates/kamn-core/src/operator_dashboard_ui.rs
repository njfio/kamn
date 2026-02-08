use crate::{
    EscrowStatus, MessageStatus, OperatorActionAuditRecord, OperatorActionOutcome,
    OperatorBindingAction, OperatorDashboardSnapshot, TaskState,
};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DashboardAttentionLevel {
    Info,
    Warning,
    Critical,
    Success,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReputationRiskTier {
    Stable,
    Watch,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DashboardSummary {
    pub total_agents: usize,
    pub active_tasks: usize,
    pub blocked_tasks: usize,
    pub failed_messages: usize,
    pub disputed_escrows: usize,
    pub critical_reputation_agents: usize,
    pub denied_operator_actions: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatorAgentListRow {
    pub agent_did: String,
    pub key_summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatorTaskTimelineEntry {
    pub task_id: String,
    pub owner: String,
    pub state: TaskState,
    pub attention: DashboardAttentionLevel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatorMessageTraceEntry {
    pub message_id: String,
    pub sender: String,
    pub recipient_count: usize,
    pub status: MessageStatus,
    pub attention: DashboardAttentionLevel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatorEscrowStatusEntry {
    pub escrow_id: String,
    pub payer: String,
    pub payee: String,
    pub status: EscrowStatus,
    pub remaining_amount: u128,
    pub attention: DashboardAttentionLevel,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OperatorReputationOverviewEntry {
    pub agent_did: String,
    pub trust_score: u32,
    pub delivery_rate: f64,
    pub dispute_rate: f64,
    pub risk_tier: ReputationRiskTier,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatorAuditTraceEntry {
    pub agent_did: String,
    pub operator_did: String,
    pub action: OperatorBindingAction,
    pub target: String,
    pub value: Option<String>,
    pub requested_at_unix: u64,
    pub outcome: OperatorActionOutcome,
    pub attention: DashboardAttentionLevel,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OperatorDashboardUiModel {
    pub summary: DashboardSummary,
    pub agent_list: Vec<OperatorAgentListRow>,
    pub task_timeline: Vec<OperatorTaskTimelineEntry>,
    pub message_traces: Vec<OperatorMessageTraceEntry>,
    pub escrow_status: Vec<OperatorEscrowStatusEntry>,
    pub reputation_overview: Vec<OperatorReputationOverviewEntry>,
    pub audit_traces: Vec<OperatorAuditTraceEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OperatorDashboardUi;

impl OperatorDashboardUi {
    pub fn new() -> Self {
        Self
    }

    pub fn compose(
        &self,
        snapshot: &OperatorDashboardSnapshot,
        audit_log: &[OperatorActionAuditRecord],
    ) -> Result<OperatorDashboardUiModel, OperatorDashboardUiError> {
        let mut agent_list = Vec::with_capacity(snapshot.agents.items.len());
        for agent in &snapshot.agents.items {
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
                agent_did: agent.agent_did.clone(),
                key_summary: format!(
                    "{}/{}/{}",
                    agent.identity_key_id, agent.signing_key_id, agent.agreement_key_id
                ),
            });
        }
        agent_list.sort_by(|left, right| left.agent_did.cmp(&right.agent_did));

        let mut task_timeline = Vec::with_capacity(snapshot.tasks.items.len());
        for task in &snapshot.tasks.items {
            task_timeline.push(OperatorTaskTimelineEntry {
                task_id: task.task_id.clone(),
                owner: task
                    .assignee
                    .clone()
                    .unwrap_or_else(|| task.requester.clone()),
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

            message_traces.push(OperatorMessageTraceEntry {
                message_id: message.message_id.clone(),
                sender: message.sender.clone(),
                recipient_count: message.recipients.len(),
                status: message.status,
                attention: message_attention(message.status),
            });
        }
        message_traces.sort_by(|left, right| left.message_id.cmp(&right.message_id));

        let mut escrow_status = Vec::with_capacity(snapshot.escrows.items.len());
        for escrow in &snapshot.escrows.items {
            escrow_status.push(OperatorEscrowStatusEntry {
                escrow_id: escrow.escrow_id.clone(),
                payer: escrow.payer.clone(),
                payee: escrow.payee.clone(),
                status: escrow.status.clone(),
                remaining_amount: escrow.remaining_amount,
                attention: escrow_attention(&escrow.status),
            });
        }
        escrow_status.sort_by(|left, right| left.escrow_id.cmp(&right.escrow_id));

        let mut reputation_overview = Vec::with_capacity(snapshot.reputation.items.len());
        for reputation in &snapshot.reputation.items {
            validate_rate(
                "delivery_rate",
                &reputation.agent_did,
                reputation.delivery_rate,
            )?;
            validate_rate(
                "dispute_rate",
                &reputation.agent_did,
                reputation.dispute_rate,
            )?;
            reputation_overview.push(OperatorReputationOverviewEntry {
                agent_did: reputation.agent_did.clone(),
                trust_score: reputation.trust_score,
                delivery_rate: reputation.delivery_rate,
                dispute_rate: reputation.dispute_rate,
                risk_tier: reputation_risk_tier(reputation.trust_score, reputation.dispute_rate),
            });
        }
        reputation_overview.sort_by(|left, right| left.agent_did.cmp(&right.agent_did));

        let mut audit_traces = Vec::with_capacity(audit_log.len());
        for record in audit_log {
            if record.requested_at_unix == 0 {
                return Err(OperatorDashboardUiError::InvalidAuditTimestamp {
                    operator_did: record.operator_did.clone(),
                });
            }

            audit_traces.push(OperatorAuditTraceEntry {
                agent_did: record.agent_did.clone(),
                operator_did: record.operator_did.clone(),
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
pub enum OperatorDashboardUiError {
    EmptyAgentKey {
        agent_did: String,
        key_role: &'static str,
    },
    EmptyMessageRecipients(String),
    InvalidReputationRate {
        agent_did: String,
        field: &'static str,
        value: f64,
    },
    InvalidAuditTimestamp {
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

fn task_attention(state: TaskState) -> DashboardAttentionLevel {
    match state {
        TaskState::Blocked | TaskState::Failed => DashboardAttentionLevel::Critical,
        TaskState::Cancelled | TaskState::Delegated => DashboardAttentionLevel::Warning,
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
