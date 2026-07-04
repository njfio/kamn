use std::collections::BTreeMap;

use kamn_core::{
    data_layer_m3_compute_blind_index, ContentRetrievalScope, DataLayerM3BlindIndexQuery,
    DataLayerM3BlindIndexRetrievalProjectionInput, DataLayerM3BlindIndexSearchMode,
    DataLayerM3MessageMetadataRecord, DataLayerM3RetrievalProjectionRecord,
    DataLayerM3SearchCatalog,
};

pub(crate) struct RecordSeed<'a> {
    pub(crate) message_id: &'a str,
    pub(crate) owner_did: &'a str,
    pub(crate) sender_did: &'a str,
    pub(crate) recipient_did: &'a str,
    pub(crate) session_id: Option<&'a str>,
    pub(crate) escrow_id: Option<&'a str>,
    pub(crate) message_type: &'a str,
    pub(crate) created_at_epoch_seconds: u64,
    pub(crate) blind_indexes: BTreeMap<String, String>,
}

pub(crate) fn record(seed: RecordSeed<'_>) -> DataLayerM3MessageMetadataRecord {
    DataLayerM3MessageMetadataRecord {
        message_id: seed.message_id.to_owned(),
        owner_did: seed.owner_did.to_owned(),
        sender_did: seed.sender_did.to_owned(),
        recipient_did: seed.recipient_did.to_owned(),
        session_id: seed.session_id.map(str::to_owned),
        escrow_id: seed.escrow_id.map(str::to_owned),
        message_type: seed.message_type.to_owned(),
        created_at_epoch_seconds: seed.created_at_epoch_seconds,
        blind_indexes: seed.blind_indexes,
    }
}

pub(crate) fn blind_index_map(entries: &[(&str, &str)]) -> BTreeMap<String, String> {
    entries
        .iter()
        .map(|(field, token)| ((*field).to_owned(), (*token).to_owned()))
        .collect()
}

pub(crate) fn derive_blind_index_token(owner_key: &str, field_name: &str, value: &str) -> String {
    data_layer_m3_compute_blind_index(owner_key, field_name, value)
        .expect("blind index token should be derived")
}

pub(crate) fn register_record(catalog: &mut DataLayerM3SearchCatalog, seed: RecordSeed<'_>) {
    catalog
        .register_record(record(seed))
        .expect("record registration should succeed");
}

pub(crate) fn owner_a_text_record(
    message_id: &str,
    created_at_epoch_seconds: u64,
    blind_indexes: BTreeMap<String, String>,
) -> RecordSeed<'_> {
    RecordSeed {
        message_id,
        owner_did: "kamn:did:owner:a",
        sender_did: "kamn:did:agent:a1",
        recipient_did: "kamn:did:agent:b1",
        session_id: Some("session-a"),
        escrow_id: None,
        message_type: "text",
        created_at_epoch_seconds,
        blind_indexes,
    }
}

pub(crate) fn register_metadata_records(catalog: &mut DataLayerM3SearchCatalog) {
    for (message_id, sender_did, session_id, message_type, created_at_epoch_seconds) in [
        (
            "msg-x-1",
            "kamn:did:agent:sender-1",
            Some("session-1"),
            "text",
            1_708_160_050,
        ),
        (
            "msg-x-2",
            "kamn:did:agent:sender-1",
            Some("session-1"),
            "text",
            1_708_160_080,
        ),
        (
            "msg-x-3",
            "kamn:did:agent:sender-2",
            Some("session-1"),
            "text",
            1_708_160_090,
        ),
        (
            "msg-x-4",
            "kamn:did:agent:sender-1",
            Some("session-2"),
            "command",
            1_708_160_095,
        ),
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

pub(crate) fn register_projection_records(catalog: &mut DataLayerM3SearchCatalog, token: &str) {
    for message_id in ["msg-r-1", "msg-r-2"] {
        register_projection_error_record(catalog, message_id, token, "session-r");
    }
}

pub(crate) fn register_projection_error_record(
    catalog: &mut DataLayerM3SearchCatalog,
    message_id: &str,
    token: &str,
    session_id: &str,
) {
    let created_at_epoch_seconds = if message_id.ends_with('2') {
        1_708_160_020
    } else {
        1_708_160_010
    };
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

pub(crate) fn projection_input(
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

pub(crate) fn projection_cids() -> BTreeMap<String, String> {
    BTreeMap::from([
        (
            "msg-r-1".to_owned(),
            "kamn:cid:v1:aaaaaaaaaaaaaaaa".to_owned(),
        ),
        (
            "msg-r-2".to_owned(),
            "kamn:cid:v1:bbbbbbbbbbbbbbbb".to_owned(),
        ),
    ])
}

pub(crate) fn assert_projection_record(first: &DataLayerM3RetrievalProjectionRecord) {
    assert_eq!(first.cid, "kamn:cid:v1:bbbbbbbbbbbbbbbb");
    assert_eq!(first.retrieval_request.requester, "kamn:did:agent:reader-1");
    assert_eq!(
        first.retrieval_request.scope,
        ContentRetrievalScope::Task("task-1".to_owned())
    );
}
