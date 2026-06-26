use std::collections::VecDeque;

use kamn_core::{
    DataLayerM1AnchoringOrchestrator, DataLayerM1BatchSchedulerPolicy,
    DataLayerM1PendingBatchMessage, InMemoryKolmeRuntimeCommitClient, KolmeRuntimeCommitClient,
    KolmeRuntimeCommitError, KolmeRuntimeCommitOutcome, KolmeRuntimeCommitRequest,
};

pub fn pending(
    message_id: &str,
    content_hash: &str,
    created_at_unix_seconds: u64,
) -> DataLayerM1PendingBatchMessage {
    DataLayerM1PendingBatchMessage {
        message_id: message_id.to_owned(),
        content_hash: content_hash.to_owned(),
        created_at_unix_seconds,
    }
}

#[derive(Debug, Clone)]
pub struct ScriptedKolmeRuntimeCommitClient {
    scripted_outcomes: VecDeque<KolmeRuntimeCommitOutcome>,
}

impl ScriptedKolmeRuntimeCommitClient {
    pub fn new(scripted_outcomes: Vec<KolmeRuntimeCommitOutcome>) -> Self {
        Self {
            scripted_outcomes: scripted_outcomes.into(),
        }
    }
}

impl KolmeRuntimeCommitClient for ScriptedKolmeRuntimeCommitClient {
    fn submit_commit(
        &mut self,
        request: &KolmeRuntimeCommitRequest,
    ) -> Result<KolmeRuntimeCommitOutcome, KolmeRuntimeCommitError> {
        request.validate()?;
        Ok(self
            .scripted_outcomes
            .pop_front()
            .unwrap_or(KolmeRuntimeCommitOutcome::Rejected {
                reason: "scripted_outcome_exhausted".to_owned(),
            }))
    }
}

pub fn memory_orchestrator(
    agent_did: &str,
    batch_min_messages: usize,
) -> DataLayerM1AnchoringOrchestrator<InMemoryKolmeRuntimeCommitClient> {
    let client = InMemoryKolmeRuntimeCommitClient::new("kolme-memory")
        .expect("in-memory client should initialize");
    let policy = DataLayerM1BatchSchedulerPolicy::new(batch_min_messages, 60)
        .expect("policy should be valid");
    DataLayerM1AnchoringOrchestrator::new(client, agent_did, "m1-root", policy)
        .expect("orchestrator should initialize")
}

pub fn scripted_orchestrator(
    agent_did: &str,
    scripted_outcomes: Vec<KolmeRuntimeCommitOutcome>,
) -> DataLayerM1AnchoringOrchestrator<ScriptedKolmeRuntimeCommitClient> {
    let client = ScriptedKolmeRuntimeCommitClient::new(scripted_outcomes);
    let policy = DataLayerM1BatchSchedulerPolicy::new(1, 60).expect("policy should be valid");
    DataLayerM1AnchoringOrchestrator::new(client, agent_did, "m1-root", policy)
        .expect("orchestrator should initialize")
}
