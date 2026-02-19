//! Operator dashboard API contracts for paginated runtime visibility views.

use crate::{
    AgentDid, AgentKeyHierarchy, EscrowLifecycle, EscrowStatus, KeyRole, MessageLifecycleStore,
    MessageStatus, ReputationError, TaskOperationError, TaskOperationRecord, TaskState,
};
use std::collections::BTreeMap;
use std::fmt;

const OPERATOR_DASHBOARD_API_INVALID_AGENT_DID_REASON_CODE: &str =
    "operator_dashboard_api_invalid_agent_did";
const OPERATOR_DASHBOARD_API_INVALID_TASK_REQUESTER_DID_REASON_CODE: &str =
    "operator_dashboard_api_invalid_task_requester_did";
const OPERATOR_DASHBOARD_API_INVALID_TASK_ASSIGNEE_DID_REASON_CODE: &str =
    "operator_dashboard_api_invalid_task_assignee_did";
const OPERATOR_DASHBOARD_API_INVALID_MESSAGE_SENDER_DID_REASON_CODE: &str =
    "operator_dashboard_api_invalid_message_sender_did";
const OPERATOR_DASHBOARD_API_INVALID_MESSAGE_RECIPIENT_DID_REASON_CODE: &str =
    "operator_dashboard_api_invalid_message_recipient_did";
const OPERATOR_DASHBOARD_API_INVALID_ESCROW_PAYER_DID_REASON_CODE: &str =
    "operator_dashboard_api_invalid_escrow_payer_did";
const OPERATOR_DASHBOARD_API_INVALID_ESCROW_PAYEE_DID_REASON_CODE: &str =
    "operator_dashboard_api_invalid_escrow_payee_did";
const OPERATOR_DASHBOARD_API_INVALID_REPUTATION_AGENT_DID_REASON_CODE: &str =
    "operator_dashboard_api_invalid_reputation_agent_did";

#[derive(Debug, Clone, PartialEq, Eq)]
/// Pagination request contract for dashboard list endpoints.
pub struct DashboardPageRequest {
    /// Maximum number of items to return in the page.
    pub limit: usize,
    /// Cursor pointing to the last item of the previous page.
    pub cursor: Option<String>,
    /// Optional prefix filter applied to the item key space.
    pub filter_prefix: Option<String>,
}

