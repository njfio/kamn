use super::super::super::*;
use crate::support::contract_server_support::strip_suffix_id;

pub(super) fn write_response(
    stream: &mut TcpStream,
    method: &str,
    path: &str,
    body: &str,
) -> Result<bool, String> {
    if write_accept_task_response(stream, method, path)? {
        return Ok(true);
    }
    if write_complete_task_response(stream, method, path)? {
        return Ok(true);
    }
    if write_fund_escrow_response(stream, method, path, body)? {
        return Ok(true);
    }
    if write_release_escrow_response(stream, method, path)? {
        return Ok(true);
    }
    Ok(false)
}

fn write_accept_task_response(
    stream: &mut TcpStream,
    method: &str,
    path: &str,
) -> Result<bool, String> {
    if method != "POST" || !path.starts_with("/v1/tasks/") || !path.ends_with("/accept") {
        return Ok(false);
    }
    let task_id = strip_suffix_id(path, "/v1/tasks/", "/accept");
    let payload = format!(
        "{{\"task_id\":\"{task_id}\",\"state\":\"accepted\",\"receipt_id\":\"task-transition-receipt-accept\",\"receipt_digest\":\"sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\"}}"
    );
    write_http_response(stream, 200, payload.as_str())?;
    Ok(true)
}

fn write_complete_task_response(
    stream: &mut TcpStream,
    method: &str,
    path: &str,
) -> Result<bool, String> {
    if method != "POST" || !path.starts_with("/v1/tasks/") || !path.ends_with("/complete") {
        return Ok(false);
    }
    let task_id = strip_suffix_id(path, "/v1/tasks/", "/complete");
    let payload = format!(
        "{{\"task_id\":\"{task_id}\",\"state\":\"completed\",\"receipt_id\":\"task-transition-receipt-complete\",\"receipt_digest\":\"sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc\"}}"
    );
    write_http_response(stream, 200, payload.as_str())?;
    Ok(true)
}

fn write_fund_escrow_response(
    stream: &mut TcpStream,
    method: &str,
    path: &str,
    body: &str,
) -> Result<bool, String> {
    if method != "POST" || path != "/v1/escrow/fund" {
        return Ok(false);
    }
    let escrow_id = format!("escrow-local-{:016x}", deterministic_tag(body.as_bytes()));
    let payload = format!(
        "{{\"escrow_id\":\"{escrow_id}\",\"state\":\"funded\",\"receipt_id\":\"escrow-transition-receipt-fund\",\"receipt_digest\":\"sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd\"}}"
    );
    write_http_response(stream, 200, payload.as_str())?;
    Ok(true)
}

fn write_release_escrow_response(
    stream: &mut TcpStream,
    method: &str,
    path: &str,
) -> Result<bool, String> {
    if method != "POST" || !path.starts_with("/v1/escrow/") || !path.ends_with("/release") {
        return Ok(false);
    }
    let escrow_id = strip_suffix_id(path, "/v1/escrow/", "/release");
    let payload = format!(
        "{{\"escrow_id\":\"{escrow_id}\",\"state\":\"released\",\"receipt_id\":\"escrow-transition-receipt-release\",\"receipt_digest\":\"sha256:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee\"}}"
    );
    write_http_response(stream, 200, payload.as_str())?;
    Ok(true)
}
