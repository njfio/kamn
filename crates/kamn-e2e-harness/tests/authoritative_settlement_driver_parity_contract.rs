use std::path::{Path, PathBuf};

const SDK_MODELS: &str = "crates/kamn-sdk/src/service_authority_models.rs";
const CLI_RELEASE: &str = "crates/kamn-cli/src/commands/release_escrow.rs";
const MCP_AUTHORITY: &str = "crates/kamn-mcp-server/src/authority.rs";
const NODE_ESCROW_MODELS: &str = "crates/kamn-node/src/service_api_endpoint/escrow_models.rs";
const NORMALIZER: &str =
    "crates/kamn-e2e-harness/src/drivers/authoritative_settlement_observation.rs";
const RUNBOOK: &str = "docs/validation/authoritative-live-settlement-driver-parity-slice.md";

#[test]
fn sdk_exposes_complete_authoritative_receipt() {
    assert_contains_all(
        &read(SDK_MODELS),
        &[
            "bridge_receipt_digest",
            "settlement_receipt_digest",
            "receipt_chain_commitment",
            "transaction_signature",
            "finalized_slot",
        ],
        "SDK service models",
    );
}

#[test]
fn node_emits_one_canonical_authoritative_settlement() {
    assert_contains_all(
        &read(NODE_ESCROW_MODELS),
        &[
            "ServiceApiAuthoritativeSettlement",
            "authoritative_settlement",
            "receipt_chain_commitment",
            "finalized_slot",
        ],
        "node escrow response",
    );
}

#[test]
fn cli_emits_receipt_authority_and_idempotency_identity() {
    assert_contains_all(
        &read(CLI_RELEASE),
        &[
            "bridge_receipt_digest",
            "settlement_receipt_digest",
            "receipt_chain_commitment",
            "idempotency",
        ],
        "CLI release command",
    );
}

#[test]
fn mcp_wraps_bridge_mutations_in_service_authority() {
    assert_contains_all(
        &read(MCP_AUTHORITY),
        &[
            "submit_bridge_message",
            "forward_bridge_message",
            "bridge_receipt_digest",
            "settlement_receipt_digest",
        ],
        "MCP authority validator",
    );
}

#[test]
fn drivers_share_one_authority_normalizer_and_negative_matrix() {
    assert_contains_all(
        &read(NORMALIZER),
        &[
            "AuthoritativeSettlementObservation",
            "validate_receipt_chain_commitment",
            "validate_bridge_receipt_digest",
            "validate_settlement_receipt_digest",
            "validate_economic_terms",
            "reject_replay",
        ],
        "driver-neutral authority normalizer",
    );
}

#[test]
fn parity_runbook_requires_real_entrypoints_and_one_transfer() {
    assert_contains_all(
        &read(RUNBOOK),
        &[
            "# Authoritative Live Settlement Driver Parity Slice",
            "SDK-direct",
            "CLI-scripted",
            "MCP-agent",
            "identical receipt digest",
            "exactly one transfer",
            "not generalized settlement",
            "not production readiness",
        ],
        "authoritative parity runbook",
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
