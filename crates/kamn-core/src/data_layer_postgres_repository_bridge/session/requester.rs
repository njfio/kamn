use crate::{AgentDid, KamnDid, DATA_LAYER_M2_REQUESTER_DID_SETTING};

use super::super::{
    DataLayerPgRepositoryBridgeError, DataLayerPgRequesterSession,
    DATA_LAYER_PG_INVALID_OWNER_DID_REASON_CODE, DATA_LAYER_PG_INVALID_REQUESTER_DID_REASON_CODE,
};

pub(crate) fn build_requester_session(
    requester_did: &str,
) -> Result<DataLayerPgRequesterSession, DataLayerPgRepositoryBridgeError> {
    let parsed = AgentDid::parse(requester_did).map_err(|error| {
        DataLayerPgRepositoryBridgeError::InvalidRequesterDid {
            field: "requester_did",
            reason_code: DATA_LAYER_PG_INVALID_REQUESTER_DID_REASON_CODE,
            detail: error.to_string(),
        }
    })?;
    Ok(DataLayerPgRequesterSession {
        setting_key: DATA_LAYER_M2_REQUESTER_DID_SETTING,
        requester_did: parsed.as_str().to_owned(),
    })
}

pub(crate) fn validate_owner_did(owner_did: &str) -> Result<(), DataLayerPgRepositoryBridgeError> {
    KamnDid::parse(owner_did).map_err(|error| {
        DataLayerPgRepositoryBridgeError::InvalidOwnerDid {
            field: "owner_did",
            reason_code: DATA_LAYER_PG_INVALID_OWNER_DID_REASON_CODE,
            detail: error.to_string(),
        }
    })?;
    Ok(())
}
