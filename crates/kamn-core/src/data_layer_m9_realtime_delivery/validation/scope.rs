use crate::data_layer_m9_realtime_delivery::{
    DataLayerM9RealtimeDeliveryError, DATA_LAYER_M9_INVALID_OWNER_DID_REASON_CODE,
    DATA_LAYER_M9_INVALID_REQUESTER_OWNER_DID_REASON_CODE,
    DATA_LAYER_M9_OWNER_SCOPE_DENIED_REASON_CODE,
};

use super::parse_kamn_did;

pub(crate) fn authorize_owner_scope(
    requester_owner_did: &str,
    owner_did: &str,
) -> Result<(), DataLayerM9RealtimeDeliveryError> {
    let requester_owner_did = parse_kamn_did(
        requester_owner_did,
        "requester_owner_did",
        DATA_LAYER_M9_INVALID_REQUESTER_OWNER_DID_REASON_CODE,
    )?;
    let owner_did = parse_kamn_did(
        owner_did,
        "owner_did",
        DATA_LAYER_M9_INVALID_OWNER_DID_REASON_CODE,
    )?;
    if requester_owner_did.as_str() != owner_did.as_str() {
        return Err(DataLayerM9RealtimeDeliveryError::OwnerScopeViolation {
            reason_code: DATA_LAYER_M9_OWNER_SCOPE_DENIED_REASON_CODE,
        });
    }
    Ok(())
}
