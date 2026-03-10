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
    let payload = format!("{{\"task_id\":\"{}\",\"state\":\"accepted\"}}", task_id);
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
    let payload = format!("{{\"task_id\":\"{}\",\"state\":\"completed\"}}", task_id);
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
    let payload = format!("{{\"escrow_id\":\"{}\",\"state\":\"funded\"}}", escrow_id);
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
    let payload = format!("{{\"escrow_id\":\"{}\",\"state\":\"released\"}}", escrow_id);
    write_http_response(stream, 200, payload.as_str())?;
    Ok(true)
}
