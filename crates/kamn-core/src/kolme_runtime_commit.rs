//! Deterministic runtime-commit request/receipt contracts for Kolme integration.

use crate::AgentDid;
#[cfg_attr(not(feature = "live-https"), allow(unused_imports))]
use kamn_kolme::{
    are_runtime_commit_request_fields_single_line as are_kolme_runtime_commit_request_fields_single_line_contract,
    build_kolme_fork_broadcast_live_provider_config as build_kamn_kolme_fork_broadcast_live_provider_config,
    build_runtime_commit_live_provider_config as build_kamn_kolme_runtime_commit_live_provider_config,
    classify_tls_failure_reason as classify_kolme_tls_failure_reason,
    classify_transport_io_error as classify_kolme_transport_io_error,
    commit_finality_from_receipt_finality as commit_finality_from_receipt_finality_contract,
    commit_finality_label as commit_finality_label_contract,
    compose_block_fallback_unresolved_reason as compose_kolme_block_fallback_unresolved_reason_contract,
    compose_finality_status_path as compose_kolme_finality_status_path,
    compose_notifications_reconnect_exhausted_reason as compose_kolme_notifications_reconnect_exhausted_reason_contract,
    compose_notifications_websocket_url as compose_kolme_notifications_websocket_url,
    deterministic_runtime_commit_id as deterministic_runtime_commit_id_contract,
    deterministic_runtime_commit_idempotency_key as deterministic_runtime_commit_idempotency_key_contract,
    find_http_header_boundary as find_kolme_http_header_boundary,
    is_broadcast_submit_path as is_kolme_broadcast_submit_path_contract,
    is_canonical_runtime_commit_signed_message as is_kolme_canonical_runtime_commit_signed_message_contract,
    is_terminal_receipt_finality as is_kolme_terminal_receipt_finality_contract,
    is_valid_block_fallback_base_url_input as is_kolme_valid_block_fallback_base_url_input_contract,
    is_valid_block_fallback_lookup_budget as is_kolme_valid_block_fallback_lookup_budget_contract,
    is_valid_block_fallback_provider_input as is_kolme_valid_block_fallback_provider_input_contract,
    is_valid_block_lookup_height as is_kolme_valid_block_lookup_height_contract,
    is_valid_expected_provider_input as is_kolme_valid_expected_provider_input_contract,
    is_valid_finality_base_url_input as is_kolme_valid_finality_base_url_input_contract,
    is_valid_finality_status_path_input as is_kolme_valid_finality_status_path_input_contract,
    is_valid_http_response_bytes_input as is_kolme_valid_http_response_bytes_input_contract,
    is_valid_http_transport_timeout_seconds as is_kolme_valid_http_transport_timeout_seconds_contract,
    is_valid_notifications_provider_input as is_kolme_valid_notifications_provider_input_contract,
    is_valid_notifications_reconnect_budget as is_kolme_valid_notifications_reconnect_budget_contract,
    is_valid_poll_attempt_budget as is_kolme_valid_poll_attempt_budget_contract,
    is_valid_receipt_commit_id_input as is_kolme_valid_receipt_commit_id_input_contract,
    is_valid_receipt_provider_input as is_kolme_valid_receipt_provider_input_contract,
    is_valid_runtime_commit_id_request as is_kolme_valid_runtime_commit_id_request_contract,
    is_valid_runtime_nonce_input as is_kolme_valid_runtime_nonce_input_contract,
    is_valid_runtime_operation_id_input as is_kolme_valid_runtime_operation_id_input_contract,
    is_valid_runtime_payload_hash_input as is_kolme_valid_runtime_payload_hash_input_contract,
    is_valid_runtime_provider_input as is_kolme_valid_runtime_provider_input_contract,
    is_valid_runtime_state_root_input as is_kolme_valid_runtime_state_root_input_contract,
    is_valid_signed_envelope_message_input as is_kolme_valid_signed_envelope_message_input_contract,
    is_valid_signed_envelope_signature_input as is_kolme_valid_signed_envelope_signature_input_contract,
    is_valid_signed_envelope_signer_key_id_input as is_kolme_valid_signed_envelope_signer_key_id_input_contract,
    is_valid_transport_idempotency_key_input as is_kolme_valid_transport_idempotency_key_input_contract,
    is_valid_transport_wire_payload_input as is_kolme_valid_transport_wire_payload_input_contract,
    is_valid_websocket_timeout_seconds as is_kolme_valid_websocket_timeout_seconds_contract,
    lifecycle_state_for_finality as lifecycle_state_for_finality_contract,
    lifecycle_state_label as lifecycle_state_label_contract,
    normalize_block_fallback_constructor_inputs as normalize_kolme_block_fallback_constructor_inputs_contract,
    normalize_broadcast_payload as normalize_kolme_broadcast_payload_contract,
    normalize_broadcast_submit_path_input as normalize_kolme_broadcast_submit_path_input_contract,
    normalize_finality_endpoint_inputs as normalize_kolme_finality_endpoint_inputs_contract,
    normalize_notifications_provider_input as normalize_kolme_notifications_provider_input_contract,
    normalize_runtime_commit_request_fields as normalize_kolme_runtime_commit_request_fields_contract,
    normalize_runtime_commit_signed_envelope_fields as normalize_kolme_runtime_commit_signed_envelope_fields_contract,
    normalize_transport_idempotency_key_input as normalize_kolme_transport_idempotency_key_input_contract,
    notification_event_to_provider_receipt as notification_event_to_kolme_provider_receipt_contract,
    parse_authorization_header_value as parse_kolme_authorization_header_value,
    parse_http_endpoint as parse_kolme_http_endpoint,
    parse_http_response_body as parse_kolme_http_response_body,
    parse_notification_event as parse_kolme_notification_event_contract,
    parse_provider_block_fallback_response as parse_kolme_provider_block_fallback_response_contract,
    parse_provider_finality_receipt as parse_kolme_provider_finality_receipt,
    parse_websocket_endpoint as parse_kolme_websocket_endpoint,
    project_failed_block_txhash_receipt as project_kolme_failed_block_txhash_receipt_contract,
    project_finalized_block_txhash_receipt as project_kolme_finalized_block_txhash_receipt_contract,
    render_block_path as render_kolme_block_path,
    render_runtime_commit_wire_payload as render_kolme_runtime_commit_wire_payload_contract,
    render_signed_envelope_wire_payload as render_kolme_signed_envelope_wire_payload_contract,
    require_commit_id_matches_expected_txhash as require_kolme_commit_id_matches_expected_txhash_contract,
    require_final_receipt_finality as require_kolme_final_receipt_finality_contract,
    resolve_lookup_upper_bound as resolve_kolme_lookup_upper_bound,
    resolve_tls_ca_file_env_result as resolve_kolme_tls_ca_file_env_result_contract,
    submit_runtime_commit_live_provider_request as submit_kamn_kolme_runtime_commit_live_provider_request,
    try_take_websocket_frame as try_take_kolme_websocket_frame,
    txhash_from_commit_id as txhash_from_kolme_commit_id,
    validate_block_identity as validate_kolme_block_identity,
    validate_block_path_template as validate_kolme_block_path_template,
    validate_lookup_txhash as validate_kolme_lookup_txhash_contract,
    validate_lookup_window as validate_kolme_lookup_window,
    validate_provider_receipt_identity as validate_kolme_provider_receipt_identity_contract,
    validate_websocket_handshake_response as validate_kolme_websocket_handshake_response,
    BlockScanPolicyError, KolmeApiBroadcastRequest as KamnKolmeApiBroadcastRequest,
    KolmeApiBroadcastResponse as KamnKolmeApiBroadcastResponse,
    KolmeApiCodecError as KamnKolmeApiCodecError,
    KolmeApiNextNonceRequest as KamnKolmeApiNextNonceRequest,
    KolmeApiNextNonceResponse as KamnKolmeApiNextNonceResponse,
    KolmeCommitReceiptFinality as KamnKolmeCommitReceiptFinality,
    KolmeHttpResponsePolicyError as KamnKolmeHttpResponsePolicyError,
    KolmeHttpScheme as KamnKolmeHttpScheme, KolmeNotificationEvent as KamnKolmeNotificationEvent,
    KolmeParsedHttpEndpoint as KamnKolmeParsedHttpEndpoint,
    KolmeProviderReceiptIdentityError as KamnKolmeProviderReceiptIdentityError,
    KolmeRuntimeCommitLiveProviderConfig as KamnKolmeRuntimeCommitLiveProviderConfig,
    KolmeRuntimeCommitLiveProviderConfigError as KamnKolmeRuntimeCommitLiveProviderConfigError,
    KolmeRuntimeProviderOutcome as KamnKolmeRuntimeProviderOutcome,
    KolmeTlsPolicyError as KamnKolmeTlsPolicyError,
    KolmeTransportRequestPolicyError as KamnKolmeTransportRequestPolicyError,
    KolmeWebsocketFrame as KamnKolmeWebsocketFrame,
    KolmeWebsocketPolicyError as KamnKolmeWebsocketPolicyError,
    RuntimeCommitLifecycleState as KamnKolmeRuntimeCommitLifecycleState,
    RuntimeLifecyclePolicyError as KamnKolmeRuntimeLifecyclePolicyError,
};

