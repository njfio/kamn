use super::super::support::*;
use super::super::super::*;

#[test]
fn regression_service_api_endpoint_websocket_reason_taxonomy_includes_presence_did_invalid_headers()
{
    assert!(
        SERVICE_API_WEBSOCKET_REASON_CODES_CSV.contains(WS_PRESENCE_OWNER_DID_INVALID_REASON_CODE)
    );
    assert!(SERVICE_API_WEBSOCKET_REASON_CODES_CSV
        .contains(WS_PRESENCE_TARGET_OWNER_DID_INVALID_REASON_CODE));
    assert!(SERVICE_API_WEBSOCKET_REASON_CODES_CSV
        .contains(WS_PRESENCE_TARGET_AGENT_DID_INVALID_REASON_CODE));
}
