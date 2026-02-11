use kamn_kolme::{parse_notification_event, KolmeNotificationEvent};

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
