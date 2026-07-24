use kamn_agent_lib::KamnAgentHandle;
use kamn_e2e_harness::drivers::normalize_authoritative_settlement;
use serde_json::{json, Value};
use std::io::{Read, Write};
use std::net::TcpListener;

const AGENT: &str = "kamn-cli";
const ESCROW: &str = "escrow-1";
const IDEMPOTENCY: &str = "operation-1";

#[test]
fn sdk_cli_and_mcp_entrypoints_preserve_identical_authority() {
    std::env::set_var("KAMN_AGENT_LIB_ALLOW_DETERMINISTIC_IDENTITY", "1");
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
    let endpoint = format!("http://{}", listener.local_addr().expect("address"));
    let handle =
        KamnAgentHandle::connect(&endpoint, "http://127.0.0.1:9", AGENT).expect("SDK handle");
    let actor = handle.identity().did().as_str().to_owned();
    let response = service_response(actor.as_str());
    let server = serve(listener, response);

    let sdk = handle
        .release_escrow_with_payload(ESCROW, release_payload().as_str())
        .expect("SDK release")
        .authoritative_settlement
        .expect("SDK authority");
    let sdk = normalize_authoritative_settlement(&authority_value(&sdk), ESCROW, actor.as_str())
        .expect("SDK normalization");

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
    let cli_value = serde_json::from_str::<Value>(cli_output.json.as_str()).expect("CLI json");
    let cli = normalize_authoritative_settlement(&cli_value, ESCROW, actor.as_str())
        .expect("CLI normalization");

    let mcp_handle =
        KamnAgentHandle::connect(&endpoint, "http://127.0.0.1:9", AGENT).expect("MCP handle");
    let mcp_response = kamn_mcp_server::dispatch_tool_request_json(
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
    let mcp_value = serde_json::from_str::<Value>(mcp_response.as_str()).expect("MCP json");
    let mcp_authority = &mcp_value["result"]["settlement_service_receipt"];
    let mcp = normalize_authoritative_settlement(mcp_authority, ESCROW, actor.as_str())
        .expect("MCP normalization");

    server.join().expect("server");
    assert_eq!(sdk, cli);
    assert_eq!(cli, mcp);
}

fn serve(listener: TcpListener, body: String) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        for _ in 0..3 {
            let (mut stream, _) = listener.accept().expect("connection");
            let mut request = [0_u8; 8192];
            let _ = stream.read(&mut request).expect("request");
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).expect("response");
        }
    })
}

fn release_payload() -> String {
    json!({
        "idempotency_key": IDEMPOTENCY,
        "authority_mode": "bridge-receipt",
        "bridge_id": "bridge-1",
    })
    .to_string()
}

fn service_response(actor: &str) -> String {
    let authority = authority(actor);
    json!({
        "actor_did": actor, "escrow_id": ESCROW, "state": "release-authorized",
        "receipt_id": "release-1", "receipt_digest": digest('e'),
        "action": "escrow:release-authorize", "bridge_id": "bridge-1",
        "settlement_receipt_id": "settlement-1",
        "settlement_receipt_digest": digest('b'),
        "settlement_receipt_action": "settlement:confirmed",
        "settlement_receipt_resource_id": ESCROW,
        "settlement_receipt_state": "confirmed",
        "authoritative_settlement": authority,
    })
    .to_string()
}

fn authority(actor: &str) -> Value {
    json!({
        "bridge_id": "bridge-1", "bridge_receipt_id": "bridge-receipt-1",
        "bridge_receipt_digest": digest('a'), "settlement_receipt_id": "settlement-1",
        "settlement_receipt_digest": digest('b'), "action": "settlement:confirmed",
        "resource_id": ESCROW, "actor_did": actor, "resulting_state": "confirmed",
        "task_id": "task-1", "escrow_id": ESCROW, "recipient": "recipient-1",
        "amount_lamports": 31, "asset": "lamports", "network": "solana:devnet",
        "transaction_signature": "signature-1", "commitment": "finalized",
        "finalized_slot": 42, "receipt_chain_commitment": digest('c'),
        "terms_digest": "terms-1", "idempotency_key": IDEMPOTENCY,
    })
}

fn authority_value(value: &kamn_agent_lib::ServiceAuthoritativeSettlement) -> Value {
    json!({
        "bridge_id": value.bridge_id, "bridge_receipt_id": value.bridge_receipt_id,
        "bridge_receipt_digest": value.bridge_receipt_digest,
        "settlement_receipt_id": value.settlement_receipt_id,
        "settlement_receipt_digest": value.settlement_receipt_digest,
        "action": value.action, "resource_id": value.resource_id, "actor_did": value.actor_did,
        "resulting_state": value.resulting_state, "task_id": value.task_id,
        "escrow_id": value.escrow_id, "recipient": value.recipient,
        "amount_lamports": value.amount_lamports, "asset": value.asset,
        "network": value.network, "transaction_signature": value.transaction_signature,
        "commitment": value.commitment, "finalized_slot": value.finalized_slot,
        "receipt_chain_commitment": value.receipt_chain_commitment,
        "terms_digest": value.terms_digest, "idempotency_key": value.idempotency_key,
    })
}

fn digest(value: char) -> String {
    format!("sha256:{}", value.to_string().repeat(64))
}
