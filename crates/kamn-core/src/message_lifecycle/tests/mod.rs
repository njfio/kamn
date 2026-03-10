use super::{
    parse_message_lifecycle_snapshot_payload, serialize_message_lifecycle_snapshot,
    FileMessageLifecycleSnapshotStore, MessageLifecycleError, MessageLifecycleSnapshot,
    MessageLifecycleSnapshotError, MessageLifecycleSnapshotStore,
    MessageLifecycleSnapshotStoreError, MessageLifecycleStore, MessageProofAdmissionError,
    MessageRecordSnapshot, MessageStatus,
};
use crate::{ProcessorProofAdmissionEvaluator, ProcessorProofArtifact, ZkDesignError};
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

fn temp_message_lifecycle_snapshot_path(tag: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be monotonic")
        .as_nanos();
    std::env::temp_dir().join(format!("kamn-message-lifecycle-snapshot-{tag}-{nonce}.log"))
}

fn temp_message_lifecycle_snapshot_journal_path(path: &Path) -> PathBuf {
    let mut journal = path.as_os_str().to_os_string();
    journal.push(".journal");
    PathBuf::from(journal)
}

fn register_default_message(store: &mut MessageLifecycleStore, message_id: &str) {
    register_message(
        store,
        message_id,
        "kamn:did:agent:sender-1",
        vec!["kamn:did:agent:recipient-1".to_owned()],
    );
}

fn register_message(
    store: &mut MessageLifecycleStore,
    message_id: &str,
    sender: &str,
    recipients: Vec<String>,
) {
    store
        .register(
            message_id,
            sender,
            recipients,
            "2026-02-07T20:15:30.123Z",
            "2026-02-07T20:45:30.123Z",
        )
        .expect("register should succeed");
}

fn transition_to_validated(store: &mut MessageLifecycleStore, message_id: &str) {
    for status in [
        MessageStatus::Signed,
        MessageStatus::Broadcast,
        MessageStatus::Included,
        MessageStatus::Delivered,
        MessageStatus::Validated,
    ] {
        store
            .transition(message_id, status)
            .expect("message transition should succeed");
    }
}

fn sample_snapshot_record(
    message_id: &str,
    status: MessageStatus,
    history: Vec<MessageStatus>,
) -> MessageRecordSnapshot {
    MessageRecordSnapshot {
        message_id: message_id.to_owned(),
        sender: "kamn:did:agent:sender-1".to_owned(),
        recipients: vec!["kamn:did:agent:recipient-1".to_owned()],
        created: "2026-02-07T20:15:30.123Z".to_owned(),
        expires: "2026-02-07T20:45:30.123Z".to_owned(),
        status,
        history,
    }
}

fn sample_processor_artifact(
    artifact_id: &str,
    message_id: &str,
    proof: &str,
) -> ProcessorProofArtifact {
    ProcessorProofArtifact::new(artifact_id, message_id, "fnv1a64:abc", proof)
        .expect("artifact should parse")
}

include!("store_lifecycle.rs");
include!("proof_admission.rs");
include!("snapshot_codec.rs");
include!("file_snapshot_store.rs");
include!("performance.rs");
