use kamn_kolme::{
    notification_event_to_receipt as notification_event_to_receipt_contract,
    parse_notification_event, KolmeCommitReceiptFinality, KolmeNotificationEvent,
    KolmeNotificationReceipt,
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
