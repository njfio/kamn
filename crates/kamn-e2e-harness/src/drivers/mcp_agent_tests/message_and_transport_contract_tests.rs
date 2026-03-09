use super::*;

#[test]
fn unit_run_live_s03_mcp_group_channel_probe_rejects_query_message_id_mismatch() {
    assert_scripted_probe_error_contains(
        "kamn-e2e-mcp-s03-query-mismatch",
        |path| write_mcp_s03_probe_script(path, "message-2", "channel-1", true),
        &base_probe_updates(),
        "mismatched message_id",
        run_live_s03_mcp_group_channel_probe,
    );
}

#[test]
fn unit_run_live_s03_mcp_group_channel_probe_rejects_missing_messages_field() {
    assert_scripted_probe_error_contains(
        "kamn-e2e-mcp-s03-missing-messages",
        |path| write_mcp_s03_probe_script(path, "message-1", "channel-1", false),
        &base_probe_updates(),
        "missing messages field",
        run_live_s03_mcp_group_channel_probe,
    );
}

#[test]
fn unit_run_live_s06_mcp_proof_verification_probe_accepts_success_payload() {
    assert_scripted_probe_succeeds(
        "kamn-e2e-mcp-s06-success",
        |path| write_mcp_tool_response_script(path, "probe-verify-proof", PROOF_SUCCESS_PAYLOAD),
        &base_probe_updates(),
        run_live_s06_mcp_proof_verification_probe,
    );
}

#[test]
fn unit_run_live_s08_mcp_crash_recovery_probe_accepts_distinct_pre_post_message_ids() {
    assert_scripted_probe_succeeds(
        "kamn-e2e-mcp-s08-success",
        write_mcp_s08_probe_script,
        &single_endpoint_key_updates(),
        run_live_s08_mcp_crash_recovery_probe,
    );
}

#[test]
fn unit_run_live_s09_mcp_transport_failover_probe_accepts_distinct_pre_post_message_ids() {
    assert_scripted_probe_succeeds(
        "kamn-e2e-mcp-s09-success",
        write_mcp_s08_probe_script,
        &s09_updates(),
        run_live_s09_mcp_transport_failover_probe,
    );
}

#[test]
fn unit_run_live_s10_mcp_topology_coherence_probe_accepts_topology_query_continuity() {
    assert_scripted_probe_succeeds(
        "kamn-e2e-mcp-s10-success",
        write_mcp_s08_probe_script,
        &s10_updates(),
        run_live_s10_mcp_topology_coherence_probe,
    );
}

const PROOF_SUCCESS_PAYLOAD: &str =
    r#"{"ok":true,"finality":"FINAL","verified":true,"block_height":42}"#;

fn base_probe_updates() -> [(&'static str, Option<&'static str>); 3] {
    [
        ("KAMN_ENDPOINT", Some("http://localhost:8080")),
        ("KAMN_AGENT_NAME", Some("probe")),
        ("KAMN_AGENT_KEY_FILE", Some("/tmp/probe.key")),
    ]
}

fn single_endpoint_key_updates() -> [(&'static str, Option<&'static str>); 2] {
    [
        ("KAMN_ENDPOINT", Some("http://localhost:8080")),
        ("KAMN_AGENT_KEY_FILE", Some("/tmp/probe.key")),
    ]
}

fn s09_updates() -> [(&'static str, Option<&'static str>); 3] {
    [
        ("KAMN_ENDPOINT", Some("http://localhost:8080")),
        ("KAMN_E2E_S09_FAILOVER_ENDPOINT", Some("http://localhost:8081")),
        ("KAMN_AGENT_KEY_FILE", Some("/tmp/probe.key")),
    ]
}

fn s10_updates() -> [(&'static str, Option<&'static str>); 4] {
    [
        ("KAMN_E2E_S10_PRIMARY_ENDPOINT", Some("http://localhost:8080")),
        ("KAMN_E2E_S10_SECONDARY_ENDPOINT", Some("http://localhost:8081")),
        ("KAMN_E2E_S10_TERTIARY_ENDPOINT", Some("http://localhost:8082")),
        ("KAMN_AGENT_KEY_FILE", Some("/tmp/probe.key")),
    ]
}
