use crate::{
    data_layer_m9_realtime_delivery::DataLayerM9RealtimeDeliveryError, AgentDid, AgentDidError,
    KamnDid, KamnDidError,
};

pub(crate) fn map_agent_did_error(
    error: AgentDidError,
    field: &'static str,
    reason_code: &'static str,
) -> DataLayerM9RealtimeDeliveryError {
    DataLayerM9RealtimeDeliveryError::InvalidDid {
        field,
        reason_code,
        detail: error.to_string(),
    }
}

pub(crate) fn map_kamn_did_error(
    error: KamnDidError,
    field: &'static str,
    reason_code: &'static str,
) -> DataLayerM9RealtimeDeliveryError {
    DataLayerM9RealtimeDeliveryError::InvalidDid {
        field,
        reason_code,
        detail: error.to_string(),
    }
}

pub(crate) fn parse_agent_did(
    value: &str,
    field: &'static str,
    reason_code: &'static str,
) -> Result<AgentDid, DataLayerM9RealtimeDeliveryError> {
    AgentDid::parse(value).map_err(|error| map_agent_did_error(error, field, reason_code))
}

pub(crate) fn parse_kamn_did(
    value: &str,
    field: &'static str,
    reason_code: &'static str,
) -> Result<KamnDid, DataLayerM9RealtimeDeliveryError> {
    KamnDid::parse(value).map_err(|error| map_kamn_did_error(error, field, reason_code))
}
