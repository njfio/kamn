use super::{
    data_layer_m3_compute_blind_index, DataLayerM3BlindIndexQuery, DataLayerM3BlindIndexSearchMode,
    DataLayerM3MessageMetadataRecord, DataLayerM3SearchCatalog, DataLayerM3SearchError,
};
use std::collections::BTreeMap;

fn fixture_record(owner_did: &str, token: &str) -> DataLayerM3MessageMetadataRecord {
    let mut blind_indexes = BTreeMap::new();
    blind_indexes.insert("subject".to_owned(), token.to_owned());
    DataLayerM3MessageMetadataRecord {
        message_id: "msg-1".to_owned(),
        owner_did: owner_did.to_owned(),
        sender_did: "kamn:did:agent:alice".to_owned(),
        recipient_did: "kamn:did:agent:bob".to_owned(),
        session_id: None,
        escrow_id: None,
        message_type: "Request".to_owned(),
        created_at_epoch_seconds: 1_000,
        blind_indexes,
    }
}

#[test]
fn unit_data_layer_m3_register_and_search_blind_index_exact_match() {
    let owner_did = "kamn:did:owner:alice";
    let token = data_layer_m3_compute_blind_index(owner_did, "subject", "Invoice 42")
        .expect("expected test fixture operation to succeed");
    let mut catalog = DataLayerM3SearchCatalog::new();
    catalog
        .register_record(fixture_record(owner_did, token.as_str()))
        .expect("expected test fixture operation to succeed");
    let results = catalog
        .search_blind_index(DataLayerM3BlindIndexQuery {
            owner_did: owner_did.to_owned(),
            field_name: "subject".to_owned(),
            token,
            mode: DataLayerM3BlindIndexSearchMode::ExactMatch,
            limit: None,
        })
        .expect("expected test fixture operation to succeed");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].message_id, "msg-1");
}

#[test]
fn unit_data_layer_m3_search_rejects_unsupported_search_mode() {
    let owner_did = "kamn:did:owner:alice";
    let token = data_layer_m3_compute_blind_index(owner_did, "subject", "Invoice 42")
        .expect("expected test fixture operation to succeed");
    let mut catalog = DataLayerM3SearchCatalog::new();
    catalog
        .register_record(fixture_record(owner_did, token.as_str()))
        .expect("expected test fixture operation to succeed");
    let error = catalog
        .search_blind_index(DataLayerM3BlindIndexQuery {
            owner_did: owner_did.to_owned(),
            field_name: "subject".to_owned(),
            token,
            mode: DataLayerM3BlindIndexSearchMode::Contains,
            limit: None,
        })
        .expect_err("expected test fixture operation to fail");
    assert!(matches!(
        error,
        DataLayerM3SearchError::UnsupportedBlindIndexSearchMode(
            DataLayerM3BlindIndexSearchMode::Contains
        )
    ));
}
