use crate::data_layer_m3_blind_index_search::{
    canonical_field_name, validate_blind_index_token, validate_kamn_did, validate_non_empty,
    DataLayerM3MessageMetadataRecord, DataLayerM3SearchCatalog, DataLayerM3SearchError,
};
use std::collections::BTreeMap;

impl DataLayerM3SearchCatalog {
    /// Creates an empty search catalog.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers one metadata record.
    pub fn register_record(
        &mut self,
        mut record: DataLayerM3MessageMetadataRecord,
    ) -> Result<(), DataLayerM3SearchError> {
        validate_record(&record, &self.seen_message_ids)?;
        record.blind_indexes = canonical_blind_indexes(record.blind_indexes)?;
        self.seen_message_ids.insert(record.message_id.clone());
        self.records.push(record);
        Ok(())
    }

    /// Returns immutable append-order records.
    pub fn records(&self) -> &[DataLayerM3MessageMetadataRecord] {
        &self.records
    }
}

fn validate_record(
    record: &DataLayerM3MessageMetadataRecord,
    seen_message_ids: &std::collections::BTreeSet<String>,
) -> Result<(), DataLayerM3SearchError> {
    validate_non_empty(record.message_id.as_str(), "message_id")?;
    validate_kamn_did(record.owner_did.as_str())?;
    validate_kamn_did(record.sender_did.as_str())?;
    validate_kamn_did(record.recipient_did.as_str())?;
    validate_non_empty(record.message_type.as_str(), "message_type")?;
    validate_optional_fields(record)?;
    validate_record_timestamp(record.created_at_epoch_seconds)?;
    validate_unique_message_id(record.message_id.as_str(), seen_message_ids)
}

fn validate_optional_fields(
    record: &DataLayerM3MessageMetadataRecord,
) -> Result<(), DataLayerM3SearchError> {
    if let Some(session_id) = record.session_id.as_deref() {
        validate_non_empty(session_id, "session_id")?;
    }
    if let Some(escrow_id) = record.escrow_id.as_deref() {
        validate_non_empty(escrow_id, "escrow_id")?;
    }
    Ok(())
}

fn validate_record_timestamp(created_at_epoch_seconds: u64) -> Result<(), DataLayerM3SearchError> {
    if created_at_epoch_seconds == 0 {
        return Err(DataLayerM3SearchError::EmptyField(
            "created_at_epoch_seconds",
        ));
    }
    Ok(())
}

fn validate_unique_message_id(
    message_id: &str,
    seen_message_ids: &std::collections::BTreeSet<String>,
) -> Result<(), DataLayerM3SearchError> {
    if seen_message_ids.contains(message_id) {
        return Err(DataLayerM3SearchError::DuplicateMessageId(
            message_id.to_owned(),
        ));
    }
    Ok(())
}

fn canonical_blind_indexes(
    indexes: BTreeMap<String, String>,
) -> Result<BTreeMap<String, String>, DataLayerM3SearchError> {
    let mut canonical = BTreeMap::new();
    for (field_name, token) in indexes {
        let field_name = canonical_field_name(field_name.as_str())?;
        validate_blind_index_token(field_name.as_str(), token.as_str())?;
        canonical.insert(field_name, token.trim().to_owned());
    }
    Ok(canonical)
}
