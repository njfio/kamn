use super::super::{
    DATA_LAYER_M8_CEK_TOMBSTONE_MARKER, DATA_LAYER_M8_CRYPTO_SHRED_REASON_CODE,
    DataLayerM8ComplianceError, DataLayerM8ComplianceRegistry, DataLayerM8CryptoShredRequest,
    DataLayerM8LegalHoldRequest, DataLayerM8MessageRecord, DataLayerM8WrappedCekInput,
    lifecycle::authorize_owner_scope,
};

pub(super) fn set_legal_hold(
    registry: &mut DataLayerM8ComplianceRegistry,
    request: DataLayerM8LegalHoldRequest,
) -> Result<DataLayerM8MessageRecord, DataLayerM8ComplianceError> {
    let owner_did = authorize_owner_scope(
        request.requester_owner_did.as_str(),
        request.owner_did.as_str(),
    )?;
    let message = registry.owner_message_mut(owner_did.as_str(), request.message_id.as_str())?;
    reject_shredded_message(message)?;
    message.legal_hold_active = request.legal_hold_active;
    Ok(message.clone())
}

pub(super) fn crypto_shred(
    registry: &mut DataLayerM8ComplianceRegistry,
    request: DataLayerM8CryptoShredRequest,
) -> Result<DataLayerM8MessageRecord, DataLayerM8ComplianceError> {
    let owner_did = authorize_owner_scope(
        request.requester_owner_did.as_str(),
        request.owner_did.as_str(),
    )?;
    validate_shred_timestamp(request.shredded_at_epoch_seconds)?;
    let message = registry.owner_message_mut(owner_did.as_str(), request.message_id.as_str())?;
    reject_legal_hold(message)?;
    reject_shredded_message(message)?;
    apply_crypto_shred(message, request.shredded_at_epoch_seconds);
    Ok(message.clone())
}

fn validate_shred_timestamp(
    shredded_at_epoch_seconds: u64,
) -> Result<(), DataLayerM8ComplianceError> {
    if shredded_at_epoch_seconds == 0 {
        return Err(DataLayerM8ComplianceError::EmptyField(
            "shredded_at_epoch_seconds",
        ));
    }
    Ok(())
}

fn reject_legal_hold(message: &DataLayerM8MessageRecord) -> Result<(), DataLayerM8ComplianceError> {
    if message.legal_hold_active {
        return Err(DataLayerM8ComplianceError::LegalHoldActive {
            message_id: message.message_id.clone(),
        });
    }
    Ok(())
}

fn reject_shredded_message(
    message: &DataLayerM8MessageRecord,
) -> Result<(), DataLayerM8ComplianceError> {
    if message.shredded_at_epoch_seconds.is_some() {
        return Err(DataLayerM8ComplianceError::AlreadyShredded {
            message_id: message.message_id.clone(),
        });
    }
    Ok(())
}

fn apply_crypto_shred(message: &mut DataLayerM8MessageRecord, shredded_at_epoch_seconds: u64) {
    message.wrapped_keys = vec![DataLayerM8WrappedCekInput {
        recipient_did: "m8:crypto-shred:tombstone".to_owned(),
        wrapped_cek: DATA_LAYER_M8_CEK_TOMBSTONE_MARKER.to_owned(),
    }];
    message.shredded_at_epoch_seconds = Some(shredded_at_epoch_seconds);
    message.shred_reason_code = Some(DATA_LAYER_M8_CRYPTO_SHRED_REASON_CODE);
}
