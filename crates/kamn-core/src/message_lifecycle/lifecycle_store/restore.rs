use super::validation::{build_message_record, is_valid_transition, validate_snapshot_record};
use super::{MessageLifecycleStore, RestoredSnapshotState};
use crate::message_lifecycle::{
    MessageLifecycleSnapshot, MessageLifecycleSnapshotError, MessageRecordSnapshot, MessageStatus,
    MESSAGE_LIFECYCLE_SNAPSHOT_SCHEMA_VERSION,
};
use std::collections::BTreeSet;

impl MessageLifecycleStore {
    /// Exports all records into a deterministic snapshot payload model.
    pub fn export_snapshot(&self) -> MessageLifecycleSnapshot {
        MessageLifecycleSnapshot {
            schema_version: MESSAGE_LIFECYCLE_SNAPSHOT_SCHEMA_VERSION,
            records: self.records.iter().map(snapshot_record).collect(),
        }
    }

    /// Restores all records from a previously exported lifecycle snapshot.
    pub fn restore_snapshot(
        &mut self,
        snapshot: MessageLifecycleSnapshot,
    ) -> Result<(), MessageLifecycleSnapshotError> {
        ensure_snapshot_version(snapshot.schema_version)?;
        let restored = restore_snapshot_state(snapshot.records)?;
        self.records = restored.records;
        self.ids_by_status = restored.ids_by_status;
        self.ids_by_sender = restored.ids_by_sender;
        self.ids_by_recipient = restored.ids_by_recipient;
        Ok(())
    }
}

fn snapshot_record(
    (message_id, record): (&String, &super::MessageRecord),
) -> MessageRecordSnapshot {
    MessageRecordSnapshot {
        message_id: message_id.clone(),
        sender: record.sender.clone(),
        recipients: record.recipients.clone(),
        created: record.created.clone(),
        expires: record.expires.clone(),
        status: record.status,
        history: record.history.clone(),
    }
}

fn ensure_snapshot_version(schema_version: u16) -> Result<(), MessageLifecycleSnapshotError> {
    if schema_version != MESSAGE_LIFECYCLE_SNAPSHOT_SCHEMA_VERSION {
        return Err(MessageLifecycleSnapshotError::SnapshotVersionMismatch {
            expected: MESSAGE_LIFECYCLE_SNAPSHOT_SCHEMA_VERSION,
            found: schema_version,
        });
    }
    Ok(())
}

fn restore_snapshot_state(
    records: Vec<MessageRecordSnapshot>,
) -> Result<RestoredSnapshotState, MessageLifecycleSnapshotError> {
    let mut restored = RestoredSnapshotState::default();
    for record_snapshot in records {
        restore_snapshot_record(&mut restored, record_snapshot)?;
    }
    Ok(restored)
}

fn restore_snapshot_record(
    restored: &mut RestoredSnapshotState,
    record_snapshot: MessageRecordSnapshot,
) -> Result<(), MessageLifecycleSnapshotError> {
    ensure_snapshot_message_id_is_unique(restored, &record_snapshot.message_id)?;
    validate_snapshot_record(&record_snapshot).map_err(MessageLifecycleSnapshotError::Lifecycle)?;
    validate_snapshot_history(&record_snapshot)?;
    insert_restored_indexes(restored, &record_snapshot);
    restored.records.insert(
        record_snapshot.message_id.clone(),
        build_message_record(
            record_snapshot.sender,
            record_snapshot.recipients,
            record_snapshot.created,
            record_snapshot.expires,
            record_snapshot.status,
            record_snapshot.history,
        ),
    );
    Ok(())
}

fn ensure_snapshot_message_id_is_unique(
    restored: &RestoredSnapshotState,
    message_id: &str,
) -> Result<(), MessageLifecycleSnapshotError> {
    if restored.records.contains_key(message_id) {
        return Err(MessageLifecycleSnapshotError::DuplicateMessageId(
            message_id.to_owned(),
        ));
    }
    Ok(())
}

fn validate_snapshot_history(
    record_snapshot: &MessageRecordSnapshot,
) -> Result<(), MessageLifecycleSnapshotError> {
    ensure_history_is_present(record_snapshot)?;
    ensure_history_starts_created(record_snapshot)?;
    ensure_history_transitions_are_valid(record_snapshot)?;
    ensure_status_matches_history(record_snapshot)
}

fn ensure_history_is_present(
    record_snapshot: &MessageRecordSnapshot,
) -> Result<(), MessageLifecycleSnapshotError> {
    if record_snapshot.history.is_empty() {
        return Err(invalid_snapshot(
            &record_snapshot.message_id,
            "history cannot be empty",
        ));
    }
    Ok(())
}

fn ensure_history_starts_created(
    record_snapshot: &MessageRecordSnapshot,
) -> Result<(), MessageLifecycleSnapshotError> {
    if record_snapshot.history[0] != MessageStatus::Created {
        return Err(invalid_snapshot(
            &record_snapshot.message_id,
            "history must start with Created",
        ));
    }
    Ok(())
}

fn ensure_history_transitions_are_valid(
    record_snapshot: &MessageRecordSnapshot,
) -> Result<(), MessageLifecycleSnapshotError> {
    for transition in record_snapshot.history.windows(2) {
        if !is_valid_transition(transition[0], transition[1]) {
            return Err(MessageLifecycleSnapshotError::InvalidSnapshot(format!(
                "invalid history transition for {}: {:?}->{:?}",
                record_snapshot.message_id, transition[0], transition[1]
            )));
        }
    }
    Ok(())
}

fn ensure_status_matches_history(
    record_snapshot: &MessageRecordSnapshot,
) -> Result<(), MessageLifecycleSnapshotError> {
    let last_status =
        record_snapshot.history.last().copied().ok_or_else(|| {
            invalid_snapshot(&record_snapshot.message_id, "history cannot be empty")
        })?;
    if record_snapshot.status != last_status {
        return Err(invalid_snapshot(
            &record_snapshot.message_id,
            "status/history mismatch",
        ));
    }
    Ok(())
}

fn insert_restored_indexes(
    restored: &mut RestoredSnapshotState,
    record_snapshot: &MessageRecordSnapshot,
) {
    let message_id = record_snapshot.message_id.clone();
    restored
        .ids_by_status
        .entry(record_snapshot.status)
        .or_insert_with(BTreeSet::new)
        .insert(message_id.clone());
    restored
        .ids_by_sender
        .entry(record_snapshot.sender.clone())
        .or_insert_with(BTreeSet::new)
        .insert(message_id.clone());
    for recipient in &record_snapshot.recipients {
        restored
            .ids_by_recipient
            .entry(recipient.clone())
            .or_insert_with(BTreeSet::new)
            .insert(message_id.clone());
    }
}

fn invalid_snapshot(message_id: &str, reason: &str) -> MessageLifecycleSnapshotError {
    MessageLifecycleSnapshotError::InvalidSnapshot(format!("{reason} for {message_id}"))
}
