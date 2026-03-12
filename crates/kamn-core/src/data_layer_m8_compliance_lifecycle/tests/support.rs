use crate::data_layer_m8_compliance_lifecycle::{
    DataLayerM8ComplianceRegistry, DataLayerM8MessageRecordInput, DataLayerM8RetentionClass,
    DataLayerM8WrappedCekInput,
};

pub(super) const OWNER_DID: &str = "kamn:did:owner:owner-6035";

pub(super) fn wrapped_key(recipient_did: &str, wrapped_cek: &str) -> DataLayerM8WrappedCekInput {
    DataLayerM8WrappedCekInput {
        recipient_did: recipient_did.to_owned(),
        wrapped_cek: wrapped_cek.to_owned(),
    }
}

pub(super) fn register_message(
    registry: &mut DataLayerM8ComplianceRegistry,
    message_id: &str,
    created_at_epoch_seconds: u64,
    retention_class: DataLayerM8RetentionClass,
) {
    registry
        .register_message(DataLayerM8MessageRecordInput {
            owner_did: OWNER_DID.to_owned(),
            message_id: message_id.to_owned(),
            created_at_epoch_seconds,
            content_hash: format!("sha256:{message_id}"),
            hash_chain_prev: "sha256:prev".to_owned(),
            retention_class,
            retention_extension_seconds: 0,
            wrapped_keys: vec![
                wrapped_key("kamn:did:agent:alice", "cek-alice"),
                wrapped_key("kamn:did:agent:bob", "cek-bob"),
            ],
        })
        .expect("fixture message registration must succeed");
}
