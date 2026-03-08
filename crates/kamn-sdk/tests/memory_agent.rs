use kamn_sdk::{
    AgentDid, AgentMetadata, AgentQuery, Artifact, ArtifactStatus, EscrowConfig,
    InMemoryKamnClient, KamnAgent, Message, SdkError, TaskDefinition, TokenAmount,
};

fn metadata(agent_type: &str, model: &str, capabilities: &[&str]) -> AgentMetadata {
    AgentMetadata {
        agent_type: agent_type.to_owned(),
        model_family: model.to_owned(),
        capabilities: capabilities.iter().map(|cap| (*cap).to_owned()).collect(),
    }
}

fn valid_did(identifier: &str) -> AgentDid {
    match AgentDid::parse(format!("kamn:did:agent:{identifier}")) {
        Ok(did) => did,
        Err(error) => panic!("did parse failed: {error}"),
    }
}

#[test]
fn register_and_resolve_round_trip() {
    let mut client = InMemoryKamnClient::new();

    let did = match client.register(metadata("autonomous", "claude-4", &["text", "code"])) {
        Ok(value) => value,
        Err(error) => panic!("register failed: {error}"),
    };
    let document = match client.resolve(&did) {
        Ok(value) => value,
        Err(error) => panic!("resolve failed: {error}"),
    };

    assert_eq!(document.id, did);
    assert_eq!(document.metadata.agent_type, "autonomous");
    assert_eq!(document.metadata.model_family, "claude-4");
}

#[test]
fn send_and_receive_drains_inbox() {
    // Regression: #133
    let mut client = InMemoryKamnClient::new();
    let sender = match client.register(metadata("autonomous", "claude-4", &["text"])) {
        Ok(value) => value,
        Err(error) => panic!("register sender failed: {error}"),
    };
    let recipient = match client.register(metadata("assistant", "gpt-5", &["text"])) {
        Ok(value) => value,
        Err(error) => panic!("register recipient failed: {error}"),
    };

    let message = Message {
        from: sender,
        to: recipient.clone(),
        body: "hello world".to_owned(),
        channel: None,
    };
    if let Err(error) = client.send(message) {
        panic!("send failed: {error}");
    }

    let first = match client.receive(&recipient) {
        Ok(value) => value,
        Err(error) => panic!("first receive failed: {error}"),
    };
    let second = match client.receive(&recipient) {
        Ok(value) => value,
        Err(error) => panic!("second receive failed: {error}"),
    };

    assert_eq!(first.len(), 1);
    assert_eq!(first[0].message.body, "hello world");
    assert!(second.is_empty());
}

#[test]
fn receive_stream_orders_messages_deterministically() {
    let mut client = InMemoryKamnClient::new();
    let sender = match client.register(metadata("autonomous", "claude-4", &["text"])) {
        Ok(value) => value,
        Err(error) => panic!("register sender failed: {error}"),
    };
    let recipient = match client.register(metadata("assistant", "gpt-5", &["text"])) {
        Ok(value) => value,
        Err(error) => panic!("register recipient failed: {error}"),
    };

    if let Err(error) = client.send(Message {
        from: sender.clone(),
        to: recipient.clone(),
        body: "first".to_owned(),
        channel: None,
    }) {
        panic!("send first failed: {error}");
    }
    if let Err(error) = client.send(Message {
        from: sender,
        to: recipient.clone(),
        body: "second".to_owned(),
        channel: None,
    }) {
        panic!("send second failed: {error}");
    }

    let bodies = match client.receive_stream(&recipient) {
        Ok(stream) => stream.map(|record| record.message.body).collect::<Vec<_>>(),
        Err(error) => panic!("receive stream failed: {error}"),
    };

    assert_eq!(bodies, vec!["first".to_owned(), "second".to_owned()]);
}

#[test]
fn receive_stream_does_not_replay_consumed_messages() {
    // Regression: #468
    let mut client = InMemoryKamnClient::new();
    let sender = match client.register(metadata("autonomous", "claude-4", &["text"])) {
        Ok(value) => value,
        Err(error) => panic!("register sender failed: {error}"),
    };
    let recipient = match client.register(metadata("assistant", "gpt-5", &["text"])) {
        Ok(value) => value,
        Err(error) => panic!("register recipient failed: {error}"),
    };

    if let Err(error) = client.send(Message {
        from: sender,
        to: recipient.clone(),
        body: "once".to_owned(),
        channel: None,
    }) {
        panic!("send failed: {error}");
    }

    let first_len = match client.receive_stream(&recipient) {
        Ok(stream) => stream.count(),
        Err(error) => panic!("first stream failed: {error}"),
    };
    let second_len = match client.receive_stream(&recipient) {
        Ok(stream) => stream.count(),
        Err(error) => panic!("second stream failed: {error}"),
    };

    assert_eq!(first_len, 1);
    assert_eq!(second_len, 0);
}

