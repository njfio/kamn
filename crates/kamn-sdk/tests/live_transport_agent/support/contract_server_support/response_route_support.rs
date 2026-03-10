use super::*;

#[path = "response_route_support/agent_route_support.rs"]
mod agent_route_support;
#[path = "response_route_support/channel_route_support.rs"]
mod channel_route_support;
#[path = "response_route_support/message_route_support.rs"]
mod message_route_support;

pub(super) fn write_response(
    stream: &mut TcpStream,
    method: &str,
    path: &str,
    body: &str,
    state: &mut ContractServerState,
) -> Result<(), String> {
    if message_route_support::write_response(stream, method, path, body, state)? {
        return Ok(());
    }
    if agent_route_support::write_response(stream, method, path, body, state)? {
        return Ok(());
    }
    if channel_route_support::write_response(stream, method, path, body)? {
        return Ok(());
    }
    write_http_response(
        stream,
        404,
        r#"{"error":"not-found","reason_code":"service_api_route_not_found","message":"route not found"}"#,
    )
}
