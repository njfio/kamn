use kamn_core::{
    ContentRetentionClass, DataLayerM5EmbeddingRecordInput,
};

pub fn vector_input(
    embedding_id: &str,
    message_id: &str,
    owner_did: &str,
    agent_did: &str,
    vector_plaintext: Option<Vec<f32>>,
) -> DataLayerM5EmbeddingRecordInput {
    DataLayerM5EmbeddingRecordInput {
        embedding_id: embedding_id.to_owned(),
        message_id: message_id.to_owned(),
        owner_did: owner_did.to_owned(),
        agent_did: agent_did.to_owned(),
        retention_class: ContentRetentionClass::Standard,
        model_id: "text-embedding-3-large".to_owned(),
        vector_encrypted: vec![0xde, 0xad, 0xbe, 0xef],
        vector_plaintext,
        created_at_epoch_seconds: 1_708_300_000,
    }
}