#[test]
fn task_accept_rejects_second_acceptance() {
    let mut client = InMemoryKamnClient::new();
    let creator = match client.register(metadata("autonomous", "claude-4", &["research"])) {
        Ok(value) => value,
        Err(error) => panic!("register creator failed: {error}"),
    };
    let assignee = match client.register(metadata("assistant", "gpt-5", &["research"])) {
        Ok(value) => value,
        Err(error) => panic!("register assignee failed: {error}"),
    };

    let task = TaskDefinition {
        creator,
        task_type: "research".to_owned(),
        description: "compare protocols".to_owned(),
    };
    let task_id = match client.create_task(task) {
        Ok(value) => value,
        Err(error) => panic!("create task failed: {error}"),
    };

    if let Err(error) = client.accept_task(&task_id, &assignee) {
        panic!("first accept failed: {error}");
    }
    let second = client.accept_task(&task_id, &assignee);

    assert_eq!(second, Err(SdkError::Conflict("task already accepted")));
}

#[test]
fn get_task_status_reports_submitted_accepted_and_completed_states() {
    let mut client = InMemoryKamnClient::new();
    let creator = client
        .register(metadata("autonomous", "claude-4", &["research"]))
        .expect("register creator should succeed");
    let assignee = client
        .register(metadata("assistant", "gpt-5", &["research"]))
        .expect("register assignee should succeed");

    let task_id = client
        .create_task(TaskDefinition {
            creator,
            task_type: "research".to_owned(),
            description: "compare protocols".to_owned(),
        })
        .expect("create task should succeed");
    assert_eq!(
        client
            .get_task_status(&task_id)
            .expect("submitted status should resolve")
            .state,
        "submitted"
    );

    client
        .accept_task(&task_id, &assignee)
        .expect("accept task should succeed");
    assert_eq!(
        client
            .get_task_status(&task_id)
            .expect("accepted status should resolve")
            .state,
        "accepted"
    );

    client
        .complete_task(&task_id)
        .expect("complete task should succeed");
    assert_eq!(
        client
            .get_task_status(&task_id)
            .expect("completed status should resolve")
            .state,
        "completed"
    );
}

#[test]
fn get_task_status_rejects_unknown_task() {
    let client = InMemoryKamnClient::new();

    assert_eq!(
        client.get_task_status(&kamn_sdk::TaskId(45)),
        Err(SdkError::NotFound {
            entity: "task",
            id: "45".to_owned(),
        })
    );
}

#[test]
fn escrow_moves_balances_from_payer_to_payee() {
    let mut client = InMemoryKamnClient::new();
    let payer = match client.register(metadata("autonomous", "claude-4", &["pay"])) {
        Ok(value) => value,
        Err(error) => panic!("register payer failed: {error}"),
    };
    let payee = match client.register(metadata("assistant", "gpt-5", &["deliver"])) {
        Ok(value) => value,
        Err(error) => panic!("register payee failed: {error}"),
    };

    let payer_before = match client.balance(&payer) {
        Ok(value) => value,
        Err(error) => panic!("balance payer before failed: {error}"),
    };
    let payee_before = match client.balance(&payee) {
        Ok(value) => value,
        Err(error) => panic!("balance payee before failed: {error}"),
    };

    let escrow = EscrowConfig {
        payer: payer.clone(),
        payee: payee.clone(),
        amount: TokenAmount(25),
    };
    let escrow_id = match client.create_escrow(escrow) {
        Ok(value) => value,
        Err(error) => panic!("create escrow failed: {error}"),
    };
    if let Err(error) = client.release_escrow(&escrow_id) {
        panic!("release escrow failed: {error}");
    }

    let payer_after = match client.balance(&payer) {
        Ok(value) => value,
        Err(error) => panic!("balance payer after failed: {error}"),
    };
    let payee_after = match client.balance(&payee) {
        Ok(value) => value,
        Err(error) => panic!("balance payee after failed: {error}"),
    };

    assert_eq!(payer_after.0, payer_before.0.saturating_sub(25));
    assert_eq!(payee_after.0, payee_before.0.saturating_add(25));
}

