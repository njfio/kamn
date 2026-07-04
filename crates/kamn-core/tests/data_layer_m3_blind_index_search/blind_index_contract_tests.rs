use kamn_core::{
    DataLayerM3BlindIndexQuery, DataLayerM3BlindIndexSearchMode, DataLayerM3SearchCatalog,
    DataLayerM3SearchError,
};

use crate::support::{
    blind_index_map, derive_blind_index_token, owner_a_text_record, register_record,
};

#[test]
fn spec_c01_blind_index_normalization_is_case_and_whitespace_deterministic() {
    let token_a = derive_blind_index_token("owner-key-a", "subject", "   Invoice   42 ");
    let token_b = derive_blind_index_token("owner-key-a", "subject", "invoice 42");
    assert_eq!(token_a, token_b);
    assert!(token_a.starts_with("sha256:"));
}

#[test]
fn spec_c02_blind_index_tokens_are_owner_scoped_by_key_material() {
    let owner_a = derive_blind_index_token("owner-key-a", "subject", "invoice 42");
    let owner_b = derive_blind_index_token("owner-key-b", "subject", "invoice 42");
    assert_ne!(owner_a, owner_b);
}

#[test]
fn spec_c03_exact_match_blind_index_search_is_owner_scoped_and_deterministic() {
    let owner_a_invoice = derive_blind_index_token("owner-key-a", "subject", "invoice 42");
    let owner_b_invoice = derive_blind_index_token("owner-key-b", "subject", "invoice 42");
    let owner_a_other = derive_blind_index_token("owner-key-a", "subject", "invoice 99");

    let mut catalog = DataLayerM3SearchCatalog::new();
    register_owner_a_invoice_records(&mut catalog, &owner_a_invoice);
    register_owner_a_other_record(&mut catalog, &owner_a_other);
    register_owner_b_record(&mut catalog, &owner_b_invoice);

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
fn spec_c05_invalid_blind_index_modes_and_bounds_fail_closed() {
    let token = derive_blind_index_token("owner-key-a", "subject", "invoice 42");
    let mut catalog = DataLayerM3SearchCatalog::new();
    register_record(
        &mut catalog,
        owner_a_text_record(
            "msg-z-1",
            1_708_160_111,
            blind_index_map(&[("subject", token.as_str())]),
        ),
    );

    assert_unsupported_search_mode(&catalog, &token);
    assert_invalid_timestamp_bounds(&catalog);
}

fn register_owner_a_invoice_records(catalog: &mut DataLayerM3SearchCatalog, token: &str) {
    register_record(
        catalog,
        owner_a_text_record(
            "msg-a-1",
            1_708_160_010,
            blind_index_map(&[("subject", token)]),
        ),
    );
    register_record(
        catalog,
        owner_a_text_record(
            "msg-a-2",
            1_708_160_020,
            blind_index_map(&[("subject", token)]),
        ),
    );
}

fn register_owner_a_other_record(catalog: &mut DataLayerM3SearchCatalog, token: &str) {
    register_record(
        catalog,
        owner_a_text_record(
            "msg-a-3",
            1_708_160_030,
            blind_index_map(&[("subject", token)]),
        ),
    );
}

fn register_owner_b_record(catalog: &mut DataLayerM3SearchCatalog, token: &str) {
    register_record(
        catalog,
        crate::support::RecordSeed {
            message_id: "msg-b-1",
            owner_did: "kamn:did:owner:b",
            sender_did: "kamn:did:agent:b1",
            recipient_did: "kamn:did:agent:a1",
            session_id: Some("session-b"),
            escrow_id: None,
            message_type: "text",
            created_at_epoch_seconds: 1_708_160_040,
            blind_indexes: blind_index_map(&[("subject", token)]),
        },
    );
}

fn assert_unsupported_search_mode(catalog: &DataLayerM3SearchCatalog, token: &str) {
    let contains = catalog.search_blind_index(DataLayerM3BlindIndexQuery {
        owner_did: "kamn:did:owner:a".to_owned(),
        field_name: "subject".to_owned(),
        token: token.to_owned(),
        mode: DataLayerM3BlindIndexSearchMode::Contains,
        limit: Some(10),
    });
    assert_eq!(
        contains,
        Err(DataLayerM3SearchError::UnsupportedBlindIndexSearchMode(
            DataLayerM3BlindIndexSearchMode::Contains
        ))
    );
}

fn assert_invalid_timestamp_bounds(catalog: &DataLayerM3SearchCatalog) {
    let invalid_bounds = catalog.search_metadata(kamn_core::DataLayerM3MetadataQuery {
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
