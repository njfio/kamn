//! `kamn-kolme` hosts the extracted Kolme runtime-commit boundary.
//!
//! This initial scaffold keeps the API surface intentionally small while
//! extraction from `kamn-core` is in flight.
#![warn(missing_docs)]

pub mod api_codec;
pub mod block_fallback_policy;
pub mod block_scan_policy;
pub mod codec;
pub mod endpoint_policy;
pub mod finality;
pub mod flat_json_policy;
pub mod http_response_policy;
pub mod notification_policy;
pub mod pipeline;
pub mod provider_outcome_policy;
pub mod provider_response_policy;
pub mod receipt_finality;
pub mod tls_policy;
pub mod transport;
pub mod websocket_policy;

pub use api_codec::{
    validate_direct_signed_transaction_message, KolmeApiBroadcastRequest,
    KolmeApiBroadcastResponse, KolmeApiCodecError, KolmeApiNextNonceRequest,
    KolmeApiNextNonceResponse,
};
pub use block_fallback_policy::{
    parse_block_fallback_response, parse_fork_block_fallback_response,
    KolmeBlockFallbackPolicyError, KolmeBlockFallbackResponse,
};
pub use block_scan_policy::{
    parse_fork_block_txhash, render_block_path, validate_block_identity,
    validate_block_path_template, validate_lookup_window, BlockScanPolicyError,
};
pub use codec::{KolmeCodecError, KolmeWireCodec, PassthroughCodec};
pub use endpoint_policy::{
    compose_finality_status_path, compose_notifications_websocket_url, parse_http_endpoint,
    parse_websocket_endpoint, KolmeEndpointPolicyError, KolmeHttpScheme, KolmeParsedHttpEndpoint,
    KolmeParsedWebsocketEndpoint,
};
pub use finality::{resolve_finality, FinalityResolution, FinalityState};
pub use flat_json_policy::{
    parse_flat_json_value_fields, required_json_string_field, required_positive_u64_json_field,
    KolmeFlatJsonPolicyError, KolmeFlatJsonValue,
};
pub use http_response_policy::{parse_http_response_body, KolmeHttpResponsePolicyError};
pub use notification_policy::{
    parse_notification_event, KolmeNotificationEvent, KolmeNotificationPolicyError,
};
pub use pipeline::{PipelineError, RuntimeCommitPipeline};
pub use provider_outcome_policy::{
    deterministic_backend_commit_id, parse_commit_id_from_response_fields,
    parse_live_provider_outcome, required_provider_response_field, txhash_from_commit_id,
    KolmeProviderOutcome, KolmeProviderOutcomePolicyError,
};
pub use provider_response_policy::{
    parse_provider_key_value_fields, parse_provider_response_fields,
    KolmeProviderResponsePolicyError,
};
pub use receipt_finality::{parse_receipt_finality, ReceiptFinality, ReceiptFinalityError};
pub use tls_policy::{
    classify_tls_failure_reason, parse_tls_ca_file_env_value, KolmeTlsPolicyError,
};
pub use transport::{
    EchoTransport, KolmeTransport, TransportError, TransportRequest, TransportResponse,
};
pub use websocket_policy::{
    find_http_header_boundary, try_take_websocket_frame, validate_websocket_handshake_response,
    KolmeWebsocketFrame, KolmeWebsocketPolicyError,
};

#[cfg(test)]
mod tests {
    use super::{resolve_finality, FinalityState};

    #[test]
    fn unit_scaffold_exports_finality_resolution() {
        let resolution = resolve_finality(1, 1, false);
        assert_eq!(resolution.state(), FinalityState::Confirmed);
        assert!(resolution.is_final());
    }
}
