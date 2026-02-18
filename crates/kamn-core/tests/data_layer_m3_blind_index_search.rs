use kamn_core::{
    data_layer_m3_compute_blind_index, DataLayerM3BlindIndexQuery, DataLayerM3BlindIndexSearchMode,
    DataLayerM3MessageMetadataRecord, DataLayerM3MetadataQuery, DataLayerM3SearchCatalog,
    DataLayerM3SearchError,
};
use std::collections::BTreeMap;

struct RecordSeed<'a> {
    message_id: &'a str,
    owner_did: &'a str,
    sender_did: &'a str,
    recipient_did: &'a str,
    session_id: Option<&'a str>,
    escrow_id: Option<&'a str>,
    message_type: &'a str,
    created_at_epoch_seconds: u64,
    blind_indexes: BTreeMap<String, String>,
}

fn record(seed: RecordSeed<'_>) -> DataLayerM3MessageMetadataRecord {
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

fn blind_index_map(entries: &[(&str, &str)]) -> BTreeMap<String, String> {
    entries
        .iter()
        .map(|(field, token)| ((*field).to_owned(), (*token).to_owned()))
        .collect()
}

#[test]
fn spec_c01_blind_index_normalization_is_case_and_whitespace_deterministic() {
    let token_a = data_layer_m3_compute_blind_index("owner-key-a", "subject", "   Invoice   42 ")
        .expect("blind index token should be derived");
    let token_b = data_layer_m3_compute_blind_index("owner-key-a", "subject", "invoice 42")
        .expect("blind index token should be derived");

    assert_eq!(token_a, token_b);
    assert!(token_a.starts_with("sha256:"));
}

#[test]
fn spec_c02_blind_index_tokens_are_owner_scoped_by_key_material() {
    let owner_a = data_layer_m3_compute_blind_index("owner-key-a", "subject", "invoice 42")
        .expect("owner A token should be derived");
    let owner_b = data_layer_m3_compute_blind_index("owner-key-b", "subject", "invoice 42")
        .expect("owner B token should be derived");

    assert_ne!(owner_a, owner_b);
}

#[test]
fn spec_c03_exact_match_blind_index_search_is_owner_scoped_and_deterministic() {
    let owner_a_invoice = data_layer_m3_compute_blind_index("owner-key-a", "subject", "invoice 42")
        .expect("owner A invoice token should be derived");
    let owner_b_invoice = data_layer_m3_compute_blind_index("owner-key-b", "subject", "invoice 42")
        .expect("owner B invoice token should be derived");
    let owner_a_other = data_layer_m3_compute_blind_index("owner-key-a", "subject", "invoice 99")
        .expect("owner A alternate token should be derived");

    let mut catalog = DataLayerM3SearchCatalog::new();
    catalog
        .register_record(record(RecordSeed {
            message_id: "msg-a-1",
            owner_did: "kamn:did:owner:a",
            sender_did: "kamn:did:agent:a1",
            recipient_did: "kamn:did:agent:b1",
            session_id: Some("session-a"),
            escrow_id: None,
            message_type: "text",
            created_at_epoch_seconds: 1_708_160_010,
            blind_indexes: blind_index_map(&[("subject", owner_a_invoice.as_str())]),
        }))
        .expect("record registration should succeed");
    catalog
        .register_record(record(RecordSeed {
            message_id: "msg-a-2",
            owner_did: "kamn:did:owner:a",
            sender_did: "kamn:did:agent:a1",
            recipient_did: "kamn:did:agent:b1",
            session_id: Some("session-a"),
            escrow_id: None,
            message_type: "text",
            created_at_epoch_seconds: 1_708_160_020,
            blind_indexes: blind_index_map(&[("subject", owner_a_invoice.as_str())]),
        }))
        .expect("record registration should succeed");
    catalog
        .register_record(record(RecordSeed {
            message_id: "msg-a-3",
            owner_did: "kamn:did:owner:a",
            sender_did: "kamn:did:agent:a1",
            recipient_did: "kamn:did:agent:b1",
            session_id: Some("session-a"),
            escrow_id: None,
            message_type: "text",
            created_at_epoch_seconds: 1_708_160_030,
            blind_indexes: blind_index_map(&[("subject", owner_a_other.as_str())]),
        }))
        .expect("record registration should succeed");
    catalog
        .register_record(record(RecordSeed {
            message_id: "msg-b-1",
            owner_did: "kamn:did:owner:b",
            sender_did: "kamn:did:agent:b1",
            recipient_did: "kamn:did:agent:a1",
            session_id: Some("session-b"),
            escrow_id: None,
            message_type: "text",
            created_at_epoch_seconds: 1_708_160_040,
            blind_indexes: blind_index_map(&[("subject", owner_b_invoice.as_str())]),
        }))
        .expect("record registration should succeed");

    let exact = catalog
        .search_blind_index(DataLayerM3BlindIndexQuery {
            owner_did: "kamn:did:owner:a".to_owned(),
            field_name: "subject".to_owned(),
            token: owner_a_invoice.clone(),
            mode: DataLayerM3BlindIndexSearchMode::ExactMatch,
            limit: Some(10),
        })
        .expect("exact blind-index search should succeed");
    let exact_ids = exact
        .iter()
        .map(|entry| entry.message_id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(exact_ids, vec!["msg-a-2", "msg-a-1"]);
}

#[test]
fn spec_c04_metadata_search_applies_filters_and_returns_stable_order() {
    let mut catalog = DataLayerM3SearchCatalog::new();
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
        catalog
            .register_record(record(RecordSeed {
                message_id,
                owner_did: "kamn:did:owner:a",
                sender_did,
                recipient_did: "kamn:did:agent:recipient-1",
                session_id,
                escrow_id: None,
                message_type,
                created_at_epoch_seconds,
                blind_indexes: BTreeMap::new(),
            }))
            .expect("record registration should succeed");
    }

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
fn spec_c05_invalid_blind_index_modes_and_bounds_fail_closed() {
    let token = data_layer_m3_compute_blind_index("owner-key-a", "subject", "invoice 42")
        .expect("token should derive");
    let mut catalog = DataLayerM3SearchCatalog::new();
    catalog
        .register_record(record(RecordSeed {
            message_id: "msg-z-1",
            owner_did: "kamn:did:owner:a",
            sender_did: "kamn:did:agent:a1",
            recipient_did: "kamn:did:agent:b1",
            session_id: Some("session-z"),
            escrow_id: None,
            message_type: "text",
            created_at_epoch_seconds: 1_708_160_111,
            blind_indexes: blind_index_map(&[("subject", token.as_str())]),
        }))
        .expect("record registration should succeed");

    let contains = catalog.search_blind_index(DataLayerM3BlindIndexQuery {
        owner_did: "kamn:did:owner:a".to_owned(),
        field_name: "subject".to_owned(),
        token: token.clone(),
        mode: DataLayerM3BlindIndexSearchMode::Contains,
        limit: Some(10),
    });
    assert_eq!(
        contains,
        Err(DataLayerM3SearchError::UnsupportedBlindIndexSearchMode(
            DataLayerM3BlindIndexSearchMode::Contains
        ))
    );

    let invalid_bounds = catalog.search_metadata(DataLayerM3MetadataQuery {
        owner_did: "kamn:did:owner:a".to_owned(),
        sender_did: None,
        recipient_did: None,
        session_id: None,
        escrow_id: None,
        message_type: None,
        created_after_inclusive: Some(100),
        created_before_inclusive: Some(10),
        limit: Some(10),
    });
    assert_eq!(
        invalid_bounds,
        Err(DataLayerM3SearchError::InvalidTimestampBounds {
            created_after_inclusive: 100,
            created_before_inclusive: 10,
        })
    );
}
