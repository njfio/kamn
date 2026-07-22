use std::path::Path;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use super::receipt_fixture::sha;
#[path = "pi_transaction_actor_v2_overrides.rs"]
mod actor_overrides;
pub(crate) use actor_overrides::apply_overrides;
#[path = "service_authority_fixture.rs"]
mod service_authority_fixture;

pub(super) fn write_all(root: &Path) {
    write_actor(root, "agent-a.json", actor("agent_a", 101, "kamn:did:a"));
    write_actor(root, "agent-b.json", actor("agent_b", 202, "kamn:did:b"));
    write_actor(root, "agent-c.json", actor("agent_c", 303, "kamn:did:c"));
}

pub(super) fn write_bound_all(root: &Path) {
    for (name, role, pid, did) in [
        ("agent-a.json", "agent_a", 101, "kamn:did:a"),
        ("agent-b.json", "agent_b", 202, "kamn:did:b"),
        ("agent-c.json", "agent_c", 303, "kamn:did:c"),
    ] {
        let mut value = actor(role, pid, did);
        value["task_id"] = json!(service_authority_fixture::TASK);
        value["transaction_id"] = json!(service_authority_fixture::TRANSACTION);
        value["escrow_id"] = json!(service_authority_fixture::ESCROW);
        value["settlement_tx_signature"] = json!(service_authority_fixture::SIGNATURE);
        value["service_receipts"] = json!(service_authority_fixture::actor_receipts(role));
        value["receipt_chain_commitment"] = json!(service_authority_fixture::commitment());
        write_actor(root, name, value);
    }
}

fn actor(role: &str, process_id: u64, did: &str) -> Value {
    let mut value = json!({
        "schema_version": "kamn.mvp.pi-transaction-actor.v2",
        "actor": role,
        "pi_process_id": process_id,
        "did": did,
        "mcp_child_process_id": process_id + 1000,
        "first_request_id": 1,
        "last_request_id": 5,
        "transport_response_digests": [sha('1'), sha('2'), sha('3'), sha('4'), sha('5')],
        "service_profile_commitment": sha(profile_marker(role)),
        "service_receipts": receipts(role, did),
        "task_id": "task-live-7099",
        "transaction_id": "transaction-live-7099",
        "escrow_id": "escrow-live-7099",
        "amount_lamports": 1000000,
        "network": "solana-devnet",
        "settlement_tx_signature": "devnet-signature-7099",
        "settlement_commitment": "finalized",
        "receipt_chain_commitment": sha('c'),
        "public_commitment": sha('d'),
        "view_scope": scope(role),
        "source_handoff_digest": sha('b'),
        "handoff_authorized": false,
    });
    if role != "agent_c" {
        value["participant_role"] = json!(if role == "agent_a" {
            "creator"
        } else {
            "provider"
        });
    }
    value
}

fn receipts(role: &str, did: &str) -> Vec<Value> {
    match role {
        "agent_a" => vec![
            receipt(
                did,
                "create_task",
                "task:create",
                "task-live-7099",
                "submitted",
                "01",
                '6',
            ),
            receipt(
                did,
                "fund_escrow",
                "escrow:fund",
                "escrow-live-7099",
                "funded",
                "02",
                '7',
            ),
            receipt(
                did,
                "release_escrow",
                "escrow:release-authorize",
                "escrow-live-7099",
                "release-authorized",
                "05",
                'a',
            ),
        ],
        "agent_b" => vec![
            receipt(
                did,
                "accept_task",
                "task:accept",
                "task-live-7099",
                "accepted",
                "03",
                '8',
            ),
            receipt(
                did,
                "complete_task",
                "task:complete",
                "task-live-7099",
                "completed",
                "04",
                '9',
            ),
        ],
        _ => Vec::new(),
    }
}

fn receipt(
    did: &str,
    tool: &str,
    action: &str,
    resource_id: &str,
    resulting_state: &str,
    id: &str,
    marker: char,
) -> Value {
    json!({
        "actor_did": did,
        "tool": tool,
        "action": action,
        "resource_id": resource_id,
        "resulting_state": resulting_state,
        "service_receipt_id": format!("service-receipt-{id}"),
        "service_receipt_digest": sha(marker),
    })
}

pub(super) fn write_actor(root: &Path, name: &str, mut value: Value) {
    value
        .as_object_mut()
        .expect("v2 actor object")
        .remove("artifact_digest");
    let unsigned = serde_json::to_string(&value).expect("v2 actor JSON");
    let digest = format!("sha256:{:x}", Sha256::digest(unsigned.as_bytes()));
    let artifact = format!(
        "{},\"artifact_digest\":\"{digest}\"}}",
        &unsigned[..unsigned.len() - 1]
    );
    std::fs::write(root.join(name), artifact).expect("write v2 actor fixture");
}

fn scope(role: &str) -> &'static str {
    if role == "agent_c" {
        "restricted-public"
    } else {
        "participant-private"
    }
}

fn profile_marker(role: &str) -> char {
    match role {
        "agent_a" => 'a',
        "agent_b" => 'b',
        _ => 'c',
    }
}
