use kamn_core::{
    cid_from_content_uri, content_uri_for_cid, ContentStorageAdapter, ContentStorageError,
    FileContentAdapter, TaskArtifactRegistry, TaskArtifactSubmission,
};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn unique_temp_file(tag: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should advance")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "kamn-content-file-adapter-{tag}-{}-{nonce}.log",
        std::process::id()
    ))
}

#[test]
fn content_storage_file_adapter_persists_round_trip_across_reopen() {
    let path = unique_temp_file("roundtrip");
    let mut adapter = FileContentAdapter::new(path.clone()).expect("store should build");
    let head = adapter
        .put("application/json", br#"{"task":"persist"}"#)
        .expect("put should succeed");

    let reopened = FileContentAdapter::new(path.clone()).expect("reopen should succeed");
    let object = reopened.get(&head.cid).expect("get should succeed");
    assert_eq!(object.media_type, "application/json");
    assert_eq!(object.payload, br#"{"task":"persist"}"#);

    fs::remove_file(path).expect("cleanup should succeed");
}

#[test]
fn content_storage_file_adapter_integration_supports_task_artifact_uri_reference() {
    let path = unique_temp_file("artifact");
    let mut adapter = FileContentAdapter::new(path.clone()).expect("store should build");
    let head = adapter
        .put("application/pdf", b"%PDF-1.7 file adapter")
        .expect("put should succeed");

    let uri = content_uri_for_cid(&head.cid).expect("uri generation should succeed");
    let cid = cid_from_content_uri(&uri).expect("uri should decode cid");
    assert_eq!(cid, head.cid);

    let mut registry = TaskArtifactRegistry::new();
    let on_chain_hash =
        TaskArtifactRegistry::integrity_fingerprint("task-file-1", "kamn:did:agent:file-1", &uri);
    registry
        .register(TaskArtifactSubmission {
            artifact_id: "artifact-file-1".to_owned(),
            task_id: "task-file-1".to_owned(),
            creator: "kamn:did:agent:file-1".to_owned(),
            created_at_unix: 1_716_000_000,
            on_chain_hash,
            off_chain_uri: uri,
            content_type: "application/pdf".to_owned(),
        })
        .expect("artifact registration should succeed");

    fs::remove_file(path).expect("cleanup should succeed");
}

#[test]
fn content_storage_file_adapter_regression_rejects_corrupt_payload_line() {
    // Regression: #2902
    let path = unique_temp_file("corrupt");
    fs::write(
        &path,
        "schema|kamn.content.file-store.v1\nobject|not-valid\n",
    )
    .expect("fixture write should succeed");

    let result = FileContentAdapter::new(path.clone());
    assert!(matches!(
        result,
        Err(ContentStorageError::InvalidPayload(value)) if value.contains("object|not-valid")
    ));

    fs::remove_file(path).expect("cleanup should succeed");
}
