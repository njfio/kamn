#[path = "authoritative_settlement_entrypoint_parity/support.rs"]
mod support;

use kamn_agent_lib::KamnAgentHandle;
use kamn_e2e_harness::{
    verify_settlement_authority_parity, SettlementAuthorityAttempt, SettlementAuthorityDriver,
};
use serde_json::{json, Value};
use support::{
    authority_value, release_payload, StatefulAuthorityService, AGENT, ESCROW, IDEMPOTENCY,
};

#[test]
fn real_entrypoints_share_one_authority_and_submission() {
    std::env::set_var("KAMN_AGENT_LIB_ALLOW_DETERMINISTIC_IDENTITY", "1");
    let identity_handle =
        KamnAgentHandle::connect("http://127.0.0.1:9", "http://127.0.0.1:9", AGENT)
            .expect("identity handle");
    let actor = identity_handle.identity().did().as_str().to_owned();
    let service = StatefulAuthorityService::spawn(actor.as_str());
    let endpoint = service.endpoint();

    let sdk_handle =
        KamnAgentHandle::connect(&endpoint, "http://127.0.0.1:9", AGENT).expect("SDK handle");
    let sdk = sdk_handle
        .release_escrow_with_payload(ESCROW, release_payload().as_str())
        .expect("SDK release")
        .authoritative_settlement
        .expect("SDK authority");

    let cli_args = kamn_cli::parse_cli_args([
        "kamn-cli",
        "release-escrow",
        "--endpoint",
        endpoint.as_str(),
        ESCROW,
        IDEMPOTENCY,
        "bridge-1",
    ])
    .expect("CLI args");
    let cli_output = kamn_cli::dispatch(&cli_args).expect("CLI release");
    let cli = serde_json::from_str::<Value>(cli_output.json.as_str()).expect("CLI json");

    let mcp_handle =
        KamnAgentHandle::connect(&endpoint, "http://127.0.0.1:9", AGENT).expect("MCP handle");
    let mcp = kamn_mcp_server::dispatch_tool_request_json(
        &mcp_handle,
        json!({
            "id": "parity-1",
            "tool": "release_escrow",
            "escrow_id": ESCROW,
            "payload": release_payload(),
        })
        .to_string()
        .as_str(),
    )
    .expect("MCP release");
    let mcp = serde_json::from_str::<Value>(mcp.as_str()).expect("MCP json");

    service.finish();
    let attempts = vec![
        attempt(SettlementAuthorityDriver::Sdk, authority_value(&sdk)),
        attempt(SettlementAuthorityDriver::Cli, cli),
        attempt(
            SettlementAuthorityDriver::Mcp,
            mcp["result"]["settlement_service_receipt"].clone(),
        ),
    ];
    let report =
        verify_settlement_authority_parity(ESCROW, actor.as_str(), IDEMPOTENCY, attempts, 1)
            .expect("entrypoint parity");
    assert_eq!(report.settlement_submissions, 1);
}

fn attempt(driver: SettlementAuthorityDriver, response: Value) -> SettlementAuthorityAttempt {
    SettlementAuthorityAttempt {
        driver,
        escrow_id: ESCROW.to_owned(),
        idempotency_key: IDEMPOTENCY.to_owned(),
        response,
    }
}
