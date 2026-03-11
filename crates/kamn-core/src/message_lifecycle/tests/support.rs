use super::{
    MessageLifecycleSnapshot, MessageLifecycleStore, MessageRecordSnapshot, MessageStatus,
    ProcessorProofArtifact,
};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) fn register_message(store: &mut MessageLifecycleStore, message_id: &str) {
    register_message_with_recipients(
        store,
        message_id,
        vec!["kamn:did:agent:recipient-1".to_owned()],
    );
}

pub(super) fn register_message_with_recipients(
    store: &mut MessageLifecycleStore,
    message_id: &str,
    recipients: Vec<String>,
) {
    store
        .register(
            message_id,
            "kamn:did:agent:sender-1",
            recipients,
            "2026-02-07T20:15:30.123Z",
            "2026-02-07T20:45:30.123Z",
        )
        .expect("register should succeed");
}

pub(super) fn advance_message(
    store: &mut MessageLifecycleStore,
    message_id: &str,
    statuses: &[MessageStatus],
) {
    for status in statuses {
        store
            .transition(message_id, *status)
            .expect("transition should succeed");
    }
}

pub(super) fn build_artifact(
    artifact_id: &str,
    message_id: &str,
    commitment: &str,
    proof: &str,
) -> ProcessorProofArtifact {
    ProcessorProofArtifact::new(artifact_id, message_id, commitment, proof)
        .expect("artifact should parse")
}

pub(super) fn snapshot_fixture(
    message_id: &str,
    status: MessageStatus,
    history: Vec<MessageStatus>,
) -> MessageLifecycleSnapshot {
    MessageLifecycleSnapshot {
        schema_version: 1,
        records: vec![MessageRecordSnapshot {
            message_id: message_id.to_owned(),
            sender: "kamn:did:agent:sender-1".to_owned(),
            recipients: vec![
                "kamn:did:agent:recipient-1".to_owned(),
                "kamn:did:agent:recipient-2".to_owned(),
            ],
            created: "2026-02-07T20:15:30.123Z".to_owned(),
            expires: "2026-02-07T20:45:30.123Z".to_owned(),
            status,
            history,
        }],
    }
}

pub(super) fn temp_message_lifecycle_snapshot_path(tag: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be monotonic")
        .as_nanos();
    std::env::temp_dir().join(format!("kamn-message-lifecycle-snapshot-{tag}-{nonce}.log"))
}

pub(super) fn temp_message_lifecycle_snapshot_journal_path(path: &Path) -> PathBuf {
    let mut journal = path.as_os_str().to_os_string();
    journal.push(".journal");
    PathBuf::from(journal)
}
