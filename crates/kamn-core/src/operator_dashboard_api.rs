use crate::{
    AgentDid, AgentKeyHierarchy, EscrowLifecycle, EscrowStatus, KeyRole, MessageLifecycleStore,
    MessageStatus, ReputationError, TaskOperationError, TaskOperationRecord, TaskState,
};
use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DashboardPageRequest {
    pub limit: usize,
    pub cursor: Option<String>,
    pub filter_prefix: Option<String>,
}

impl DashboardPageRequest {
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
pub struct DashboardPage<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatorAgentView {
    pub agent_did: String,
    pub identity_key_id: String,
    pub signing_key_id: String,
    pub agreement_key_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatorTaskView {
    pub task_id: String,
    pub requester: String,
    pub assignee: Option<String>,
    pub state: TaskState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatorMessageView {
    pub message_id: String,
    pub sender: String,
    pub recipients: Vec<String>,
    pub status: MessageStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OperatorEscrowView {
    pub escrow_id: String,
    pub payer: String,
    pub payee: String,
    pub status: EscrowStatus,
    pub remaining_amount: u128,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OperatorReputationView {
    pub agent_did: String,
    pub trust_score: u32,
    pub delivery_rate: f64,
    pub dispute_rate: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OperatorDashboardSnapshot {
    pub agents: DashboardPage<OperatorAgentView>,
    pub tasks: DashboardPage<OperatorTaskView>,
    pub messages: DashboardPage<OperatorMessageView>,
    pub escrows: DashboardPage<OperatorEscrowView>,
    pub reputation: DashboardPage<OperatorReputationView>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct OperatorDashboardApi {
    agents: BTreeMap<String, OperatorAgentView>,
    tasks: BTreeMap<String, OperatorTaskView>,
    messages: BTreeMap<String, OperatorMessageView>,
    escrows: BTreeMap<String, OperatorEscrowView>,
    reputation: BTreeMap<String, OperatorReputationView>,
}

impl OperatorDashboardApi {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn upsert_agent_from_hierarchy(
        &mut self,
        agent_did: &str,
        hierarchy: &AgentKeyHierarchy,
    ) -> Result<(), OperatorDashboardApiError> {
        validate_did(agent_did)?;
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
            agent_did.to_owned(),
            OperatorAgentView {
                agent_did: agent_did.to_owned(),
                identity_key_id,
                signing_key_id,
                agreement_key_id,
            },
        );
        Ok(())
    }

    pub fn upsert_task(
        &mut self,
        task: &TaskOperationRecord,
    ) -> Result<(), OperatorDashboardApiError> {
        if task.task_id.trim().is_empty() {
            return Err(OperatorDashboardApiError::EmptyField("task_id"));
        }
        validate_did(&task.requester)?;
        if let Some(assignee) = task.assignee.as_deref() {
            validate_did(assignee)?;
        }
        self.tasks.insert(
            task.task_id.clone(),
            OperatorTaskView {
                task_id: task.task_id.clone(),
                requester: task.requester.clone(),
                assignee: task.assignee.clone(),
                state: task.lifecycle.state(),
            },
        );
        Ok(())
    }

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
        validate_did(sender)?;
        for recipient in recipients {
            validate_did(recipient)?;
        }

        self.messages.insert(
            message_id.to_owned(),
            OperatorMessageView {
                message_id: message_id.to_owned(),
                sender: sender.to_owned(),
                recipients: recipients.to_vec(),
                status,
            },
        );
        Ok(())
    }

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
        validate_did(payer)?;
        validate_did(payee)?;

        self.escrows.insert(
            escrow_id.to_owned(),
            OperatorEscrowView {
                escrow_id: escrow_id.to_owned(),
                payer: payer.to_owned(),
                payee: payee.to_owned(),
                status: escrow.status(),
                remaining_amount: escrow.remaining_amount(),
            },
        );
        Ok(())
    }

    pub fn upsert_reputation(
        &mut self,
        reputation: &crate::AgentReputation,
    ) -> Result<(), OperatorDashboardApiError> {
        validate_did(&reputation.agent_did)?;
        self.reputation.insert(
            reputation.agent_did.clone(),
            OperatorReputationView {
                agent_did: reputation.agent_did.clone(),
                trust_score: reputation.trust_score,
                delivery_rate: reputation.delivery_rate,
                dispute_rate: reputation.dispute_rate,
            },
        );
        Ok(())
    }

    pub fn list_agents(
        &self,
        request: &DashboardPageRequest,
    ) -> Result<DashboardPage<OperatorAgentView>, OperatorDashboardApiError> {
        paginate_map(&self.agents, request)
    }

    pub fn list_tasks(
        &self,
        request: &DashboardPageRequest,
    ) -> Result<DashboardPage<OperatorTaskView>, OperatorDashboardApiError> {
        paginate_map(&self.tasks, request)
    }

    pub fn list_messages(
        &self,
        request: &DashboardPageRequest,
    ) -> Result<DashboardPage<OperatorMessageView>, OperatorDashboardApiError> {
        paginate_map(&self.messages, request)
    }

    pub fn list_escrows(
        &self,
        request: &DashboardPageRequest,
    ) -> Result<DashboardPage<OperatorEscrowView>, OperatorDashboardApiError> {
        paginate_map(&self.escrows, request)
    }

    pub fn list_reputation(
        &self,
        request: &DashboardPageRequest,
    ) -> Result<DashboardPage<OperatorReputationView>, OperatorDashboardApiError> {
        paginate_map(&self.reputation, request)
    }

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
pub enum OperatorDashboardApiError {
    InvalidPageLimit(usize),
    InvalidPaginationCursor(String),
    EmptyField(&'static str),
    InvalidDid(String),
    Hierarchy(String),
    Message(String),
    Task(String),
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
            Self::InvalidDid(value) => write!(f, "invalid did: {value}"),
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

fn validate_did(value: &str) -> Result<(), OperatorDashboardApiError> {
    AgentDid::parse(value)
        .map_err(|error| OperatorDashboardApiError::InvalidDid(error.to_string()))?;
    Ok(())
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
