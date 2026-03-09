use super::*;

#[test]
fn unit_run_live_s11_mcp_signer_rotation_probe_accepts_rotation_continuity() {
    assert_scripted_probe_succeeds(
        "kamn-e2e-mcp-s11-success",
        write_mcp_s11_probe_script,
        &s11_updates(),
        run_live_s11_mcp_signer_rotation_probe,
    );
}

#[test]
fn unit_run_live_s14_mcp_batch_merkle_probe_accepts_distinct_batch_ids_and_final_proofs() {
    assert_scripted_probe_succeeds(
        "kamn-e2e-mcp-s14-success",
        write_mcp_s14_probe_script,
        &s14_updates(),
        run_live_s14_mcp_batch_merkle_probe,
    );
}

#[test]
fn unit_run_live_s15_mcp_performance_smoke_probe_accepts_bounded_latency_continuity() {
    assert_scripted_probe_succeeds(
        "kamn-e2e-mcp-s15-success",
        write_mcp_s15_probe_script,
        &s15_updates(),
        run_live_s15_mcp_performance_smoke_probe,
    );
}

fn s11_updates() -> [(&'static str, Option<&'static str>); 4] {
    [
        ("KAMN_ENDPOINT", Some("http://localhost:8080")),
        ("KAMN_AGENT_KEY_FILE", Some("/tmp/probe.key")),
        (
            "KAMN_E2E_S11_PRIMARY_AGENT_NAME",
            Some("kamn-e2e-mcp-s11-primary"),
        ),
        (
            "KAMN_E2E_S11_ROTATED_AGENT_NAME",
            Some("kamn-e2e-mcp-s11-rotated"),
        ),
    ]
}

fn s14_updates() -> [(&'static str, Option<&'static str>); 3] {
    [
        ("KAMN_ENDPOINT", Some("http://localhost:8080")),
        ("KAMN_AGENT_KEY_FILE", Some("/tmp/probe.key")),
        ("KAMN_E2E_S14_AGENT_NAME", Some("kamn-e2e-mcp-s14")),
    ]
}

fn s15_updates() -> [(&'static str, Option<&'static str>); 4] {
    [
        ("KAMN_ENDPOINT", Some("http://localhost:8080")),
        ("KAMN_AGENT_KEY_FILE", Some("/tmp/probe.key")),
        ("KAMN_E2E_S15_AGENT_NAME", Some("kamn-e2e-mcp-s15")),
        ("KAMN_E2E_S15_ITERATIONS", Some("3")),
    ]
}
