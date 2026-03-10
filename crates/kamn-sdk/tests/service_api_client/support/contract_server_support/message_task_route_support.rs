use super::*;

#[path = "message_task_route_support/channel_task_route_support.rs"]
mod channel_task_route_support;
#[path = "message_task_route_support/message_route_support.rs"]
mod message_route_support;
#[path = "message_task_route_support/task_mutation_route_support.rs"]
mod task_mutation_route_support;

pub(super) fn write_response(
    stream: &mut TcpStream,
    method: &str,
    path: &str,
    body: &str,
) -> Result<bool, String> {
    if message_route_support::write_response(stream, method, path, body)? {
        return Ok(true);
    }
    if channel_task_route_support::write_response(stream, method, path, body)? {
        return Ok(true);
    }
    task_mutation_route_support::write_response(stream, method, path, body)
}
