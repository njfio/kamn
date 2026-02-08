use crate::{
    AgentDid, AgentMetadata, AgentQuery, AgentReputation, AgentSummary, Artifact, ArtifactId,
    DidDocument, EscrowConfig, EscrowId, KamnAgent, Message, MessageId, MessageRecord, SdkError,
    TaskDefinition, TaskId, TokenAmount,
};
use std::collections::HashMap;

const INITIAL_AGENT_BALANCE: u64 = 100;
const INITIAL_REPUTATION_SCORE: u32 = 500;

#[derive(Debug, Clone, PartialEq, Eq)]
struct TaskState {
    definition: TaskDefinition,
    accepted_by: Option<AgentDid>,
    completed: bool,
    artifacts: Vec<ArtifactId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EscrowState {
    config: EscrowConfig,
    released: bool,
}

#[derive(Debug, Default)]
pub struct InMemoryKamnClient {
    next_agent_id: u64,
    next_message_id: u64,
    next_task_id: u64,
    next_artifact_id: u64,
    next_escrow_id: u64,
    registry: HashMap<AgentDid, DidDocument>,
    inboxes: HashMap<AgentDid, Vec<MessageRecord>>,
    tasks: HashMap<TaskId, TaskState>,
    artifacts: HashMap<ArtifactId, Artifact>,
    escrows: HashMap<EscrowId, EscrowState>,
    balances: HashMap<AgentDid, TokenAmount>,
    reputations: HashMap<AgentDid, AgentReputation>,
}

impl InMemoryKamnClient {
    pub fn new() -> Self {
        Self {
            next_agent_id: 1,
            next_message_id: 1,
            next_task_id: 1,
            next_artifact_id: 1,
            next_escrow_id: 1,
            registry: HashMap::new(),
            inboxes: HashMap::new(),
            tasks: HashMap::new(),
            artifacts: HashMap::new(),
            escrows: HashMap::new(),
            balances: HashMap::new(),
            reputations: HashMap::new(),
        }
    }

    fn ensure_registered(&self, did: &AgentDid) -> Result<(), SdkError> {
        if self.registry.contains_key(did) {
            return Ok(());
        }
        Err(SdkError::NotFound {
            entity: "agent",
            id: did.to_string(),
        })
    }

    fn next_did(&mut self) -> Result<AgentDid, SdkError> {
        let did = AgentDid::parse(format!("kamn:did:agent:agent-{}", self.next_agent_id))?;
        self.next_agent_id += 1;
        Ok(did)
    }

    fn validate_metadata(metadata: &AgentMetadata) -> Result<(), SdkError> {
        if metadata.agent_type.trim().is_empty() {
            return Err(SdkError::InvalidInput {
                field: "agent_type",
                reason: "must not be empty",
            });
        }
        if metadata.model_family.trim().is_empty() {
            return Err(SdkError::InvalidInput {
                field: "model_family",
                reason: "must not be empty",
            });
        }
        if metadata.capabilities.is_empty() {
            return Err(SdkError::InvalidInput {
                field: "capabilities",
                reason: "must include at least one capability",
            });
        }
        if metadata
            .capabilities
            .iter()
            .any(|value| value.trim().is_empty())
        {
            return Err(SdkError::InvalidInput {
                field: "capabilities",
                reason: "must not include empty capability entries",
            });
        }
        Ok(())
    }

    fn validate_message(message: &Message) -> Result<(), SdkError> {
        if message.body.trim().is_empty() {
            return Err(SdkError::InvalidInput {
                field: "body",
                reason: "message body must not be empty",
            });
        }
        Ok(())
    }

    fn validate_task(task: &TaskDefinition) -> Result<(), SdkError> {
        if task.task_type.trim().is_empty() {
            return Err(SdkError::InvalidInput {
                field: "task_type",
                reason: "must not be empty",
            });
        }
        if task.description.trim().is_empty() {
            return Err(SdkError::InvalidInput {
                field: "description",
                reason: "must not be empty",
            });
        }
        Ok(())
    }

