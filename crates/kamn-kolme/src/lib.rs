//! `kamn-kolme` hosts the extracted Kolme runtime-commit boundary.
//!
//! This initial scaffold keeps the API surface intentionally small while
//! extraction from `kamn-core` is in flight.
#![warn(missing_docs)]

pub mod api_codec;
pub mod block_scan_policy;
pub mod codec;
pub mod endpoint_policy;
pub mod finality;
pub mod notification_policy;
pub mod pipeline;
pub mod receipt_finality;
pub mod transport;

pub use api_codec::{
    KolmeApiBroadcastRequest, KolmeApiBroadcastResponse, KolmeApiCodecError,
    KolmeApiNextNonceRequest, KolmeApiNextNonceResponse,
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
pub use notification_policy::{
    parse_notification_event, KolmeNotificationEvent, KolmeNotificationPolicyError,
};
pub use pipeline::{PipelineError, RuntimeCommitPipeline};
pub use receipt_finality::{parse_receipt_finality, ReceiptFinality, ReceiptFinalityError};
pub use transport::{
    EchoTransport, KolmeTransport, TransportError, TransportRequest, TransportResponse,
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
