use super::*;

#[test]
fn unit_run_live_s13_mcp_tool_call_rewrites_error_context() {
    assert_tool_call_error_contains("mcp live s13", || {
        run_live_s13_mcp_tool_call(
            "/definitely/missing/kamn-mcp-server",
            "http://localhost:8080",
            "probe-agent",
            "/tmp/probe.key",
            "probe-s13",
            "submit_bridge_message",
            "{}",
        )
    });
}

#[test]
fn unit_run_live_s14_mcp_tool_call_rewrites_error_context() {
    assert_tool_call_error_contains("mcp live s14", || {
        run_live_s14_mcp_tool_call(
            "/definitely/missing/kamn-mcp-server",
            "http://localhost:8080",
            "probe-agent",
            "/tmp/probe.key",
            "probe-s14",
            "verify_proof",
            "{}",
        )
    });
}

#[test]
fn unit_run_live_s15_mcp_tool_call_rewrites_error_context() {
    assert_tool_call_error_contains("mcp live s15", || {
        run_live_s15_mcp_tool_call(
            "/definitely/missing/kamn-mcp-server",
            "http://localhost:8080",
            "probe-agent",
            "/tmp/probe.key",
            "probe-s15",
            "query_message",
            "{}",
        )
    });
}

#[test]
fn unit_run_live_s04_mcp_tool_call_rejects_missing_binary() {
    assert_tool_call_error_contains("failed to spawn", s04_missing_binary_result);
}

#[test]
fn unit_run_live_s04_mcp_tool_call_rejects_non_success_exit_status() {
    with_executable_script(
        "kamn-e2e-mcp-tool-call-exit-1",
        NON_SUCCESS_SCRIPT,
        |script_path| {
            assert_tool_call_error_contains("exit_status=1", || s04_tool_call_result(script_path));
        },
    );
}

#[test]
fn unit_run_live_s04_mcp_tool_call_accepts_ok_true_payload() {
    with_tool_call_script("kamn-e2e-mcp-tool-call", r#"{"ok":true}"#, |script_path| {
        let payload = s04_tool_call_result(script_path).expect("ok=true payload should pass");
        assert!(payload.contains(r#""ok":true"#));
    });
}

#[test]
fn regression_issue_6214_run_live_s04_mcp_tool_call_rejects_nested_ok_true_when_root_false() {
    with_tool_call_script(
        "kamn-e2e-mcp-tool-call-root-false",
        ROOT_FALSE_PAYLOAD,
        |script_path| {
            assert_tool_call_error_contains("non-success payload", || {
                s04_tool_call_result(script_path)
            });
        },
    );
}

#[test]
fn unit_run_live_s04_mcp_tool_call_success_status_still_requires_framed_payloads() {
    with_executable_script(
        "kamn-e2e-mcp-tool-call-unframed",
        UNFRAMED_SUCCESS_SCRIPT,
        |script_path| {
            assert_tool_call_error_contains("invalid framed output", || {
                s04_tool_call_result(script_path)
            });
        },
    );
}

const ROOT_FALSE_PAYLOAD: &str = r#"{"ok":false,"detail":{"ok":true}}"#;
const NON_SUCCESS_SCRIPT: &str = r#"#!/usr/bin/env python3
import sys
sys.exit(1)
"#;
const UNFRAMED_SUCCESS_SCRIPT: &str = r#"#!/usr/bin/env python3
import sys
sys.stdout.write("not framed")
"#;

fn assert_tool_call_error_contains<F>(expected: &str, runner: F)
where
    F: FnOnce() -> Result<String, String>,
{
    let error = runner().expect_err("tool call should fail");
    assert!(
        error.contains(expected),
        "error should mention {expected}: {error}"
    );
}

fn s04_missing_binary_result() -> Result<String, String> {
    run_live_s04_mcp_tool_call(
        "/definitely/missing/kamn-mcp-server",
        "http://localhost:8080",
        "probe",
        "/tmp/probe.key",
        "probe-request",
        "health",
        "{}",
    )
}

fn s04_tool_call_result(script_path: &str) -> Result<String, String> {
    run_live_s04_mcp_tool_call(
        script_path,
        "http://localhost:8080",
        "probe",
        "/tmp/probe.key",
        "probe-request",
        "health",
        "{}",
    )
}

fn with_tool_call_script<F>(stem: &str, payload: &str, test: F)
where
    F: FnOnce(&str),
{
    let script_path = unique_temp_script_path(stem);
    write_mcp_tool_response_script(&script_path, "probe-request", payload);
    test(script_path_str(&script_path));
    remove_script(&script_path);
}

fn with_executable_script<F>(stem: &str, source: &str, test: F)
where
    F: FnOnce(&str),
{
    let script_path = unique_temp_script_path(stem);
    write_executable_python_script(&script_path, source);
    test(script_path_str(&script_path));
    remove_script(&script_path);
}
