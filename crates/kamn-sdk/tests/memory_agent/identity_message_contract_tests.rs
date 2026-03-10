use super::support::*;

#[test]
fn register_and_resolve_round_trip() {
    let mut client = InMemoryKamnClient::new();
    let did = register_agent(&mut client, "autonomous", "claude-4", &["text", "code"]);
    let document = client.resolve(&did).expect("resolve failed");
    assert_eq!(document.id, did);
    assert_eq!(document.metadata.agent_type, "autonomous");
    assert_eq!(document.metadata.model_family, "claude-4");
}

#[test]
fn send_and_receive_drains_inbox() {
    let mut client = InMemoryKamnClient::new();
    let sender = register_agent(&mut client, "autonomous", "claude-4", &["text"]);
    let recipient = register_agent(&mut client, "assistant", "gpt-5", &["text"]);
    client
        .send(Message {
            from: sender,
            to: recipient.clone(),
            body: "hello world".to_owned(),
            channel: None,
        })
        .expect("send failed");
    let first = client.receive(&recipient).expect("first receive failed");
    let second = client.receive(&recipient).expect("second receive failed");
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].message.body, "hello world");
    assert!(second.is_empty());
}

#[test]
fn receive_stream_orders_messages_deterministically() {
    let mut client = InMemoryKamnClient::new();
    let sender = register_agent(&mut client, "autonomous", "claude-4", &["text"]);
    let recipient = register_agent(&mut client, "assistant", "gpt-5", &["text"]);
    for body in ["first", "second"] {
        client
            .send(Message {
                from: sender.clone(),
                to: recipient.clone(),
                body: body.to_owned(),
                channel: None,
            })
            .unwrap_or_else(|error| panic!("send failed: {error}"));
    }
    let bodies = client
        .receive_stream(&recipient)
        .expect("receive stream failed")
        .map(|record| record.message.body)
        .collect::<Vec<_>>();
    assert_eq!(bodies, vec!["first".to_owned(), "second".to_owned()]);
}

#[test]
fn receive_stream_does_not_replay_consumed_messages() {
    let mut client = InMemoryKamnClient::new();
    let sender = register_agent(&mut client, "autonomous", "claude-4", &["text"]);
    let recipient = register_agent(&mut client, "assistant", "gpt-5", &["text"]);
    client
        .send(Message {
            from: sender,
            to: recipient.clone(),
            body: "once".to_owned(),
            channel: None,
        })
        .expect("send failed");
    let first_len = client.receive_stream(&recipient).expect("first stream failed").count();
    let second_len = client.receive_stream(&recipient).expect("second stream failed").count();
    assert_eq!(first_len, 1);
    assert_eq!(second_len, 0);
}

#[test]
fn get_message_status_reports_known_sent_message_after_receive() {
    let mut client = InMemoryKamnClient::new();
    let sender = register_agent(&mut client, "autonomous", "claude-4", &["text"]);
    let recipient = register_agent(&mut client, "assistant", "gpt-5", &["text"]);
    let message_id = client
        .send(Message {
            from: sender,
            to: recipient.clone(),
            body: "hello".to_owned(),
            channel: None,
        })
        .expect("send should succeed");
    let _ = client.receive(&recipient).expect("receive should succeed");
    assert_eq!(
        client
            .get_message_status(&message_id)
            .expect("message status should remain available")
            .status,
        "created"
    );
}

#[test]
fn get_message_status_rejects_unknown_message() {
    let client = InMemoryKamnClient::new();
    assert_not_found(client.get_message_status(&MessageId(46)), "message", "46");
}
