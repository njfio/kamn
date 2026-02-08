use kamn_core::{
    cid_from_content_uri, content_uri_for_cid, ContentStorageAdapter, ContentStorageError,
    InMemoryContentAdapter, TaskArtifactRegistry, TaskArtifactSubmission,
};

#[test]
fn content_storage_adapter_generates_deterministic_cid_for_same_payload() {
    let mut adapter = InMemoryContentAdapter::new();
    let first = adapter
        .put("application/json", br#"{"task":"analyze"}"#)
        .expect("first put should succeed");
    let second = adapter
        .put("application/json", br#"{"task":"analyze"}"#)
        .expect("second put should succeed");

    assert_eq!(first.cid, second.cid);
    assert_eq!(first.integrity_tag, second.integrity_tag);
}

#[test]
fn content_storage_adapter_put_get_head_verify_round_trip() {
    let mut adapter = InMemoryContentAdapter::new();
    let head = adapter
        .put("text/plain", b"hello-kamn")
        .expect("put should succeed");

    let object = adapter.get(&head.cid).expect("get should succeed");
    assert_eq!(object.media_type, "text/plain");
    assert_eq!(object.payload, b"hello-kamn");

    let fetched_head = adapter.head(&head.cid).expect("head should succeed");
    assert_eq!(fetched_head.size_bytes, head.size_bytes);

    adapter.verify(&head.cid).expect("verify should succeed");
}

#[test]
fn content_storage_adapter_integration_supports_task_artifact_uri_reference() {
    let mut adapter = InMemoryContentAdapter::new();
    let head = adapter
        .put("application/pdf", b"%PDF-1.7 artifact content")
        .expect("put should succeed");

    let uri = content_uri_for_cid(&head.cid).expect("uri generation should succeed");
    let cid = cid_from_content_uri(&uri).expect("uri should decode cid");
    assert_eq!(cid, head.cid);

    let mut registry = TaskArtifactRegistry::new();
    let on_chain_hash =
        TaskArtifactRegistry::integrity_fingerprint("task-88", "kamn:did:agent:builder-1", &uri);
    registry
        .register(TaskArtifactSubmission {
            artifact_id: "artifact-88".to_owned(),
            task_id: "task-88".to_owned(),
            creator: "kamn:did:agent:builder-1".to_owned(),
            created_at_unix: 1_716_000_000,
            on_chain_hash,
            off_chain_uri: uri,
            content_type: "application/pdf".to_owned(),
        })
        .expect("artifact registration should succeed");
}

#[test]
fn content_storage_adapter_rejects_invalid_cid_lookup() {
    let adapter = InMemoryContentAdapter::new();
    assert_eq!(
        adapter.get("kamn:cid:v1:nothexvalue"),
        Err(ContentStorageError::InvalidCid(
            "kamn:cid:v1:nothexvalue".to_owned()
        ))
    );
}

#[test]
fn content_storage_adapter_regression_detects_tampered_payload() {
    // Regression: #169
    let mut adapter = InMemoryContentAdapter::new();
    let head = adapter
        .put("application/octet-stream", b"original")
        .expect("put should succeed");
    adapter
        .replace_payload_unchecked(&head.cid, b"tampered".to_vec())
        .expect("tamper operation should succeed");

    let result = adapter.verify(&head.cid);
    assert!(matches!(
        result,
        Err(ContentStorageError::IntegrityMismatch { .. })
    ));
}