    fn validate_artifact(artifact: &Artifact) -> Result<(), SdkError> {
        if artifact.name.trim().is_empty() {
            return Err(SdkError::InvalidInput {
                field: "artifact.name",
                reason: "must not be empty",
            });
        }
        if artifact.bytes.is_empty() {
            return Err(SdkError::InvalidInput {
                field: "artifact.bytes",
                reason: "must not be empty",
            });
        }
        Ok(())
    }
}

impl KamnAgent for InMemoryKamnClient {
    fn register(&mut self, metadata: AgentMetadata) -> Result<AgentDid, SdkError> {
        Self::validate_metadata(&metadata)?;
        let did = self.next_did()?;
        let document = DidDocument {
            id: did.clone(),
            metadata,
            service_endpoint: format!("kamn://messaging/{}", did.as_str()),
        };
        self.registry.insert(did.clone(), document);
        self.inboxes.insert(did.clone(), Vec::new());
        self.balances
            .insert(did.clone(), TokenAmount(INITIAL_AGENT_BALANCE));
        self.reputations.insert(
            did.clone(),
            AgentReputation {
                did: did.clone(),
                score: INITIAL_REPUTATION_SCORE,
            },
        );
        Ok(did)
    }

    fn resolve(&self, did: &AgentDid) -> Result<DidDocument, SdkError> {
        self.registry
            .get(did)
            .cloned()
            .ok_or_else(|| SdkError::NotFound {
                entity: "agent",
                id: did.to_string(),
            })
    }

    fn send(&mut self, message: Message) -> Result<MessageId, SdkError> {
        Self::validate_message(&message)?;
        self.ensure_registered(&message.from)?;
        self.ensure_registered(&message.to)?;

        let message_id = MessageId(self.next_message_id);
        self.next_message_id += 1;

        let record = MessageRecord {
            id: message_id.clone(),
            message,
        };
        if let Some(inbox) = self.inboxes.get_mut(&record.message.to) {
            inbox.push(record);
            return Ok(message_id);
        }
        Err(SdkError::NotFound {
            entity: "inbox",
            id: "recipient".to_owned(),
        })
    }

    fn receive(&mut self, did: &AgentDid) -> Result<Vec<MessageRecord>, SdkError> {
        self.ensure_registered(did)?;
        let inbox = self
            .inboxes
            .get_mut(did)
            .ok_or_else(|| SdkError::NotFound {
                entity: "inbox",
                id: did.to_string(),
            })?;
        Ok(std::mem::take(inbox))
    }

    fn create_task(&mut self, task: TaskDefinition) -> Result<TaskId, SdkError> {
        self.ensure_registered(&task.creator)?;
        Self::validate_task(&task)?;

        let task_id = TaskId(self.next_task_id);
        self.next_task_id += 1;
        self.tasks.insert(
            task_id.clone(),
            TaskState {
                definition: task,
                accepted_by: None,
                completed: false,
                artifacts: Vec::new(),
            },
        );
        Ok(task_id)
    }

    fn accept_task(&mut self, task_id: &TaskId, assignee: &AgentDid) -> Result<(), SdkError> {
        self.ensure_registered(assignee)?;
        let task = self
            .tasks
            .get_mut(task_id)
            .ok_or_else(|| SdkError::NotFound {
                entity: "task",
                id: task_id.0.to_string(),
            })?;

        if task.completed {
            return Err(SdkError::Conflict("task already completed"));
        }
        if task.accepted_by.is_some() {
            return Err(SdkError::Conflict("task already accepted"));
        }
        task.accepted_by = Some(assignee.clone());
        Ok(())
    }

    fn submit_artifact(
        &mut self,
        task_id: &TaskId,
        artifact: Artifact,
    ) -> Result<ArtifactId, SdkError> {
        Self::validate_artifact(&artifact)?;
        let task = self
            .tasks
            .get_mut(task_id)
            .ok_or_else(|| SdkError::NotFound {
                entity: "task",
                id: task_id.0.to_string(),
            })?;
        if task.accepted_by.is_none() {
            return Err(SdkError::Conflict(
                "task must be accepted before artifact submission",
            ));
        }
        if task.completed {
            return Err(SdkError::Conflict("task already completed"));
        }

        let artifact_id = ArtifactId(self.next_artifact_id);
        self.next_artifact_id += 1;
        task.artifacts.push(artifact_id.clone());
        self.artifacts.insert(artifact_id.clone(), artifact);
        Ok(artifact_id)
    }