#[test]
fn submit_artifact_and_complete_task_flow() {
    let mut client = InMemoryKamnClient::new();
    let creator = match client.register(metadata("autonomous", "claude-4", &["research"])) {
        Ok(value) => value,
        Err(error) => panic!("register creator failed: {error}"),
    };
    let assignee = match client.register(metadata("assistant", "gpt-5", &["research"])) {
        Ok(value) => value,
        Err(error) => panic!("register assignee failed: {error}"),
    };

    let task_id = match client.create_task(TaskDefinition {
        creator,
        task_type: "analysis".to_owned(),
        description: "analyze benchmark results".to_owned(),
    }) {
        Ok(value) => value,
        Err(error) => panic!("create task failed: {error}"),
    };
    if let Err(error) = client.accept_task(&task_id, &assignee) {
        panic!("accept task failed: {error}");
    }
    let artifact_id = match client.submit_artifact(
        &task_id,
        Artifact {
            name: "report.md".to_owned(),
            bytes: b"summary".to_vec(),
        },
    ) {
        Ok(value) => value,
        Err(error) => panic!("submit artifact failed: {error}"),
    };
    assert!(artifact_id.0 > 0);

    if let Err(error) = client.complete_task(&task_id) {
        panic!("complete task failed: {error}");
    }
    let second_complete = client.complete_task(&task_id);
    assert_eq!(
        second_complete,
        Err(SdkError::Conflict("task already completed"))
    );
}

#[test]
fn get_artifact_status_returns_retained_status_for_known_artifact() {
    let mut client = InMemoryKamnClient::new();
    let creator = client
        .register(metadata("autonomous", "claude-4", &["research"]))
        .expect("register creator should succeed");
    let assignee = client
        .register(metadata("assistant", "gpt-5", &["research"]))
        .expect("register assignee should succeed");
    let task_id = client
        .create_task(TaskDefinition {
            creator,
            task_type: "analysis".to_owned(),
            description: "analyze benchmark results".to_owned(),
        })
        .expect("create task should succeed");
    client
        .accept_task(&task_id, &assignee)
        .expect("accept task should succeed");
    let artifact_id = client
        .submit_artifact(
            &task_id,
            Artifact {
                name: "report.md".to_owned(),
                bytes: b"summary".to_vec(),
            },
        )
        .expect("submit artifact should succeed");

    let status = client
        .get_artifact_status(&artifact_id)
        .expect("artifact status should succeed");

    assert_eq!(
        status,
        ArtifactStatus {
            artifact_id,
            lifecycle_state: "retained".to_owned(),
            redaction_status: "none".to_owned(),
        }
    );
}

#[test]
fn get_artifact_status_rejects_unknown_artifact() {
    let client = InMemoryKamnClient::new();

    assert_eq!(
        client.get_artifact_status(&kamn_sdk::ArtifactId(42)),
        Err(SdkError::NotFound {
            entity: "artifact",
            id: "42".to_owned(),
        })
    );
}

#[test]
fn expire_artifact_returns_expired_status_for_known_artifact() {
    let mut client = InMemoryKamnClient::new();
    let creator = client
        .register(metadata("autonomous", "claude-4", &["research"]))
        .expect("register creator should succeed");
    let assignee = client
        .register(metadata("assistant", "gpt-5", &["research"]))
        .expect("register assignee should succeed");
    let task_id = client
        .create_task(TaskDefinition {
            creator,
            task_type: "analysis".to_owned(),
            description: "analyze benchmark results".to_owned(),
        })
        .expect("create task should succeed");
    client
        .accept_task(&task_id, &assignee)
        .expect("accept task should succeed");
    let artifact_id = client
        .submit_artifact(
            &task_id,
            Artifact {
                name: "report.md".to_owned(),
                bytes: b"summary".to_vec(),
            },
        )
        .expect("submit artifact should succeed");

    let status = client
        .expire_artifact(&artifact_id)
        .expect("artifact expire should succeed");

    assert_eq!(
        status,
        ArtifactStatus {
            artifact_id: artifact_id.clone(),
            lifecycle_state: "expired".to_owned(),
            redaction_status: "none".to_owned(),
        }
    );
    assert_eq!(
        client
            .get_artifact_status(&artifact_id)
            .expect("status should reflect expiration"),
        status
    );
}

