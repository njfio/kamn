use kamn_kolme::{
    compose_notifications_reconnect_exhausted_reason, is_valid_notifications_provider_input,
    is_valid_notifications_reconnect_budget, normalize_notifications_provider_input,
    notification_event_to_provider_receipt as notification_event_to_provider_receipt_contract,
    notification_event_to_receipt as notification_event_to_receipt_contract,
    parse_notification_event, KolmeCommitReceiptFinality, KolmeNotificationEvent,
    KolmeNotificationReceipt, KolmeProviderNotificationReceipt,
};

#[test]
fn functional_parse_notification_event_maps_new_block_variant() {
    let event = parse_notification_event(r#"{"event":"NewBlock","txhash":"0xabc123","height":42}"#)
        .expect("new block event should parse");
    assert_eq!(
        event,
        KolmeNotificationEvent::NewBlock {
            txhash: "0xabc123".to_owned(),
            block_height: Some(42),
        }
    );
}

#[test]
fn regression_issue_1735_notification_parser_rejects_unsupported_variant() {
    // Regression: #1735
    let error = parse_notification_event(r#"{"event":"SomethingElse","txhash":"0xabc"}"#)
        .expect_err("unsupported variant must fail");
    assert_eq!(error.to_string(), "notification variant is unsupported");
}

#[test]
fn functional_notification_event_to_receipt_maps_new_block_and_failed_transaction() {
    let new_block_receipt =
        notification_event_to_receipt_contract(&KolmeNotificationEvent::NewBlock {
            txhash: "0xabc123".to_owned(),
            block_height: Some(42),
        })
        .expect("new block should map to receipt");
    assert_eq!(
        new_block_receipt,
        KolmeNotificationReceipt {
            commit_id: "kolme-commit:0xabc123:h42".to_owned(),
            finality: KolmeCommitReceiptFinality::Final,
        }
    );

    let failed_receipt =
        notification_event_to_receipt_contract(&KolmeNotificationEvent::FailedTransaction {
            txhash: "0xff00".to_owned(),
            proposed_height: Some(44),
        })
        .expect("failed transaction should map to receipt");
    assert_eq!(
        failed_receipt,
        KolmeNotificationReceipt {
            commit_id: "kolme-commit:0xff00".to_owned(),
            finality: KolmeCommitReceiptFinality::Failed,
        }
    );
}

#[test]
fn regression_issue_1848_notification_event_to_receipt_returns_none_for_latest_block() {
    // Regression: #1848
    let receipt =
        notification_event_to_receipt_contract(&KolmeNotificationEvent::LatestBlock { height: 55 });
    assert_eq!(receipt, None);
}

#[test]
fn functional_notification_event_to_provider_receipt_normalizes_provider_and_maps_receipt() {
    let receipt = notification_event_to_provider_receipt_contract(
        "  kolme-fork  ",
        &KolmeNotificationEvent::NewBlock {
            txhash: "0xabc123".to_owned(),
            block_height: Some(42),
        },
    )
    .expect("new block + provider should map to provider receipt");
    assert_eq!(
        receipt,
        KolmeProviderNotificationReceipt {
            provider: "kolme-fork".to_owned(),
            commit_id: "kolme-commit:0xabc123:h42".to_owned(),
            finality: KolmeCommitReceiptFinality::Final,
        }
    );
}

#[test]
fn regression_issue_1852_notification_event_to_provider_receipt_rejects_empty_provider() {
    // Regression: #1852
    let receipt = notification_event_to_provider_receipt_contract(
        "   ",
        &KolmeNotificationEvent::FailedTransaction {
            txhash: "0xff00".to_owned(),
            proposed_height: Some(44),
        },
    );
    assert_eq!(receipt, None);
}

#[test]
fn functional_notification_policy_accepts_valid_notifications_consumer_inputs() {
    assert!(is_valid_notifications_provider_input("kolme-fork-local"));
    assert!(is_valid_notifications_reconnect_budget(2));
}

#[test]
fn functional_notification_policy_normalizes_notifications_provider_input() {
    assert_eq!(
        normalize_notifications_provider_input("  kolme-fork-local  "),
        "kolme-fork-local"
    );
}

#[test]
fn regression_issue_1868_notification_policy_rejects_invalid_notifications_consumer_inputs() {
    // Regression: #1868
    assert!(!is_valid_notifications_provider_input(" "));
    assert!(!is_valid_notifications_reconnect_budget(0));
}

#[test]
fn regression_issue_1916_notification_policy_trims_outer_provider_whitespace() {
    // Regression: #1916
    assert_eq!(
        normalize_notifications_provider_input("\nkolme-fork-local\n"),
        "kolme-fork-local"
    );
}

#[test]
fn functional_notification_policy_composes_reconnect_exhausted_reason() {
    assert_eq!(
        compose_notifications_reconnect_exhausted_reason(3),
        "notification reconnect attempts exhausted after 3 retries"
    );
}

#[test]
fn regression_issue_1924_notification_policy_composes_reconnect_exhausted_reason_deterministically()
{
    // Regression: #1924
    assert_eq!(
        compose_notifications_reconnect_exhausted_reason(2),
        "notification reconnect attempts exhausted after 2 retries"
    );
}
