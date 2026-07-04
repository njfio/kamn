use super::super::{
    lifecycle::{parse_kamn_did, validate_non_empty, validate_wrapped_keys},
    DataLayerM8ComplianceError, DataLayerM8ComplianceRegistry, DataLayerM8MessageRecord,
    DataLayerM8MessageRecordInput, DataLayerM8RetentionClass,
};

pub(super) fn register_message(
    registry: &mut DataLayerM8ComplianceRegistry,
    input: DataLayerM8MessageRecordInput,
) -> Result<DataLayerM8MessageRecord, DataLayerM8ComplianceError> {
    let owner_did = parse_kamn_did(input.owner_did.as_str())?;
    validate_message_input(&input)?;

    let owner_did_key = owner_did.as_str().to_owned();
    let owner_records = registry
        .messages_by_owner
        .entry(owner_did_key.clone())
        .or_default();
    reject_duplicate_message(owner_records, &owner_did_key, input.message_id.as_str())?;

    let record = build_record(owner_did.as_str(), owner_records.len() as u64 + 1, input);
    owner_records.push(record.clone());
    Ok(record)
}

fn validate_message_input(
    input: &DataLayerM8MessageRecordInput,
) -> Result<(), DataLayerM8ComplianceError> {
    validate_non_empty(input.message_id.as_str(), "message_id")?;
    validate_non_empty(input.content_hash.as_str(), "content_hash")?;
    validate_non_empty(input.hash_chain_prev.as_str(), "hash_chain_prev")?;
    if input.created_at_epoch_seconds == 0 {
        return Err(DataLayerM8ComplianceError::EmptyField(
            "created_at_epoch_seconds",
        ));
    }
    validate_wrapped_keys(&input.wrapped_keys)
}

fn reject_duplicate_message(
    owner_records: &[DataLayerM8MessageRecord],
    owner_did: &str,
    message_id: &str,
) -> Result<(), DataLayerM8ComplianceError> {
    if owner_records
        .iter()
        .any(|record| record.message_id == message_id)
    {
        return Err(DataLayerM8ComplianceError::DuplicateMessageId {
            owner_did: owner_did.to_owned(),
            message_id: message_id.to_owned(),
        });
    }
    Ok(())
}

fn build_record(
    owner_did: &str,
    sequence: u64,
    input: DataLayerM8MessageRecordInput,
) -> DataLayerM8MessageRecord {
    let mut wrapped_keys = input.wrapped_keys;
    sort_wrapped_keys(&mut wrapped_keys);

    DataLayerM8MessageRecord {
        owner_did: owner_did.to_owned(),
        message_id: input.message_id,
        created_at_epoch_seconds: input.created_at_epoch_seconds,
        content_hash: input.content_hash,
        hash_chain_prev: input.hash_chain_prev,
        retention_class: input.retention_class,
        retention_extension_seconds: input.retention_extension_seconds,
        wrapped_keys,
        legal_hold_active: matches!(input.retention_class, DataLayerM8RetentionClass::LegalHold),
        shredded_at_epoch_seconds: None,
        shred_reason_code: None,
        sequence,
    }
}

fn sort_wrapped_keys(wrapped_keys: &mut [super::super::DataLayerM8WrappedCekInput]) {
    wrapped_keys.sort_by(|left, right| {
        left.recipient_did
            .cmp(&right.recipient_did)
            .then(left.wrapped_cek.cmp(&right.wrapped_cek))
    });
}
