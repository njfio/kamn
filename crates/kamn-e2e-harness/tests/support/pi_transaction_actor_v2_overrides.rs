#![allow(dead_code)]

use std::path::Path;

use serde_json::{json, Value};

use super::super::{sha, Overrides};
use super::write_actor;

pub(crate) fn apply_overrides(root: &Path, overrides: &Overrides) {
    rewrite(root.join("agent-c.json").as_path(), |actor| {
        actor["pi_process_id"] = json!(overrides.agent_c_pid);
        actor["did"] = json!(overrides.agent_c_did);
        if overrides.agent_c_projection != sha('3') {
            actor["public_commitment"] = json!(overrides.agent_c_projection);
        }
        if overrides.agent_c_private.is_some() {
            actor["participant_role"] = json!("creator");
        }
    });
    rewrite(root.join("agent-b.json").as_path(), |actor| {
        actor["escrow_id"] = json!(overrides.agent_b_escrow);
    });
    rewrite(root.join("agent-a.json").as_path(), |actor| {
        apply_agent_a(actor, overrides)
    });
}

fn apply_agent_a(actor: &mut Value, overrides: &Overrides) {
    actor["handoff_authorized"] = if overrides.agent_a_handoff_as_string {
        json!(overrides.agent_a_handoff_authorized.to_string())
    } else {
        json!(overrides.agent_a_handoff_authorized)
    };
    let receipts = actor["service_receipts"].as_array_mut().expect("receipts");
    if !overrides.agent_a_include_release || overrides.agent_a_release_error {
        receipts.pop();
    } else if overrides.agent_a_duplicate_fund {
        receipts[2] = receipts[1].clone();
    } else if overrides.agent_a_receipt_digest_mismatch {
        receipts[2]["service_receipt_digest"] = json!(sha('f'));
    } else if overrides.agent_a_public_fact_drift {
        receipts[1]["resource_id"] = json!("escrow-other");
    }
}

fn rewrite(path: &Path, update: impl FnOnce(&mut Value)) {
    let raw = std::fs::read_to_string(path).expect("actor fixture");
    let mut actor: Value = serde_json::from_str(raw.as_str()).expect("actor JSON");
    actor
        .as_object_mut()
        .expect("actor object")
        .remove("artifact_digest");
    update(&mut actor);
    write_actor(
        path.parent().expect("actor root"),
        path.file_name()
            .expect("actor name")
            .to_str()
            .expect("UTF-8 name"),
        actor,
    );
}
