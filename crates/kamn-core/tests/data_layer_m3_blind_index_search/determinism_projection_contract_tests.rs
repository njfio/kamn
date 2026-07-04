use kamn_core::{
    DataLayerM3BlindIndexDeterminismDecision, DataLayerM3BlindIndexDeterminismInput,
    DataLayerM3SearchCatalog, DataLayerM3SearchError,
    DATA_LAYER_M3_BLIND_INDEX_DETERMINISM_DRIFTED_REASON_CODE,
    DATA_LAYER_M3_BLIND_INDEX_DETERMINISM_STABLE_REASON_CODE,
};

use crate::support::{
    blind_index_map, derive_blind_index_token, owner_a_text_record, register_record,
};

#[test]
fn spec_c06_blind_index_determinism_reports_stable_when_baseline_matches_query_order() {
    let token = derive_blind_index_token("owner-key-a", "subject", "invoice 42");
    let mut catalog = DataLayerM3SearchCatalog::new();
    register_determinism_records(&mut catalog, &token, "msg-d");

    let report = catalog
        .evaluate_blind_index_determinism(determinism_input(
            &token,
            vec!["msg-d-2".to_owned(), "msg-d-1".to_owned()],
            Some(10),
        ))
        .expect("determinism evaluation should succeed");
    assert_eq!(
        report.decision,
        DataLayerM3BlindIndexDeterminismDecision::Stable
    );
    assert_eq!(
        report.reason_code,
        DATA_LAYER_M3_BLIND_INDEX_DETERMINISM_STABLE_REASON_CODE
    );
    assert!(report.missing_message_ids.is_empty());
    assert!(report.unexpected_message_ids.is_empty());
    assert!(report.out_of_order_message_ids.is_empty());
}

#[test]
fn spec_c07_blind_index_determinism_reports_drift_with_missing_and_out_of_order_evidence() {
    let token = derive_blind_index_token("owner-key-a", "subject", "invoice 42");
    let mut catalog = DataLayerM3SearchCatalog::new();
    register_determinism_records(&mut catalog, &token, "msg-e");

    let report = catalog
        .evaluate_blind_index_determinism(determinism_input(
            &token,
            vec![
                "msg-e-1".to_owned(),
                "msg-e-2".to_owned(),
                "msg-e-missing".to_owned(),
            ],
            Some(10),
        ))
        .expect("determinism evaluation should succeed");
    assert_eq!(
        report.decision,
        DataLayerM3BlindIndexDeterminismDecision::Drifted
    );
    assert_eq!(
        report.reason_code,
        DATA_LAYER_M3_BLIND_INDEX_DETERMINISM_DRIFTED_REASON_CODE
    );
    assert_eq!(report.missing_message_ids, vec!["msg-e-missing".to_owned()]);
    assert_eq!(
        report.out_of_order_message_ids,
        vec!["msg-e-1".to_owned(), "msg-e-2".to_owned()]
    );
}

#[test]
fn spec_c08_blind_index_determinism_rejects_empty_baseline_and_invalid_limit() {
    let token = derive_blind_index_token("owner-key-a", "subject", "invoice 42");
    let catalog = DataLayerM3SearchCatalog::new();

    let empty_baseline =
        catalog.evaluate_blind_index_determinism(determinism_input(&token, Vec::new(), Some(10)));
    assert_eq!(
        empty_baseline,
        Err(DataLayerM3SearchError::EmptyField(
            "baseline_ordered_message_ids"
        ))
    );

    let invalid_limit = catalog.evaluate_blind_index_determinism(determinism_input(
        &token,
        vec!["msg-any".to_owned()],
        Some(0),
    ));
    assert_eq!(invalid_limit, Err(DataLayerM3SearchError::InvalidLimit(0)));
}

fn register_determinism_records(catalog: &mut DataLayerM3SearchCatalog, token: &str, prefix: &str) {
    register_record(
        catalog,
        owner_a_text_record(
            &format!("{prefix}-1"),
            1_708_160_010,
            blind_index_map(&[("subject", token)]),
        ),
    );
    register_record(
        catalog,
        owner_a_text_record(
            &format!("{prefix}-2"),
            1_708_160_020,
            blind_index_map(&[("subject", token)]),
        ),
    );
}

fn determinism_input(
    token: &str,
    baseline_ordered_message_ids: Vec<String>,
    limit: Option<usize>,
) -> DataLayerM3BlindIndexDeterminismInput {
    DataLayerM3BlindIndexDeterminismInput {
        owner_did: "kamn:did:owner:a".to_owned(),
        field_name: "subject".to_owned(),
        token: token.to_owned(),
        baseline_ordered_message_ids,
        limit,
    }
}
