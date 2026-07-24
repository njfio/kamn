use std::path::{Path, PathBuf};

const RUNBOOK: &str = "docs/validation/live-chain-backed-bridge-finality-slice.md";
const INDEX: &str = "docs/validation/current-proven-runtime-slices.md";
const DISPATCH: &str = "crates/kamn-node/src/service_api_endpoint/live_bridge_dispatch.rs";
const MODELS: &str = "crates/kamn-node/src/service_api_endpoint/message_store/models.rs";
const LIVE_TESTS: &str = concat!(
    "crates/kamn-node/src/main_tests/service_api_endpoint_tests/",
    "bridge_persistence_restart_contract_tests/live_bridge_contract_tests.rs"
);

#[test]
fn live_bridge_dispatch_uses_transaction_receipt_evidence() {
    let dispatch = read(DISPATCH);
    assert!(!dispatch.contains("solana-devnet-proof-"));
    assert_contains_all(
        &dispatch,
        &[
            "transaction_signature",
            "receipt_digest",
            "finalized_slot",
            "reconcile",
        ],
        "live bridge dispatch",
    );
}

#[test]
fn persisted_bridge_model_carries_canonical_receipt() {
    assert_contains_all(
        &read(MODELS),
        &["bridge_receipt", "transaction_signature", "finalized_slot"],
        "persisted bridge model",
    );
}

#[test]
fn service_contract_covers_finality_and_reconciliation() {
    assert_contains_all(
        &read(LIVE_TESTS),
        &[
            "persists_finalized_transaction_receipt",
            "rejects_mismatched_finality_evidence",
            "reconciles_before_resubmit",
            "observes_exactly_one_transfer",
        ],
        "live bridge service tests",
    );
}

#[test]
fn proof_runbook_and_index_preserve_bounded_claims() {
    assert_contains_all(
        &read(RUNBOOK),
        &[
            "# Live Chain-Backed Bridge Finality Slice",
            "real Solana devnet transaction",
            "independent RPC verification",
            "reconcile before resubmit",
            "not generalized cross-chain finality",
            "not production readiness",
        ],
        "live bridge finality runbook",
    );
    assert_contains_all(
        &read(INDEX),
        &[
            "live chain-backed bridge finality slice:",
            "proves one bounded finalized Solana devnet bridge receipt",
        ],
        "runtime proof index",
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
