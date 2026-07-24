use std::path::{Path, PathBuf};

const INTENT_MODEL: &str =
    "crates/kamn-node/src/service_api_endpoint/message_store/task_models.rs";
const RELEASE: &str = concat!(
    "crates/kamn-node/src/service_api_endpoint/middleware_impl/http_routes/",
    "mutations/update_routes/state_routes_release/live_settlement.rs"
);
const PROJECTION: &str =
    "crates/kamn-node/src/service_api_endpoint/message_store/task_projection/receipt_chain/settlement.rs";
const VERIFIER: &str =
    "crates/kamn-e2e-harness/src/mvp_demo/independent_settlement_verify.rs";
const RUNBOOK: &str = "docs/validation/bridge-authorized-escrow-settlement-slice.md";

#[test]
fn settlement_intent_binds_finalized_bridge_receipt() {
    assert_contains_all(
        &read(INTENT_MODEL),
        &[
            "bridge_id",
            "bridge_receipt_id",
            "bridge_receipt_digest",
            "bridge_transaction_signature",
        ],
        "settlement intent model",
    );
}

#[test]
fn release_consumes_bridge_transfer_without_resubmission() {
    assert_contains_all(
        &read(RELEASE),
        &[
            "validate_bridge_settlement_authority",
            "consume_finalized_bridge_receipt",
            "BRIDGE_SETTLEMENT_AUTHORITY_MISMATCH",
            "without_resubmission",
        ],
        "live settlement release",
    );
}

#[test]
fn projection_and_verifier_share_bridge_binding() {
    for (path, label) in [
        (PROJECTION, "task projection"),
        (VERIFIER, "independent verifier"),
    ] {
        assert_contains_all(
            &read(path),
            &["bridge_receipt_digest", "bridge_transaction_signature"],
            label,
        );
    }
}

#[test]
fn proof_runbook_preserves_single_transfer_boundary() {
    assert_contains_all(
        &read(RUNBOOK),
        &[
            "# Bridge-Authorized Escrow Settlement Slice",
            "exactly one economic transfer",
            "cross-resource receipt",
            "authority validation precedes settlement evidence validation",
            "not generalized settlement",
            "not production readiness",
        ],
        "bridge-authorized settlement runbook",
    );
}

fn assert_contains_all(content: &str, markers: &[&str], label: &str) {
    for marker in markers {
        assert!(content.contains(marker), "{label} missing marker: {marker}");
    }
}

fn read(relative: &str) -> String {
    let path = workspace_root().join(relative);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root should resolve")
}
