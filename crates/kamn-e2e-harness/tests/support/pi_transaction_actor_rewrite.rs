use std::path::Path;

use sha2::{Digest, Sha256};

pub(super) fn rebind_actor(path: &Path) {
    rewrite_actor(
        path,
        &[
            ("task-live-7099", "task-local-bound-7086"),
            ("transaction-live-7099", "task-local-bound-7086"),
            ("escrow-live-7099", "escrow-local-bound-7086"),
            ("devnet-signature-7099", "devnet-signature-111"),
        ],
    );
}

#[allow(dead_code)]
pub(super) fn reorder_actor_mutations(path: &Path) {
    rewrite_actor(
        path,
        &[
            ("\"tool\":\"create_task\"", "\"tool\":\"swap_task\""),
            ("\"tool\":\"fund_escrow\"", "\"tool\":\"create_task\""),
            ("\"tool\":\"swap_task\"", "\"tool\":\"fund_escrow\""),
        ],
    );
}

fn rewrite_actor(path: &Path, replacements: &[(&str, &str)]) {
    let raw = std::fs::read_to_string(path).expect("actor fixture");
    let marker = raw.rfind(",\"artifact_digest\":").expect("artifact digest");
    let mut unsigned = format!("{}}}", &raw[..marker]);
    for (from, to) in replacements {
        unsigned = unsigned.replace(from, to);
    }
    let digest = format!("sha256:{:x}", Sha256::digest(unsigned.as_bytes()));
    let artifact = format!(
        "{},\"artifact_digest\":\"{digest}\"}}",
        &unsigned[..unsigned.len() - 1]
    );
    std::fs::write(path, artifact).expect("rewrite actor fixture");
}
