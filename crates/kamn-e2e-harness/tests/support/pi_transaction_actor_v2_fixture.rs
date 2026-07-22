#![allow(dead_code)]
#![allow(clippy::duplicate_mod)]

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
    });
    add_transaction_fields(&mut value);
    add_projection_fields(&mut value, role);
    value
}

fn add_transaction_fields(value: &mut Value) {
    value["task_id"] = json!("task-live-7099");
    value["transaction_id"] = json!("transaction-live-7099");
    value["escrow_id"] = json!("escrow-live-7099");
    value["amount_lamports"] = json!(1000000);
    value["network"] = json!("solana-devnet");
    value["settlement_tx_signature"] = json!("devnet-signature-7099");
    value["settlement_commitment"] = json!("finalized");
    value["receipt_chain_commitment"] = json!(sha('c'));
}

fn add_projection_fields(value: &mut Value, role: &str) {
    value["public_commitment"] = json!(sha('d'));
    value["view_scope"] = json!(scope(role));
    value["source_handoff_digest"] = json!(sha('b'));
    value["handoff_authorized"] = json!(false);
    if role != "agent_c" {
        value["participant_role"] = json!(if role == "agent_a" {
            "creator"
        } else {
            "provider"
        });
    }
}

fn receipts(role: &str, did: &str) -> Vec<Value> {
    match role {
        "agent_a" => agent_a_receipts(did),
        "agent_b" => agent_b_receipts(did),
        _ => Vec::new(),
    }
}

fn agent_a_receipts(did: &str) -> Vec<Value> {
    vec![create_receipt(did), fund_receipt(did), release_receipt(did)]
}

fn create_receipt(did: &str) -> Value {
    receipt(
        did,
        "create_task",
        "task:create",
        "task-live-7099",
        "submitted",
        "01",
        '6',
    )
}

fn fund_receipt(did: &str) -> Value {
    receipt(
        did,
        "fund_escrow",
        "escrow:fund",
        "escrow-live-7099",
        "funded",
        "02",
        '7',
    )
}

fn release_receipt(did: &str) -> Value {
    receipt(
        did,
        "release_escrow",
        "escrow:release-authorize",
        "escrow-live-7099",
        "release-authorized",
        "05",
        'a',
    )
}

fn agent_b_receipts(did: &str) -> Vec<Value> {
    vec![
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
    ]
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
    match role {
        "agent_c" => "restricted-public",
        _ => "participant-private",
    }
}

fn profile_marker(role: &str) -> char {
    match role {
        "agent_a" => 'a',
        "agent_b" => 'b',
        _ => 'c',
    }
}
