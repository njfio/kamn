use std::collections::BTreeSet;

use crate::KamnDid;

use super::{
    DATA_LAYER_M8_OWNER_SCOPE_DENIED_REASON_CODE, DataLayerM8ComplianceError,
    DataLayerM8WrappedCekInput,
};

pub(crate) fn validate_non_empty(
    value: &str,
    field: &'static str,
) -> Result<(), DataLayerM8ComplianceError> {
    if value.trim().is_empty() {
        return Err(DataLayerM8ComplianceError::EmptyField(field));
    }
    Ok(())
}

pub(crate) fn parse_kamn_did(value: &str) -> Result<KamnDid, DataLayerM8ComplianceError> {
    KamnDid::parse(value).map_err(|_| DataLayerM8ComplianceError::InvalidDid(value.to_owned()))
}

pub(crate) fn validate_wrapped_keys(
    wrapped_keys: &[DataLayerM8WrappedCekInput],
) -> Result<(), DataLayerM8ComplianceError> {
    if wrapped_keys.is_empty() {
        return Err(DataLayerM8ComplianceError::EmptyWrappedKeys);
    }

    let mut seen_recipients = BTreeSet::new();
    for key in wrapped_keys {
        validate_wrapped_key(key, &mut seen_recipients)?;
    }
    Ok(())
}

fn validate_wrapped_key(
    key: &DataLayerM8WrappedCekInput,
    seen_recipients: &mut BTreeSet<String>,
) -> Result<(), DataLayerM8ComplianceError> {
    let recipient_did = parse_kamn_did(key.recipient_did.as_str())?;
    let recipient_did_key = recipient_did.as_str().to_owned();
    if !seen_recipients.insert(recipient_did_key.clone()) {
        return Err(DataLayerM8ComplianceError::DuplicateWrappedKeyRecipient {
            recipient_did: recipient_did_key,
        });
    }
    if key.wrapped_cek.trim().is_empty() {
        return Err(DataLayerM8ComplianceError::InvalidWrappedKey("wrapped_cek"));
    }
    Ok(())
}

pub(crate) fn authorize_owner_scope(
    requester_owner_did: &str,
    owner_did: &str,
) -> Result<KamnDid, DataLayerM8ComplianceError> {
    let requester_owner_did = parse_kamn_did(requester_owner_did)?;
    let owner_did = parse_kamn_did(owner_did)?;
    if requester_owner_did.as_str() != owner_did.as_str() {
        return Err(DataLayerM8ComplianceError::OwnerScopeViolation {
            reason_code: DATA_LAYER_M8_OWNER_SCOPE_DENIED_REASON_CODE,
        });
    }
    Ok(owner_did)
}