impl DashboardPageRequest {
    /// Builds a pagination request and rejects zero limits.
    pub fn new(
        limit: usize,
        cursor: Option<String>,
        filter_prefix: Option<String>,
    ) -> Result<Self, OperatorDashboardApiError> {
        if limit == 0 {
            return Err(OperatorDashboardApiError::InvalidPageLimit(0));
        }
        Ok(Self {
            limit,
            cursor,
            filter_prefix,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Generic paginated response container used by dashboard list APIs.
pub struct DashboardPage<T> {
    /// Items included in the current page.
    pub items: Vec<T>,
    /// Cursor token for fetching the next page.
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Dashboard projection of agent key hierarchy state.
pub struct OperatorAgentView {
    /// Agent DID identifier.
    pub agent_did: String,
    /// Active identity key reference.
    pub identity_key_id: String,
    /// Active signing key reference.
    pub signing_key_id: String,
    /// Active key-agreement key reference.
    pub agreement_key_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Dashboard projection of task lifecycle state.
pub struct OperatorTaskView {
    /// Unique task identifier.
    pub task_id: String,
    /// Task requester DID.
    pub requester: String,
    /// Optional assignee DID.
    pub assignee: Option<String>,
    /// Current task lifecycle state.
    pub state: TaskState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Dashboard projection of message delivery state.
pub struct OperatorMessageView {
    /// Unique message identifier.
    pub message_id: String,
    /// Sender DID.
    pub sender: String,
    /// Recipient DIDs.
    pub recipients: Vec<String>,
    /// Current message status.
    pub status: MessageStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Dashboard projection of escrow contract state.
pub struct OperatorEscrowView {
    /// Escrow record identifier.
    pub escrow_id: String,
    /// Payer DID.
    pub payer: String,
    /// Payee DID.
    pub payee: String,
    /// Current escrow status.
    pub status: EscrowStatus,
    /// Remaining escrow amount in base units.
    pub remaining_amount: u128,
}

#[derive(Debug, Clone, PartialEq)]
/// Dashboard projection of reputation indicators per agent.
pub struct OperatorReputationView {
    /// Agent DID identifier.
    pub agent_did: String,
    /// Trust score scalar.
    pub trust_score: u32,
    /// Delivery success rate percentage.
    pub delivery_rate: f64,
    /// Dispute rate percentage.
    pub dispute_rate: f64,
}

#[derive(Debug, Clone, PartialEq)]
/// Aggregated dashboard snapshot across all operator-visible domains.
pub struct OperatorDashboardSnapshot {
    /// Paginated agent records.
    pub agents: DashboardPage<OperatorAgentView>,
    /// Paginated task records.
    pub tasks: DashboardPage<OperatorTaskView>,
    /// Paginated message records.
    pub messages: DashboardPage<OperatorMessageView>,
    /// Paginated escrow records.
    pub escrows: DashboardPage<OperatorEscrowView>,
    /// Paginated reputation records.
    pub reputation: DashboardPage<OperatorReputationView>,
}

#[derive(Debug, Clone, PartialEq, Default)]
/// In-memory operator dashboard API projection store.
pub struct OperatorDashboardApi {
    agents: BTreeMap<String, OperatorAgentView>,
    tasks: BTreeMap<String, OperatorTaskView>,
    messages: BTreeMap<String, OperatorMessageView>,
    escrows: BTreeMap<String, OperatorEscrowView>,
    reputation: BTreeMap<String, OperatorReputationView>,
}

impl OperatorDashboardApi {
    /// Creates an empty dashboard API projection store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Upserts an agent projection using its current key hierarchy.
    pub fn upsert_agent_from_hierarchy(
        &mut self,
        agent_did: &str,
        hierarchy: &AgentKeyHierarchy,
    ) -> Result<(), OperatorDashboardApiError> {
        let agent_did = parse_agent_did(
            agent_did,
            "agent_did",
            OPERATOR_DASHBOARD_API_INVALID_AGENT_DID_REASON_CODE,
        )?;
        let identity_key_id = hierarchy
            .current_key(KeyRole::Identity)
            .map_err(|error| OperatorDashboardApiError::Hierarchy(error.to_string()))?
            .to_owned();
        let signing_key_id = hierarchy
            .current_key(KeyRole::Signing)
            .map_err(|error| OperatorDashboardApiError::Hierarchy(error.to_string()))?
            .to_owned();
        let agreement_key_id = hierarchy
            .current_key(KeyRole::Agreement)
            .map_err(|error| OperatorDashboardApiError::Hierarchy(error.to_string()))?
            .to_owned();

        self.agents.insert(
            agent_did.as_str().to_owned(),
            OperatorAgentView {
                agent_did: agent_did.as_str().to_owned(),
                identity_key_id,
                signing_key_id,
                agreement_key_id,
            },
        );
        Ok(())
    }

    /// Upserts a task projection from a task operation record.
    pub fn upsert_task(
        &mut self,
        task: &TaskOperationRecord,
    ) -> Result<(), OperatorDashboardApiError> {
        if task.task_id.trim().is_empty() {
            return Err(OperatorDashboardApiError::EmptyField("task_id"));
        }
        let requester = parse_agent_did(
            task.requester.as_str(),
            "task.requester",
            OPERATOR_DASHBOARD_API_INVALID_TASK_REQUESTER_DID_REASON_CODE,
        )?;
        let assignee = task
            .assignee
            .as_deref()
            .map(|value| {
                parse_agent_did(
                    value,
                    "task.assignee",
                    OPERATOR_DASHBOARD_API_INVALID_TASK_ASSIGNEE_DID_REASON_CODE,
                )
                .map(|did| did.as_str().to_owned())
            })
            .transpose()?;
        self.tasks.insert(
            task.task_id.clone(),
            OperatorTaskView {
                task_id: task.task_id.clone(),
                requester: requester.as_str().to_owned(),
                assignee,
                state: task.lifecycle.state(),
            },
        );
        Ok(())
    }

    /// Upserts a message projection from lifecycle store state.
    pub fn upsert_message_from_store(
        &mut self,
        store: &MessageLifecycleStore,
        message_id: &str,
    ) -> Result<(), OperatorDashboardApiError> {
        let status = store
            .status(message_id)
            .map_err(|error| OperatorDashboardApiError::Message(error.to_string()))?;
        let (sender, recipients) = store
            .participants(message_id)
            .map_err(|error| OperatorDashboardApiError::Message(error.to_string()))?;
        let sender = parse_agent_did(
            sender,
            "message.sender",
            OPERATOR_DASHBOARD_API_INVALID_MESSAGE_SENDER_DID_REASON_CODE,
        )?;
        let recipients = recipients
            .iter()
            .map(|recipient| {
                parse_agent_did(
                    recipient,
                    "message.recipients[]",
                    OPERATOR_DASHBOARD_API_INVALID_MESSAGE_RECIPIENT_DID_REASON_CODE,
                )
                .map(|did| did.as_str().to_owned())
            })
            .collect::<Result<Vec<_>, _>>()?;

        self.messages.insert(
            message_id.to_owned(),
            OperatorMessageView {
                message_id: message_id.to_owned(),
                sender: sender.as_str().to_owned(),
                recipients,
                status,
            },
        );
        Ok(())
    }

    /// Upserts an escrow projection from escrow lifecycle state.
    pub fn upsert_escrow(
        &mut self,
        escrow_id: &str,
        payer: &str,
        payee: &str,
        escrow: &EscrowLifecycle,
    ) -> Result<(), OperatorDashboardApiError> {
        if escrow_id.trim().is_empty() {
            return Err(OperatorDashboardApiError::EmptyField("escrow_id"));
        }
        let payer = parse_agent_did(
            payer,
            "escrow.payer",
            OPERATOR_DASHBOARD_API_INVALID_ESCROW_PAYER_DID_REASON_CODE,
        )?;
        let payee = parse_agent_did(
            payee,
            "escrow.payee",
            OPERATOR_DASHBOARD_API_INVALID_ESCROW_PAYEE_DID_REASON_CODE,
        )?;

        self.escrows.insert(
            escrow_id.to_owned(),
            OperatorEscrowView {
                escrow_id: escrow_id.to_owned(),
                payer: payer.as_str().to_owned(),
                payee: payee.as_str().to_owned(),
                status: escrow.status(),
                remaining_amount: escrow.remaining_amount(),
            },
        );
        Ok(())
    }

    /// Upserts a reputation projection from an agent reputation record.
    pub fn upsert_reputation(
        &mut self,
        reputation: &crate::AgentReputation,
    ) -> Result<(), OperatorDashboardApiError> {
        let agent_did = parse_agent_did(
            reputation.agent_did.as_str(),
            "reputation.agent_did",
            OPERATOR_DASHBOARD_API_INVALID_REPUTATION_AGENT_DID_REASON_CODE,
        )?;
        self.reputation.insert(
            agent_did.as_str().to_owned(),
            OperatorReputationView {
                agent_did: agent_did.as_str().to_owned(),
                trust_score: reputation.trust_score,
                delivery_rate: reputation.delivery_rate,
                dispute_rate: reputation.dispute_rate,
            },
        );
        Ok(())
    }

    /// Returns a paginated list of agent projections.
    pub fn list_agents(
        &self,
        request: &DashboardPageRequest,
    ) -> Result<DashboardPage<OperatorAgentView>, OperatorDashboardApiError> {
        paginate_map(&self.agents, request)
    }

    /// Returns a paginated list of task projections.
    pub fn list_tasks(
        &self,
        request: &DashboardPageRequest,
    ) -> Result<DashboardPage<OperatorTaskView>, OperatorDashboardApiError> {
        paginate_map(&self.tasks, request)
    }

    /// Returns a paginated list of message projections.
    pub fn list_messages(
        &self,
        request: &DashboardPageRequest,
    ) -> Result<DashboardPage<OperatorMessageView>, OperatorDashboardApiError> {
        paginate_map(&self.messages, request)
    }

    /// Returns a paginated list of escrow projections.
    pub fn list_escrows(
        &self,
        request: &DashboardPageRequest,
    ) -> Result<DashboardPage<OperatorEscrowView>, OperatorDashboardApiError> {
        paginate_map(&self.escrows, request)
    }

    /// Returns a paginated list of reputation projections.
    pub fn list_reputation(
        &self,
        request: &DashboardPageRequest,
    ) -> Result<DashboardPage<OperatorReputationView>, OperatorDashboardApiError> {
        paginate_map(&self.reputation, request)
    }

    /// Returns a multi-domain dashboard snapshot for a shared page request.
    pub fn snapshot(
        &self,
        request: &DashboardPageRequest,
    ) -> Result<OperatorDashboardSnapshot, OperatorDashboardApiError> {
        Ok(OperatorDashboardSnapshot {
            agents: self.list_agents(request)?,
            tasks: self.list_tasks(request)?,
            messages: self.list_messages(request)?,
            escrows: self.list_escrows(request)?,
            reputation: self.list_reputation(request)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Error taxonomy for dashboard API validation and projection failures.
pub enum OperatorDashboardApiError {
    /// Page limit is invalid.
    InvalidPageLimit(usize),
    /// Cursor does not exist in the current key space.
    InvalidPaginationCursor(String),
    /// Required field is empty.
    EmptyField(&'static str),
    /// DID value is invalid.
    InvalidDid {
        /// Input field carrying the DID value.
        field: &'static str,
        /// Stable reason marker.
        reason_code: &'static str,
        /// Canonical parser detail.
        detail: String,
    },
    /// Agent key hierarchy lookup failed.
    Hierarchy(String),
    /// Message lifecycle lookup failed.
    Message(String),
    /// Task projection conversion failed.
    Task(String),
    /// Reputation projection conversion failed.
    Reputation(String),
}

impl fmt::Display for OperatorDashboardApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPageLimit(limit) => {
                write!(f, "page limit must be positive, found {limit}")
            }
            Self::InvalidPaginationCursor(cursor) => {
                write!(f, "invalid pagination cursor: {cursor}")
            }
            Self::EmptyField(field) => write!(f, "field must not be empty: {field}"),
            Self::InvalidDid {
                field,
                reason_code,
                detail,
            } => write!(f, "invalid did field {field}: {reason_code} ({detail})"),
            Self::Hierarchy(value) => write!(f, "key hierarchy error: {value}"),
            Self::Message(value) => write!(f, "message lifecycle error: {value}"),
            Self::Task(value) => write!(f, "task operation error: {value}"),
            Self::Reputation(value) => write!(f, "reputation error: {value}"),
        }
    }
}

impl std::error::Error for OperatorDashboardApiError {}

impl From<TaskOperationError> for OperatorDashboardApiError {
    fn from(value: TaskOperationError) -> Self {
        Self::Task(value.to_string())
    }
}

impl From<ReputationError> for OperatorDashboardApiError {
    fn from(value: ReputationError) -> Self {
        Self::Reputation(value.to_string())
    }
}

impl OperatorDashboardApiError {
    /// Stable reason taxonomy for dashboard API failures.
    pub fn reason_code(&self) -> &'static str {
        match self {
            Self::InvalidPageLimit(_) => "operator_dashboard_api_invalid_page_limit",
            Self::InvalidPaginationCursor(_) => "operator_dashboard_api_invalid_pagination_cursor",
            Self::EmptyField(_) => "operator_dashboard_api_empty_field",
            Self::InvalidDid { reason_code, .. } => reason_code,
            Self::Hierarchy(_) => "operator_dashboard_api_hierarchy_error",
            Self::Message(_) => "operator_dashboard_api_message_error",
            Self::Task(_) => "operator_dashboard_api_task_error",
            Self::Reputation(_) => "operator_dashboard_api_reputation_error",
        }
    }
}

fn parse_agent_did(
    value: &str,
    field: &'static str,
    reason_code: &'static str,
) -> Result<AgentDid, OperatorDashboardApiError> {
    AgentDid::parse(value).map_err(|error| OperatorDashboardApiError::InvalidDid {
        field,
        reason_code,
        detail: error.to_string(),
    })
}

fn paginate_map<T: Clone>(
    values: &BTreeMap<String, T>,
    request: &DashboardPageRequest,
) -> Result<DashboardPage<T>, OperatorDashboardApiError> {
    if request.limit == 0 {
        return Err(OperatorDashboardApiError::InvalidPageLimit(0));
    }

    let mut keys: Vec<String> = values.keys().cloned().collect();
    if let Some(prefix) = request.filter_prefix.as_deref() {
        keys.retain(|key| key.starts_with(prefix));
    }

    let start_idx = if let Some(cursor) = request.cursor.as_deref() {
        let Some(position) = keys.iter().position(|key| key == cursor) else {
            return Err(OperatorDashboardApiError::InvalidPaginationCursor(
                cursor.to_owned(),
            ));
        };
        position + 1
    } else {
        0
    };

    let page_keys: Vec<String> = keys
        .iter()
        .skip(start_idx)
        .take(request.limit)
        .cloned()
        .collect();
    let items = page_keys
        .iter()
        .filter_map(|key| values.get(key).cloned())
        .collect();
    let next_cursor = if start_idx + page_keys.len() < keys.len() {
        page_keys.last().cloned()
    } else {
        None
    };

    Ok(DashboardPage { items, next_cursor })
}

#[cfg(test)]
mod tests {
    use super::{DashboardPageRequest, OperatorDashboardApiError};

    #[test]
    fn request_rejects_zero_limit() {
        assert_eq!(
            DashboardPageRequest::new(0, None, None),
            Err(OperatorDashboardApiError::InvalidPageLimit(0))
        );
    }
}
