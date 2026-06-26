use crate::data_layer_m8_compliance_lifecycle::{
    DataLayerM8ComplianceRegistry, DataLayerM8MessageRecordInput, DataLayerM8OwnerScopeQuery,
    DataLayerM8RetentionClass, DATA_LAYER_M8_RETENTION_DUE_REASON_CODE,
};

use super::support::{wrapped_key, OWNER_DID};

#[test]
fn unit_m8_registry_assigns_deterministic_sequence_and_due_projection_order() {
    let mut registry = DataLayerM8ComplianceRegistry::new();
    let first = registry
        .register_message(DataLayerM8MessageRecordInput {
            owner_did: OWNER_DID.to_owned(),
            message_id: "msg-2".to_owned(),
            created_at_epoch_seconds: 100,
            content_hash: "sha256:msg-2".to_owned(),
            hash_chain_prev: "sha256:prev".to_owned(),
            retention_class: DataLayerM8RetentionClass::Ephemeral,
            retention_extension_seconds: 0,
            wrapped_keys: vec![wrapped_key("kamn:did:agent:bob", "cek-bob")],
        })
        .expect("first message should register");
    let second = registry
        .register_message(DataLayerM8MessageRecordInput {
            owner_did: OWNER_DID.to_owned(),
            message_id: "msg-1".to_owned(),
            created_at_epoch_seconds: 200,
            content_hash: "sha256:msg-1".to_owned(),
            hash_chain_prev: "sha256:msg-2".to_owned(),
            retention_class: DataLayerM8RetentionClass::Ephemeral,
            retention_extension_seconds: 0,
            wrapped_keys: vec![wrapped_key("kamn:did:agent:alice", "cek-alice")],
        })
        .expect("second message should register");
    assert_eq!(first.sequence, 1);
    assert_eq!(second.sequence, 2);

    let due_candidates = registry
        .retention_due_for_owner(
            DataLayerM8OwnerScopeQuery {
                requester_owner_did: OWNER_DID.to_owned(),
                owner_did: OWNER_DID.to_owned(),
            },
            86_700,
        )
        .expect("retention due query should succeed");

    assert_eq!(
        due_candidates
            .iter()
            .map(|candidate| candidate.message_id.as_str())
            .collect::<Vec<_>>(),
        vec!["msg-2", "msg-1"]
    );
    assert!(due_candidates
        .iter()
        .all(|candidate| candidate.reason_code == DATA_LAYER_M8_RETENTION_DUE_REASON_CODE));
}
