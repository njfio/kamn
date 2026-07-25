use kamn_agent_lib::ServiceAuthoritativeSettlement;
use serde_json::{json, Value};
use std::collections::HashSet;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

pub const AGENT: &str = "kamn-cli";
pub const ESCROW: &str = "escrow-1";
pub const IDEMPOTENCY: &str = "operation-1";

pub struct StatefulAuthorityService {
    endpoint: String,
    submissions: Arc<Mutex<HashSet<String>>>,
    server: std::thread::JoinHandle<()>,
}

impl StatefulAuthorityService {
    pub fn spawn(actor: &str) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener");
        let endpoint = format!("http://{}", listener.local_addr().expect("address"));
        let submissions = Arc::new(Mutex::new(HashSet::new()));
        let server = serve(listener, service_response(actor), submissions.clone());
        Self {
            endpoint,
            submissions,
            server,
        }
    }

    pub fn endpoint(&self) -> String {
        self.endpoint.clone()
    }

    pub fn finish(self) {
        self.server.join().expect("server");
        assert_eq!(self.submissions.lock().expect("submissions").len(), 1);
    }
}

fn serve(
    listener: TcpListener,
    body: String,
    submissions: Arc<Mutex<HashSet<String>>>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        for _ in 0..3 {
            let (mut stream, _) = listener.accept().expect("connection");
            let request = read_request(&mut stream);
            assert!(request.contains(ESCROW), "request must bind shared escrow");
            assert!(
                request.contains(IDEMPOTENCY),
                "request must bind shared idempotency key"
            );
            submissions
                .lock()
                .expect("submissions")
                .insert(IDEMPOTENCY.to_owned());
            write_response(&mut stream, body.as_str());
        }
    })
}

fn read_request(stream: &mut std::net::TcpStream) -> String {
    let mut request = [0_u8; 8192];
    let length = stream.read(&mut request).expect("request");
    String::from_utf8(request[..length].to_vec()).expect("utf8 request")
}

fn write_response(stream: &mut std::net::TcpStream, body: &str) {
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    stream.write_all(response.as_bytes()).expect("response");
}

pub fn release_payload() -> String {
    json!({
        "idempotency_key": IDEMPOTENCY,
        "authority_mode": "bridge-receipt",
        "bridge_id": "bridge-1",
    })
    .to_string()
}

fn service_response(actor: &str) -> String {
    json!({
        "actor_did": actor, "escrow_id": ESCROW, "state": "release-authorized",
        "receipt_id": "release-1", "receipt_digest": digest('e'),
        "action": "escrow:release-authorize", "bridge_id": "bridge-1",
        "settlement_receipt_id": "settlement-1",
        "settlement_receipt_digest": digest('b'),
        "settlement_receipt_action": "settlement:confirmed",
        "settlement_receipt_resource_id": ESCROW,
        "settlement_receipt_state": "confirmed",
        "authoritative_settlement": authority(actor),
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
        "terms_digest": digest('d'), "idempotency_key": IDEMPOTENCY,
    })
}

pub fn authority_value(value: &ServiceAuthoritativeSettlement) -> Value {
    json!({
        "bridge_id": value.bridge_id, "bridge_receipt_id": value.bridge_receipt_id,
        "bridge_receipt_digest": value.bridge_receipt_digest,
        "settlement_receipt_id": value.settlement_receipt_id,
        "settlement_receipt_digest": value.settlement_receipt_digest,
        "action": value.action, "resource_id": value.resource_id,
        "actor_did": value.actor_did, "resulting_state": value.resulting_state,
        "task_id": value.task_id, "escrow_id": value.escrow_id,
        "recipient": value.recipient, "amount_lamports": value.amount_lamports,
        "asset": value.asset, "network": value.network,
        "transaction_signature": value.transaction_signature,
        "commitment": value.commitment, "finalized_slot": value.finalized_slot,
        "receipt_chain_commitment": value.receipt_chain_commitment,
        "terms_digest": value.terms_digest, "idempotency_key": value.idempotency_key,
    })
}

fn digest(value: char) -> String {
    format!("sha256:{}", value.to_string().repeat(64))
}