#[test]
fn expire_artifact_rejects_unknown_artifact() {
    let mut client = InMemoryKamnClient::new();

    assert_eq!(
        client.expire_artifact(&kamn_sdk::ArtifactId(43)),
        Err(SdkError::NotFound {
            entity: "artifact",
            id: "43".to_owned(),
        })
    );
}

#[test]
fn tombstone_artifact_returns_tombstoned_status_for_known_artifact() {
    let mut client = InMemoryKamnClient::new();
    let creator = client
        .register(metadata("autonomous", "claude-4", &["research"]))
        .expect("register creator should succeed");
    let assignee = client
        .register(metadata("assistant", "gpt-5", &["research"]))
        .expect("register assignee should succeed");
    let task_id = client
        .create_task(TaskDefinition {
            creator,
            task_type: "analysis".to_owned(),
            description: "analyze benchmark results".to_owned(),
        })
        .expect("create task should succeed");
    client
        .accept_task(&task_id, &assignee)
        .expect("accept task should succeed");
    let artifact_id = client
        .submit_artifact(
            &task_id,
            Artifact {
                name: "report.md".to_owned(),
                bytes: b"summary".to_vec(),
            },
        )
        .expect("submit artifact should succeed");

    let status = client
        .tombstone_artifact(&artifact_id)
        .expect("artifact tombstone should succeed");

    assert_eq!(
        status,
        ArtifactStatus {
            artifact_id: artifact_id.clone(),
            lifecycle_state: "tombstoned".to_owned(),
            redaction_status: "redacted".to_owned(),
        }
    );
    assert_eq!(
        client
            .get_artifact_status(&artifact_id)
            .expect("status should reflect tombstone"),
        status
    );
}

#[test]
fn tombstone_artifact_rejects_unknown_artifact() {
    let mut client = InMemoryKamnClient::new();

    assert_eq!(
        client.tombstone_artifact(&kamn_sdk::ArtifactId(44)),
        Err(SdkError::NotFound {
            entity: "artifact",
            id: "44".to_owned(),
        })
    );
}

#[test]
fn search_agents_filters_by_capability_and_reputation_exists() {
    let mut client = InMemoryKamnClient::new();
    let did_a = match client.register(metadata("autonomous", "claude-4", &["text", "code"])) {
        Ok(value) => value,
        Err(error) => panic!("register did_a failed: {error}"),
    };
    let _did_b = match client.register(metadata("assistant", "gpt-5", &["text"])) {
        Ok(value) => value,
        Err(error) => panic!("register did_b failed: {error}"),
    };

    let results = match client.search_agents(AgentQuery {
        capability: Some("code".to_owned()),
        model_family: None,
    }) {
        Ok(value) => value,
        Err(error) => panic!("search agents failed: {error}"),
    };
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].did, did_a.clone());

    let reputation = match client.get_reputation(&did_a) {
        Ok(value) => value,
        Err(error) => panic!("get reputation failed: {error}"),
    };
    assert_eq!(reputation.did, did_a);
    assert!(reputation.score > 0);
}

#[test]
fn create_channel_returns_deterministic_channel_id() {
    let mut client = InMemoryKamnClient::new();

    let first = client
        .create_channel("ops")
        .expect("first channel create should succeed");
    let second = client
        .create_channel("ops")
        .expect("second channel create should succeed");

    assert_eq!(first.0, "channel-local-1");
    assert_eq!(second.0, "channel-local-2");
}

#[test]
fn create_channel_rejects_empty_or_whitespace_name() {
    let mut client = InMemoryKamnClient::new();

    let error = client
        .create_channel("   ")
        .expect_err("whitespace-only channel name must fail");

    assert_eq!(
        error,
        SdkError::InvalidInput {
            field: "channel_name",
            reason: "must not be empty",
        }
    );
}

#[test]
fn did_parse_rejects_wrong_prefix() {
    let invalid = AgentDid::parse("did:example:abc").map_err(SdkError::from);
    assert_eq!(
        invalid,
        Err(SdkError::InvalidInput {
            field: "did",
            reason: "must start with kamn:did:agent:",
        })
    );

    let parsed = valid_did("alpha-1");
    assert_eq!(parsed.as_str(), "kamn:did:agent:alpha-1");
}

#[test]
fn register_rejects_empty_capability_entries() {
    // Regression: #583
    let mut client = InMemoryKamnClient::new();
    let result = client.register(metadata("autonomous", "claude-4", &["text", ""]));

    assert_eq!(
        result,
        Err(SdkError::InvalidInput {
            field: "capabilities",
            reason: "must not include empty capability entries",
        })
    );
}
