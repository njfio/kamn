use std::collections::BTreeMap;

use kamn_core::{
    data_layer_m3_compute_blind_index, DataLayerM3MessageMetadataRecord, DataLayerM3SearchCatalog,
};

pub(crate) struct RecordSeed<'a> {
    pub(crate) message_id: &'a str,
    pub(crate) owner_did: &'a str,
    pub(crate) sender_did: &'a str,
    pub(crate) recipient_did: &'a str,
    pub(crate) session_id: Option<&'a str>,
    pub(crate) escrow_id: Option<&'a str>,
    pub(crate) message_type: &'a str,
    pub(crate) created_at_epoch_seconds: u64,
    pub(crate) blind_indexes: BTreeMap<String, String>,
}

pub(crate) fn record(seed: RecordSeed<'_>) -> DataLayerM3MessageMetadataRecord {
    DataLayerM3MessageMetadataRecord {
        message_id: seed.message_id.to_owned(),
        owner_did: seed.owner_did.to_owned(),
        sender_did: seed.sender_did.to_owned(),
        recipient_did: seed.recipient_did.to_owned(),
        session_id: seed.session_id.map(str::to_owned),
        escrow_id: seed.escrow_id.map(str::to_owned),
        message_type: seed.message_type.to_owned(),
        created_at_epoch_seconds: seed.created_at_epoch_seconds,
        blind_indexes: seed.blind_indexes,
    }
}

pub(crate) fn blind_index_map(entries: &[(&str, &str)]) -> BTreeMap<String, String> {
    entries
        .iter()
        .map(|(field, token)| ((*field).to_owned(), (*token).to_owned()))
        .collect()
}

pub(crate) fn derive_token(owner_key: &str, field_name: &str, value: &str) -> String {
    data_layer_m3_compute_blind_index(owner_key, field_name, value)
        .expect("blind index token should be derived")
}

pub(crate) fn register_record(
    catalog: &mut DataLayerM3SearchCatalog,
    seed: RecordSeed<'_>,
) {
    catalog
        .register_record(record(seed))
        .expect("record registration should succeed");
}

pub(crate) fn owner_a_text_record(
    message_id: &str,
    created_at_epoch_seconds: u64,
    blind_indexes: BTreeMap<String, String>,
) -> RecordSeed<'_> {
    RecordSeed {
        message_id,
        owner_did: "kamn:did:owner:a",
        sender_did: "kamn:did:agent:a1",
        recipient_did: "kamn:did:agent:b1",
        session_id: Some("session-a"),
        escrow_id: None,
        message_type: "text",
        created_at_epoch_seconds,
        blind_indexes,
    }
}
