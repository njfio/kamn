use crate::{ContentRetentionClass, KamnDid};
use std::collections::{BTreeMap, BTreeSet};

/// Embedding storage/privacy mode contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataLayerM5EmbeddingPrivacyMode {
    /// Embeddings are stored encrypted only; semantic and anomaly operations run owner-side.
    OwnerSideEncrypted,
    /// Plaintext vectors are opt-in for server-side semantic and anomaly operations.
    ServerSidePlaintextOptIn,
}

impl DataLayerM5EmbeddingPrivacyMode {
    pub(crate) fn marker(self) -> &'static str {
        match self {
            Self::OwnerSideEncrypted => "owner-side-encrypted",
            Self::ServerSidePlaintextOptIn => "server-side-plaintext-opt-in",
        }
    }
}

/// Input payload for registering one embedding record.
#[derive(Debug, Clone, PartialEq)]
pub struct DataLayerM5EmbeddingRecordInput {
    pub embedding_id: String,
    pub message_id: String,
    pub owner_did: String,
    pub agent_did: String,
    pub retention_class: ContentRetentionClass,
    pub model_id: String,
    pub vector_encrypted: Vec<u8>,
    pub vector_plaintext: Option<Vec<f32>>,
    pub created_at_epoch_seconds: u64,
}

/// Stored append-only embedding record.
#[derive(Debug, Clone, PartialEq)]
pub struct DataLayerM5EmbeddingRecord {
    pub embedding_id: String,
    pub message_id: String,
    pub owner_did: String,
    pub agent_did: String,
    pub retention_class: ContentRetentionClass,
    pub model_id: String,
    pub vector_encrypted: Vec<u8>,
    pub privacy_mode: DataLayerM5EmbeddingPrivacyMode,
    pub vector_plaintext: Option<Vec<f32>>,
    pub vector_dimensions: usize,
    pub sequence: u64,
    pub created_at_epoch_seconds: u64,
    pub hash_chain_prev: String,
    pub record_hash: String,
}

/// Semantic query input contract.
#[derive(Debug, Clone, PartialEq)]
pub struct DataLayerM5SemanticQuery {
    pub owner_did: String,
    pub query_vector: Vec<f32>,
    pub limit: Option<usize>,
}

/// One semantic query result row.
#[derive(Debug, Clone, PartialEq)]
pub struct DataLayerM5SemanticQueryResult {
    pub embedding_id: String,
    pub message_id: String,
    pub similarity_score: f32,
    pub cosine_distance: f32,
}

/// M5 embedding registry, semantic query, and anomaly evaluation engine.
#[derive(Debug, Clone, PartialEq)]
pub struct DataLayerM5EmbeddingRegistry {
    pub(crate) privacy_mode: DataLayerM5EmbeddingPrivacyMode,
    pub(crate) records_by_owner: BTreeMap<String, Vec<DataLayerM5EmbeddingRecord>>,
    pub(crate) seen_embedding_ids: BTreeSet<String>,
}

impl DataLayerM5EmbeddingRegistry {
    /// Creates an empty registry for one privacy mode.
    pub fn new(privacy_mode: DataLayerM5EmbeddingPrivacyMode) -> Self {
        Self {
            privacy_mode,
            records_by_owner: BTreeMap::new(),
            seen_embedding_ids: BTreeSet::new(),
        }
    }

    /// Returns the configured privacy mode.
    pub fn privacy_mode(&self) -> DataLayerM5EmbeddingPrivacyMode {
        self.privacy_mode
    }

    /// Returns embedding records for one owner scope in append order.
    pub fn embedding_records(&self, owner_did: &str) -> Option<&[DataLayerM5EmbeddingRecord]> {
        let owner_did = KamnDid::parse(owner_did).ok()?;
        self.records_by_owner
            .get(owner_did.as_str())
            .map(Vec::as_slice)
    }
}
