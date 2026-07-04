use std::collections::BTreeMap;

use kamn_core::{DataLayerM3MetadataQuery, DataLayerM3SearchCatalog, DataLayerM3SearchError};

use crate::support::{
    assert_projection_record, derive_blind_index_token, projection_cids, projection_input,
    register_metadata_records, register_projection_error_record, register_projection_records,
};

#[test]
fn spec_c04_metadata_search_applies_filters_and_returns_stable_order() {
    let mut catalog = DataLayerM3SearchCatalog::new();
    register_metadata_records(&mut catalog);

    let filtered = catalog
        .search_metadata(DataLayerM3MetadataQuery {
            owner_did: "kamn:did:owner:a".to_owned(),
            sender_did: Some("kamn:did:agent:sender-1".to_owned()),
            recipient_did: Some("kamn:did:agent:recipient-1".to_owned()),
            session_id: Some("session-1".to_owned()),
            escrow_id: None,
            message_type: Some("text".to_owned()),
            created_after_inclusive: Some(1_708_160_000),
            created_before_inclusive: Some(1_708_160_090),
            limit: Some(10),
        })
        .expect("metadata search should succeed");
    let filtered_ids = filtered
        .iter()
        .map(|entry| entry.message_id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(filtered_ids, vec!["msg-x-2", "msg-x-1"]);
}

#[test]
fn spec_c09_blind_index_projection_builds_deterministic_content_retrieval_requests() {
    let token = derive_blind_index_token("owner-key-a", "subject", "invoice 42");
    let mut catalog = DataLayerM3SearchCatalog::new();
    register_projection_records(&mut catalog, &token);

    let projection = catalog
        .project_blind_index_to_retrieval_requests(projection_input(
            &token,
            "kamn:did:agent:reader-1",
            projection_cids(),
        ))
        .expect("projection should succeed");

    let message_ids = projection
        .iter()
        .map(|entry| entry.message_id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(message_ids, vec!["msg-r-2", "msg-r-1"]);
    assert_projection_record(
        projection
            .first()
            .expect("first projection record should exist"),
    );
}

#[test]
fn spec_c10_blind_index_projection_fails_closed_when_message_cid_map_missing() {
    let token = derive_blind_index_token("owner-key-a", "subject", "invoice 42");
    let mut catalog = DataLayerM3SearchCatalog::new();
    register_projection_error_record(&mut catalog, "msg-missing-cid", &token, "session-m");

    let missing = catalog.project_blind_index_to_retrieval_requests(projection_input(
        &token,
        "kamn:did:agent:reader-1",
        BTreeMap::new(),
    ));
    assert_eq!(
        missing,
        Err(DataLayerM3SearchError::MissingContentCidForMessage {
            message_id: "msg-missing-cid".to_owned(),
        })
    );
}

#[test]
fn spec_c11_blind_index_projection_fails_closed_for_invalid_requester_contract() {
    let token = derive_blind_index_token("owner-key-a", "subject", "invoice 42");
    let mut catalog = DataLayerM3SearchCatalog::new();
    register_projection_error_record(&mut catalog, "msg-invalid-requester", &token, "session-i");

    let invalid = catalog.project_blind_index_to_retrieval_requests(projection_input(
        &token,
        "invalid-requester",
        BTreeMap::from([(
            "msg-invalid-requester".to_owned(),
            "kamn:cid:v1:aaaaaaaaaaaaaaaa".to_owned(),
        )]),
    ));
    assert!(matches!(
        invalid,
        Err(DataLayerM3SearchError::InvalidRetrievalRequestProjection { .. })
    ));
}