mod adapter_backed_client;
mod api_codec;
mod block_fallback_reconciler;
mod errors;
mod finality_checker;
mod fork_finality_resolver;
#[cfg(feature = "live-https")]
mod http_transport;
#[cfg(not(feature = "live-https"))]
mod http_transport_disabled;
mod in_memory_client;
mod interfaces;
mod live_provider;
mod notifications_consumer;
mod notifications_websocket;
mod outcomes;
mod request_envelope;
mod runtime_pipeline;

/// Finality classification for a runtime commit receipt.
pub type KolmeCommitReceiptFinality = KamnKolmeCommitReceiptFinality;

/// Runtime lifecycle state projected from commit receipt outcomes.
pub type RuntimeCommitLifecycleState = KamnKolmeRuntimeCommitLifecycleState;

pub use adapter_backed_client::AdapterBackedKolmeRuntimeCommitClient;
pub use api_codec::{
    KolmeApiBroadcastRequest, KolmeApiBroadcastResponse, KolmeApiNextNonceRequest,
    KolmeApiNextNonceResponse,
};
pub use block_fallback_reconciler::KolmeRuntimeCommitBlockFallbackReconciler;
pub use errors::KolmeRuntimeCommitError;
pub use finality_checker::KolmeRuntimeCommitFinalityChecker;
pub use fork_finality_resolver::KolmeRuntimeCommitForkFinalityResolver;
#[cfg(feature = "live-https")]
pub use http_transport::KolmeRuntimeCommitHttpTransport;
#[cfg(not(feature = "live-https"))]
pub use http_transport_disabled::KolmeRuntimeCommitHttpTransport;
pub use in_memory_client::InMemoryKolmeRuntimeCommitClient;
pub use interfaces::{KolmeRuntimeCommitClient, KolmeRuntimeCommitProvider};
pub use kamn_kolme::{
    KolmeRuntimeCommitBlockFallbackTransport, KolmeRuntimeCommitFinalityTransport,
    KolmeRuntimeCommitNotificationsConnection, KolmeRuntimeCommitNotificationsConnector,
    KolmeRuntimeCommitProviderError, KolmeRuntimeCommitProviderTransport,
    KolmeRuntimeCommitTransportErrorKind,
};
pub use live_provider::KolmeRuntimeCommitLiveProvider;
pub use notifications_consumer::KolmeRuntimeCommitNotificationsConsumer;
pub use notifications_websocket::{
    KolmeRuntimeCommitWebsocketConnection, KolmeRuntimeCommitWebsocketConnector,
};
pub use outcomes::{
    KolmeRuntimeCommitNotificationEvent, KolmeRuntimeCommitOutcome,
    KolmeRuntimeCommitProviderOutcome, KolmeRuntimeCommitProviderReceipt,
    KolmeRuntimeCommitReceipt,
};
pub use request_envelope::{KolmeRuntimeCommitRequest, KolmeRuntimeCommitSignedBroadcastEnvelope};
pub use runtime_pipeline::{
    RuntimeCommitFinalityProjection, RuntimeCommitLifecycleRecord, RuntimeCommitPipeline,
};

#[cfg(test)]
mod tests {
    use super::{
        classify_kolme_tls_failure_reason, KolmeRuntimeCommitError, KolmeRuntimeCommitRequest,
    };

    #[test]
    fn deterministic_request_rejects_empty_operation_id() {
        assert_eq!(
            KolmeRuntimeCommitRequest::deterministic(
                "",
                "state:abc",
                "kamn:did:agent:test-runtime",
                1,
                "payload:abc",
            ),
            Err(KolmeRuntimeCommitError::InvalidRequest {
                field: "operation_id",
                reason: "must not be empty",
            })
        );
    }

    #[test]
    fn tls_failure_reason_classifier_detects_certificate_errors() {
        let reason = classify_kolme_tls_failure_reason(
            "verify error:num=18:self-signed certificate\ncertificate verify failed",
        );
        assert_eq!(reason, "tls certificate verification failed");
    }

    #[test]
    fn tls_failure_reason_classifier_detects_handshake_errors() {
        let reason =
            classify_kolme_tls_failure_reason("ssl routines:ssl3_get_record:wrong version number");
        assert_eq!(reason, "tls handshake failed");
    }
}
