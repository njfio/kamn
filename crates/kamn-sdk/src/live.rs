use crate::{
    AgentDid, AgentMetadata, AgentQuery, AgentReputation, AgentSummary, Artifact, ArtifactId,
    DidDocument, EscrowConfig, EscrowId, InMemoryKamnClient, KamnAgent, KamnTransport, Message,
    MessageId, MessageRecord, MessageStream, SdkError, TaskDefinition, TaskId, TokenAmount,
    TransportMode,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveTransportConfig {
    pub endpoint: String,
}

impl LiveTransportConfig {
    pub fn new(endpoint: &str) -> Result<Self, SdkError> {
        let normalized = endpoint.trim().to_ascii_lowercase();
        if !(normalized.starts_with("https://") || normalized.starts_with("wss://")) {
            return Err(SdkError::InvalidInput {
                field: "transport.endpoint",
                reason: "must start with https:// or wss://",
            });
        }
        if endpoint.trim().len() < "https://a".len() {
            return Err(SdkError::InvalidInput {
                field: "transport.endpoint",
                reason: "must include host information",
            });
        }

        Ok(Self {
            endpoint: endpoint.to_owned(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct LiveTransportKamnClient {
    config: LiveTransportConfig,
    shared_state: Arc<Mutex<InMemoryKamnClient>>,
}

impl LiveTransportKamnClient {
    pub fn connect(endpoint: &str) -> Result<Self, SdkError> {
        let config = LiveTransportConfig::new(endpoint)?;
        let shared_state = resolve_live_transport_state(&config.endpoint)?;
        Ok(Self {
            config,
            shared_state,
        })
    }

    pub fn endpoint(&self) -> &str {
        &self.config.endpoint
    }

    fn with_client<T>(
        &self,
        callback: impl FnOnce(&InMemoryKamnClient) -> Result<T, SdkError>,
    ) -> Result<T, SdkError> {
        let guard = self
            .shared_state
            .lock()
            .map_err(|_| SdkError::TransportFailure("live transport state lock poisoned"))?;
        callback(&guard)
    }

    fn with_client_mut<T>(
        &self,
        callback: impl FnOnce(&mut InMemoryKamnClient) -> Result<T, SdkError>,
    ) -> Result<T, SdkError> {
        let mut guard = self
            .shared_state
            .lock()
            .map_err(|_| SdkError::TransportFailure("live transport state lock poisoned"))?;
        callback(&mut guard)
    }
}

fn resolve_live_transport_state(
    endpoint: &str,
) -> Result<Arc<Mutex<InMemoryKamnClient>>, SdkError> {
    let registry = live_transport_registry();
    let mut locked_registry = registry
        .lock()
        .map_err(|_| SdkError::TransportFailure("live transport registry lock poisoned"))?;

    Ok(locked_registry
        .entry(endpoint.to_owned())
        .or_insert_with(|| Arc::new(Mutex::new(InMemoryKamnClient::new())))
        .clone())
}

fn live_transport_registry() -> &'static Mutex<HashMap<String, Arc<Mutex<InMemoryKamnClient>>>> {
    static REGISTRY: OnceLock<Mutex<HashMap<String, Arc<Mutex<InMemoryKamnClient>>>>> =
        OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

impl KamnTransport for LiveTransportKamnClient {
    fn transport_mode(&self) -> TransportMode {
        TransportMode::Live
    }
}

impl KamnAgent for LiveTransportKamnClient {
    fn register(&mut self, metadata: AgentMetadata) -> Result<AgentDid, SdkError> {
        self.with_client_mut(move |client| client.register(metadata))
    }

    fn resolve(&self, did: &AgentDid) -> Result<DidDocument, SdkError> {
        self.with_client(|client| client.resolve(did))
    }

    fn send(&mut self, message: Message) -> Result<MessageId, SdkError> {
        self.with_client_mut(move |client| client.send(message))
    }

    fn receive(&mut self, did: &AgentDid) -> Result<Vec<MessageRecord>, SdkError> {
        self.with_client_mut(move |client| client.receive(did))
    }

    fn receive_stream(&mut self, did: &AgentDid) -> Result<MessageStream, SdkError> {
        self.with_client_mut(move |client| client.receive_stream(did))
    }

    fn create_task(&mut self, task: TaskDefinition) -> Result<TaskId, SdkError> {
        self.with_client_mut(move |client| client.create_task(task))
    }

    fn accept_task(&mut self, task_id: &TaskId, assignee: &AgentDid) -> Result<(), SdkError> {
        self.with_client_mut(move |client| client.accept_task(task_id, assignee))
    }

    fn submit_artifact(
        &mut self,
        task_id: &TaskId,
        artifact: Artifact,
    ) -> Result<ArtifactId, SdkError> {
        self.with_client_mut(move |client| client.submit_artifact(task_id, artifact))
    }

    fn complete_task(&mut self, task_id: &TaskId) -> Result<(), SdkError> {
        self.with_client_mut(move |client| client.complete_task(task_id))
    }

    fn create_escrow(&mut self, escrow: EscrowConfig) -> Result<EscrowId, SdkError> {
        self.with_client_mut(move |client| client.create_escrow(escrow))
    }

    fn release_escrow(&mut self, escrow_id: &EscrowId) -> Result<(), SdkError> {
        self.with_client_mut(move |client| client.release_escrow(escrow_id))
    }

    fn balance(&self, did: &AgentDid) -> Result<TokenAmount, SdkError> {
        self.with_client(|client| client.balance(did))
    }

    fn search_agents(&self, query: AgentQuery) -> Result<Vec<AgentSummary>, SdkError> {
        self.with_client(move |client| client.search_agents(query))
    }

    fn get_reputation(&self, agent: &AgentDid) -> Result<AgentReputation, SdkError> {
        self.with_client(|client| client.get_reputation(agent))
    }
}