    fn complete_task(&mut self, task_id: &TaskId) -> Result<(), SdkError> {
        let task = self
            .tasks
            .get_mut(task_id)
            .ok_or_else(|| SdkError::NotFound {
                entity: "task",
                id: task_id.0.to_string(),
            })?;
        if task.accepted_by.is_none() {
            return Err(SdkError::Conflict(
                "task must be accepted before completion",
            ));
        }
        if task.completed {
            return Err(SdkError::Conflict("task already completed"));
        }
        task.completed = true;
        Ok(())
    }

    fn create_escrow(&mut self, escrow: EscrowConfig) -> Result<EscrowId, SdkError> {
        self.ensure_registered(&escrow.payer)?;
        self.ensure_registered(&escrow.payee)?;
        if escrow.amount.0 == 0 {
            return Err(SdkError::InvalidInput {
                field: "escrow.amount",
                reason: "must be greater than zero",
            });
        }

        let payer_balance =
            self.balances
                .get_mut(&escrow.payer)
                .ok_or_else(|| SdkError::NotFound {
                    entity: "balance",
                    id: escrow.payer.to_string(),
                })?;
        if payer_balance.0 < escrow.amount.0 {
            return Err(SdkError::InsufficientFunds {
                available: payer_balance.0,
                required: escrow.amount.0,
            });
        }
        payer_balance.0 -= escrow.amount.0;

        let escrow_id = EscrowId(self.next_escrow_id);
        self.next_escrow_id += 1;
        self.escrows.insert(
            escrow_id.clone(),
            EscrowState {
                config: escrow,
                released: false,
            },
        );
        Ok(escrow_id)
    }

    fn release_escrow(&mut self, escrow_id: &EscrowId) -> Result<(), SdkError> {
        let escrow = self
            .escrows
            .get_mut(escrow_id)
            .ok_or_else(|| SdkError::NotFound {
                entity: "escrow",
                id: escrow_id.0.to_string(),
            })?;
        if escrow.released {
            return Err(SdkError::Conflict("escrow already released"));
        }

        let payee_balance =
            self.balances
                .get_mut(&escrow.config.payee)
                .ok_or_else(|| SdkError::NotFound {
                    entity: "balance",
                    id: escrow.config.payee.to_string(),
                })?;
        payee_balance.0 += escrow.config.amount.0;
        escrow.released = true;
        Ok(())
    }

    fn balance(&self, did: &AgentDid) -> Result<TokenAmount, SdkError> {
        self.balances
            .get(did)
            .copied()
            .ok_or_else(|| SdkError::NotFound {
                entity: "balance",
                id: did.to_string(),
            })
    }

    fn search_agents(&self, query: AgentQuery) -> Result<Vec<AgentSummary>, SdkError> {
        let mut results: Vec<AgentSummary> = self
            .registry
            .values()
            .filter(|document| match &query.model_family {
                Some(model) => document.metadata.model_family == *model,
                None => true,
            })
            .filter(|document| match &query.capability {
                Some(capability) => document
                    .metadata
                    .capabilities
                    .iter()
                    .any(|value| value == capability),
                None => true,
            })
            .map(|document| AgentSummary {
                did: document.id.clone(),
                agent_type: document.metadata.agent_type.clone(),
                model_family: document.metadata.model_family.clone(),
                capabilities: document.metadata.capabilities.clone(),
            })
            .collect();

        results.sort_by(|left, right| left.did.as_str().cmp(right.did.as_str()));
        Ok(results)
    }

    fn get_reputation(&self, agent: &AgentDid) -> Result<AgentReputation, SdkError> {
        self.reputations
            .get(agent)
            .cloned()
            .ok_or_else(|| SdkError::NotFound {
                entity: "reputation",
                id: agent.to_string(),
            })
    }
}
