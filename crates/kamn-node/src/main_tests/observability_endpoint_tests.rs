use super::*;
use crate::daemon_test_env_lock;
use crate::observability_endpoint::{
    enforce_observability_endpoint_payload_contract,
    set_observability_endpoint_tls_mode_override_for_current_thread_for_tests,
    validate_observability_endpoint_payload_contract, ObservabilityEndpointPayloadSurface,
    ObservabilityEndpointTlsModeOverride, RuntimeObservabilitySnapshot,
};
use std::sync::Barrier;

#[path = "observability_endpoint_tests/async_regression_contract_tests.rs"]
mod async_regression_contract_tests;
#[path = "observability_endpoint_tests/endpoint_runtime_contract_tests.rs"]
mod endpoint_runtime_contract_tests;
#[path = "observability_endpoint_tests/payload_contract_tests.rs"]
mod payload_contract_tests;
#[path = "observability_endpoint_tests/runtime_projection_contract_tests.rs"]
mod runtime_projection_contract_tests;
#[path = "observability_endpoint_tests/stream_runtime_contract_tests.rs"]
mod stream_runtime_contract_tests;
#[path = "observability_endpoint_tests/support.rs"]
mod support;
#[path = "observability_endpoint_tests/tls_contract_tests.rs"]
mod tls_contract_tests;
