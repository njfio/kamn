use std::collections::BTreeMap;

use kamn_core::{
    ContentRetrievalScope, DataLayerM3BlindIndexQuery, DataLayerM3BlindIndexRetrievalProjectionInput,
    DataLayerM3BlindIndexSearchMode, DataLayerM3MetadataQuery, DataLayerM3RetrievalProjectionRecord,
    DataLayerM3SearchCatalog, DataLayerM3SearchError,
};

use crate::support::{blind_index_map, derive_token, register_record, RecordSeed};

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
    let filtered_ids = filtered.iter().map(|entry| entry.message_id.as_str()).collect::<Vec<_>>();
    assert_eq!(filtered_ids, vec!["msg-x-2", "msg-x-1"]);
}

#[test]
fn spec_c09_blind_index_projection_builds_deterministic_content_retrieval_requests() {
    let token = derive_token("owner-key-a", "subject", "invoice 42");
    let mut catalog = DataLayerM3SearchCatalog::new();
    register_projection_records(&mut catalog, &token);

    let projection = catalog
        .project_blind_index_to_retrieval_requests(DataLayerM3BlindIndexRetrievalProjectionInput {
            blind_index_query: DataLayerM3BlindIndexQuery {
                owner_did: "kamn:did:owner:a".to_owned(),
                field_name: "subject".to_owned(),
                token: token.clone(),
                mode: DataLayerM3BlindIndexSearchMode::ExactMatch,
                limit: Some(10),
            },
            requester_did: "kamn:did:agent:reader-1".to_owned(),
            retrieval_scope: ContentRetrievalScope::Task("task-1".to_owned()),
            requested_at_unix: 1_708_160_100,
            message_cids_by_message_id: projection_cids(),
        })
        .expect("projection should succeed");

    let message_ids = projection.iter().map(|entry| entry.message_id.as_str()).collect::<Vec<_>>();
    assert_eq!(message_ids, vec!["msg-r-2", "msg-r-1"]);
    assert_projection_record(projection.first().expect("first projection record should exist"));
}

#[test]
fn spec_c10_blind_index_projection_fails_closed_when_message_cid_map_missing() {
    let token = derive_token("owner-key-a", "subject", "invoice 42");
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
    let token = derive_token("owner-key-a", "subject", "invoice 42");
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

fn register_metadata_records(catalog: &mut DataLayerM3SearchCatalog) {
    for (message_id, sender_did, session_id, message_type, created_at_epoch_seconds) in [
        ("msg-x-1", "kamn:did:agent:sender-1", Some("session-1"), "text", 1_708_160_050),
        ("msg-x-2", "kamn:did:agent:sender-1", Some("session-1"), "text", 1_708_160_080),
        ("msg-x-3", "kamn:did:agent:sender-2", Some("session-1"), "text", 1_708_160_090),
        ("msg-x-4", "kamn:did:agent:sender-1", Some("session-2"), "command", 1_708_160_095),
    ] {
        register_record(
            catalog,
            RecordSeed {
                message_id,
                owner_did: "kamn:did:owner:a",
                sender_did,
                recipient_did: "kamn:did:agent:recipient-1",
                session_id,
                escrow_id: None,
                message_type,
                created_at_epoch_seconds,
                blind_indexes: BTreeMap::new(),
            },
        );
    }
}

fn register_projection_records(catalog: &mut DataLayerM3SearchCatalog, token: &str) {
    for (message_id, created_at_epoch_seconds) in [("msg-r-1", 1_708_160_010), ("msg-r-2", 1_708_160_020)] {
        register_projection_error_record(catalog, message_id, token, "session-r");
        let _ = created_at_epoch_seconds;
    }
}

fn register_projection_error_record(
    catalog: &mut DataLayerM3SearchCatalog,
    message_id: &str,
    token: &str,
    session_id: &str,
) {
    let created_at_epoch_seconds = if message_id.ends_with("2") { 1_708_160_020 } else { 1_708_160_010 };
    register_record(
        catalog,
        RecordSeed {
            message_id,
            owner_did: "kamn:did:owner:a",
            sender_did: "kamn:did:agent:a1",
            recipient_did: "kamn:did:agent:b1",
            session_id: Some(session_id),
            escrow_id: None,
            message_type: "text",
            created_at_epoch_seconds,
            blind_indexes: blind_index_map(&[("subject", token)]),
        },
    );
}

fn projection_input(
    token: &str,
    requester_did: &str,
    message_cids_by_message_id: BTreeMap<String, String>,
) -> DataLayerM3BlindIndexRetrievalProjectionInput {
    DataLayerM3BlindIndexRetrievalProjectionInput {
        blind_index_query: DataLayerM3BlindIndexQuery {
            owner_did: "kamn:did:owner:a".to_owned(),
            field_name: "subject".to_owned(),
            token: token.to_owned(),
            mode: DataLayerM3BlindIndexSearchMode::ExactMatch,
            limit: Some(10),
        },
        requester_did: requester_did.to_owned(),
        retrieval_scope: ContentRetrievalScope::Task("task-1".to_owned()),
        requested_at_unix: 1_708_160_100,
        message_cids_by_message_id,
    }
}

fn projection_cids() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("msg-r-1".to_owned(), "kamn:cid:v1:aaaaaaaaaaaaaaaa".to_owned()),
        ("msg-r-2".to_owned(), "kamn:cid:v1:bbbbbbbbbbbbbbbb".to_owned()),
    ])
}

fn assert_projection_record(first: &DataLayerM3RetrievalProjectionRecord) {
    assert_eq!(first.cid, "kamn:cid:v1:bbbbbbbbbbbbbbbb");
    assert_eq!(first.retrieval_request.requester, "kamn:did:agent:reader-1");
    assert_eq!(first.retrieval_request.scope, ContentRetrievalScope::Task("task-1".to_owned()));
}
