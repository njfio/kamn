use super::super::{
    DataLayerM8ComplianceError, DataLayerM8ComplianceRegistry, DataLayerM8MessageRecord,
    lifecycle::{parse_kamn_did, validate_non_empty},
};

pub(super) fn message_for_owner<'a>(
    registry: &'a DataLayerM8ComplianceRegistry,
    owner_did: &str,
    message_id: &str,
) -> Result<&'a DataLayerM8MessageRecord, DataLayerM8ComplianceError> {
    let owner_did = parse_kamn_did(owner_did)?;
    let owner_records = owner_records_or_error(registry, owner_did.as_str())?;
    owner_records
        .iter()
        .find(|record| record.message_id == message_id)
        .ok_or_else(|| DataLayerM8ComplianceError::MessageNotFound {
            owner_did: owner_did.as_str().to_owned(),
            message_id: message_id.to_owned(),
        })
}

pub(super) fn owner_records_or_error<'a>(
    registry: &'a DataLayerM8ComplianceRegistry,
    owner_did: &str,
) -> Result<&'a [DataLayerM8MessageRecord], DataLayerM8ComplianceError> {
    let owner_did = parse_kamn_did(owner_did)?;
    let owner_did_key = owner_did.as_str();
    registry
        .messages_by_owner
        .get(owner_did_key)
        .map(Vec::as_slice)
        .ok_or_else(|| DataLayerM8ComplianceError::OwnerNotFound {
            owner_did: owner_did_key.to_owned(),
        })
}

pub(super) fn owner_message_mut<'a>(
    registry: &'a mut DataLayerM8ComplianceRegistry,
    owner_did: &str,
    message_id: &str,
) -> Result<&'a mut DataLayerM8MessageRecord, DataLayerM8ComplianceError> {
    let owner_did = parse_kamn_did(owner_did)?;
    let owner_did_key = owner_did.as_str();
    validate_non_empty(message_id, "message_id")?;
    let owner_records = registry
        .messages_by_owner
        .get_mut(owner_did_key)
        .ok_or_else(|| DataLayerM8ComplianceError::OwnerNotFound {
            owner_did: owner_did_key.to_owned(),
        })?;
    owner_records
        .iter_mut()
        .find(|record| record.message_id == message_id)
        .ok_or_else(|| DataLayerM8ComplianceError::MessageNotFound {
            owner_did: owner_did_key.to_owned(),
            message_id: message_id.to_owned(),
        })
}
