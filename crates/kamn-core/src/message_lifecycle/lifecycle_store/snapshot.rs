use super::transitions::{is_valid_transition, validate_snapshot_record};
use super::*;

impl MessageLifecycleStore {
    /// Exports all records into a deterministic snapshot payload model.
    pub fn export_snapshot(&self) -> MessageLifecycleSnapshot {
        let records = self
            .records
            .iter()
            .map(|(message_id, record)| MessageRecordSnapshot {
                message_id: message_id.clone(),
                sender: record.sender.clone(),
                recipients: record.recipients.clone(),
                created: record.created.clone(),
                expires: record.expires.clone(),
                status: record.status,
                history: record.history.clone(),
            })
            .collect();

        MessageLifecycleSnapshot {
            schema_version: MESSAGE_LIFECYCLE_SNAPSHOT_SCHEMA_VERSION,
            records,
        }
    }

    /// Restores all records from a previously exported lifecycle snapshot.
    pub fn restore_snapshot(
        &mut self,
        snapshot: MessageLifecycleSnapshot,
    ) -> Result<(), MessageLifecycleSnapshotError> {
        validate_snapshot_version(snapshot.schema_version)?;
        let mut records = BTreeMap::new();
        let mut ids_by_status = BTreeMap::new();
        let mut ids_by_sender = BTreeMap::new();
        let mut ids_by_recipient = BTreeMap::new();
        for record_snapshot in snapshot.records {
            restore_snapshot_record(
                &mut records,
                &mut ids_by_status,
                &mut ids_by_sender,
                &mut ids_by_recipient,
                record_snapshot,
            )?;
        }
        self.records = records;
        self.ids_by_status = ids_by_status;
        self.ids_by_sender = ids_by_sender;
        self.ids_by_recipient = ids_by_recipient;
        Ok(())
    }
}

fn validate_snapshot_version(schema_version: u16) -> Result<(), MessageLifecycleSnapshotError> {
    if schema_version == MESSAGE_LIFECYCLE_SNAPSHOT_SCHEMA_VERSION {
        return Ok(());
    }
    Err(MessageLifecycleSnapshotError::SnapshotVersionMismatch {
        expected: MESSAGE_LIFECYCLE_SNAPSHOT_SCHEMA_VERSION,
        found: schema_version,
    })
}

fn restore_snapshot_record(
    records: &mut BTreeMap<String, MessageRecord>,
    ids_by_status: &mut BTreeMap<MessageStatus, BTreeSet<String>>,
    ids_by_sender: &mut BTreeMap<String, BTreeSet<String>>,
    ids_by_recipient: &mut BTreeMap<String, BTreeSet<String>>,
    record_snapshot: MessageRecordSnapshot,
) -> Result<(), MessageLifecycleSnapshotError> {
    reject_duplicate_snapshot_id(records, &record_snapshot.message_id)?;
    validate_snapshot_record(&record_snapshot).map_err(MessageLifecycleSnapshotError::Lifecycle)?;
    validate_snapshot_history(&record_snapshot)?;
    index_snapshot_record(
        ids_by_status,
        ids_by_sender,
        ids_by_recipient,
        &record_snapshot,
    );
    records.insert(
        record_snapshot.message_id.clone(),
        snapshot_record_into_record(record_snapshot),
    );
    Ok(())
}

fn reject_duplicate_snapshot_id(
    records: &BTreeMap<String, MessageRecord>,
    message_id: &str,
) -> Result<(), MessageLifecycleSnapshotError> {
    if records.contains_key(message_id) {
        return Err(MessageLifecycleSnapshotError::DuplicateMessageId(
            message_id.to_owned(),
        ));
    }
    Ok(())
}

fn validate_snapshot_history(
    record_snapshot: &MessageRecordSnapshot,
) -> Result<(), MessageLifecycleSnapshotError> {
    let last_status = snapshot_history_last(record_snapshot)?;
    ensure_snapshot_history_starts_created(record_snapshot)?;
    ensure_snapshot_history_transitions(record_snapshot)?;
    ensure_snapshot_status_matches_history(record_snapshot, last_status)
}

fn snapshot_history_last(
    record_snapshot: &MessageRecordSnapshot,
) -> Result<MessageStatus, MessageLifecycleSnapshotError> {
    record_snapshot.history.last().copied().ok_or_else(|| {
        MessageLifecycleSnapshotError::InvalidSnapshot(format!(
            "history cannot be empty for {}",
            record_snapshot.message_id
        ))
    })
}

fn ensure_snapshot_history_starts_created(
    record_snapshot: &MessageRecordSnapshot,
) -> Result<(), MessageLifecycleSnapshotError> {
    if record_snapshot.history.first() == Some(&MessageStatus::Created) {
        return Ok(());
    }
    Err(MessageLifecycleSnapshotError::InvalidSnapshot(format!(
        "history must start with Created for {}",
        record_snapshot.message_id
    )))
}

fn ensure_snapshot_history_transitions(
    record_snapshot: &MessageRecordSnapshot,
) -> Result<(), MessageLifecycleSnapshotError> {
    for transition in record_snapshot.history.windows(2) {
        let (from, to) = (transition[0], transition[1]);
        if !is_valid_transition(from, to) {
            return Err(MessageLifecycleSnapshotError::InvalidSnapshot(format!(
                "invalid history transition for {}: {from:?}->{to:?}",
                record_snapshot.message_id
            )));
        }
    }
    Ok(())
}

fn ensure_snapshot_status_matches_history(
    record_snapshot: &MessageRecordSnapshot,
    last_status: MessageStatus,
) -> Result<(), MessageLifecycleSnapshotError> {
    if record_snapshot.status == last_status {
        return Ok(());
    }
    Err(MessageLifecycleSnapshotError::InvalidSnapshot(format!(
        "status/history mismatch for {}",
        record_snapshot.message_id
    )))
}

fn index_snapshot_record(
    ids_by_status: &mut BTreeMap<MessageStatus, BTreeSet<String>>,
    ids_by_sender: &mut BTreeMap<String, BTreeSet<String>>,
    ids_by_recipient: &mut BTreeMap<String, BTreeSet<String>>,
    record_snapshot: &MessageRecordSnapshot,
) {
    let message_id = record_snapshot.message_id.clone();
    ids_by_status
        .entry(record_snapshot.status)
        .or_default()
        .insert(message_id.clone());
    ids_by_sender
        .entry(record_snapshot.sender.clone())
        .or_default()
        .insert(message_id.clone());
    for recipient in &record_snapshot.recipients {
        ids_by_recipient
            .entry(recipient.clone())
            .or_default()
            .insert(message_id.clone());
    }
}

fn snapshot_record_into_record(record_snapshot: MessageRecordSnapshot) -> MessageRecord {
    MessageRecord {
        sender: record_snapshot.sender,
        recipients: record_snapshot.recipients,
        created: record_snapshot.created,
        expires: record_snapshot.expires,
        status: record_snapshot.status,
        history: record_snapshot.history,
    }
}
