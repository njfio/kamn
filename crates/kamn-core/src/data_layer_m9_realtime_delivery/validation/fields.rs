use crate::data_layer_m9_realtime_delivery::DataLayerM9RealtimeDeliveryError;

pub(crate) fn validate_non_empty(
    value: &str,
    field: &'static str,
) -> Result<(), DataLayerM9RealtimeDeliveryError> {
    if value.trim().is_empty() {
        return Err(DataLayerM9RealtimeDeliveryError::EmptyField(field));
    }
    Ok(())
}

pub(crate) fn normalize_pair(left: &str, right: &str) -> (String, String) {
    if left <= right {
        (left.to_owned(), right.to_owned())
    } else {
        (right.to_owned(), left.to_owned())
    }
}
