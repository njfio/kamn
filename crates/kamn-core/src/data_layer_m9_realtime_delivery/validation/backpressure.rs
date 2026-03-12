use crate::{
    data_layer_m9_realtime_delivery::{
        DataLayerM9RealtimeDeliveryError,
        DATA_LAYER_M9_RUNTIME_BACKPRESSURE_EVALUATION_FAILED_REASON_CODE,
        DATA_LAYER_M9_RUNTIME_BACKPRESSURE_INPUT_INVALID_REASON_CODE,
        DATA_LAYER_M9_RUNTIME_BACKPRESSURE_POLICY_INVALID_REASON_CODE,
    },
    RuntimeBackpressureError,
};

pub(crate) fn map_runtime_backpressure_policy_error_to_m9_projection_error(
    error: RuntimeBackpressureError,
) -> DataLayerM9RealtimeDeliveryError {
    DataLayerM9RealtimeDeliveryError::RuntimeBackpressurePolicyInvalid {
        reason_code: DATA_LAYER_M9_RUNTIME_BACKPRESSURE_POLICY_INVALID_REASON_CODE,
        detail: error.reason_code().to_owned(),
    }
}

pub(crate) fn map_runtime_backpressure_input_error_to_m9_projection_error(
    error: RuntimeBackpressureError,
) -> DataLayerM9RealtimeDeliveryError {
    DataLayerM9RealtimeDeliveryError::RuntimeBackpressureInputInvalid {
        reason_code: DATA_LAYER_M9_RUNTIME_BACKPRESSURE_INPUT_INVALID_REASON_CODE,
        detail: error.reason_code().to_owned(),
    }
}

pub(crate) fn map_runtime_backpressure_evaluation_error_to_m9_projection_error(
    error: RuntimeBackpressureError,
) -> DataLayerM9RealtimeDeliveryError {
    DataLayerM9RealtimeDeliveryError::RuntimeBackpressureEvaluationFailed {
        reason_code: DATA_LAYER_M9_RUNTIME_BACKPRESSURE_EVALUATION_FAILED_REASON_CODE,
        detail: error.reason_code().to_owned(),
    }
}
