use super::super::models::*;
use crate::{data_layer_hashing::tagged_sha256, ContentRetentionClass};

pub(crate) struct DataLayerM5RecordHashMaterial<'a> {
    pub embedding_id: &'a str,
    pub message_id: &'a str,
    pub owner_did: &'a str,
    pub agent_did: &'a str,
    pub retention_class: ContentRetentionClass,
    pub model_id: &'a str,
    pub vector_encrypted: &'a [u8],
    pub vector_plaintext: Option<&'a [f32]>,
    pub vector_dimensions: usize,
    pub created_at_epoch_seconds: u64,
    pub privacy_mode: DataLayerM5EmbeddingPrivacyMode,
}

pub(crate) fn compute_embedding_record_hash(
    sequence: u64,
    material: &DataLayerM5RecordHashMaterial<'_>,
    hash_chain_prev: &str,
) -> String {
    tagged_digest(
        format!(
            "m5-embedding|seq:{sequence}|embedding:{embedding_id}|message:{message_id}|owner:{owner_did}|agent:{agent_did}|retention:{retention_class}|model:{model_id}|encrypted:{}|plaintext:{}|dims:{vector_dimensions}|created:{created_at_epoch_seconds}|mode:{}|metric:{}|prev:{hash_chain_prev}",
            bytes_marker(material.vector_encrypted),
            vector_marker(material.vector_plaintext),
            material.privacy_mode.marker(),
            DATA_LAYER_M5_VECTOR_DISTANCE_METRIC_COSINE,
            embedding_id = material.embedding_id,
            message_id = material.message_id,
            owner_did = material.owner_did,
            agent_did = material.agent_did,
            retention_class = retention_class_marker(material.retention_class),
            model_id = material.model_id,
            vector_dimensions = material.vector_dimensions,
            created_at_epoch_seconds = material.created_at_epoch_seconds
        )
        .as_str(),
    )
}

fn retention_class_marker(class: ContentRetentionClass) -> &'static str {
    match class {
        ContentRetentionClass::ShortLived => "short_lived",
        ContentRetentionClass::Standard => "standard",
        ContentRetentionClass::Compliance => "compliance",
    }
}

fn bytes_marker(value: &[u8]) -> String {
    if value.is_empty() {
        return "none".to_owned();
    }
    value
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join("")
}

fn vector_marker(value: Option<&[f32]>) -> String {
    match value {
        Some(vector) => vector
            .iter()
            .map(|coordinate| format!("{coordinate:.8}"))
            .collect::<Vec<_>>()
            .join(","),
        None => "none".to_owned(),
    }
}

fn tagged_digest(value: &str) -> String {
    tagged_sha256(value, DATA_LAYER_M5_HASH_ALGORITHM)
}
