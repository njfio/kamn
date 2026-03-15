use std::fs;

const DOC_PATH: &str = "docs/validation/live-escrow-settlement-slice.md";
const INDEX_PATH: &str = "docs/validation/current-proven-runtime-slices.md";
const REQUIRED_DOC_MARKERS: &[&str] = &[
    "external-execution `sdk-direct` S-05",
    "--enable-external-execution",
    "--scenarios S-05",
    "target/debug/kamn-e2e-harness verify",
    "not prove Solana-backed settlement",
    "not bridge settlement",
    "not external-chain settlement",
    "live escrow settlement slice: `docs/validation/live-escrow-settlement-slice.md`",
];

#[test]
fn live_escrow_settlement_doc_exists_and_stays_bounded() {
    let doc = fs::read_to_string(DOC_PATH)
        .unwrap_or_else(|error| panic!("live escrow settlement slice doc missing: {error}"));
    for marker in REQUIRED_DOC_MARKERS {
        assert!(
            doc.contains(marker),
            "live escrow settlement doc missing marker: {marker}"
        );
    }
}

#[test]
fn runtime_proof_index_includes_live_escrow_settlement_slice() {
    let index = fs::read_to_string(INDEX_PATH)
        .unwrap_or_else(|error| panic!("runtime proof index missing: {error}"));
    assert!(
        index.contains("live escrow settlement slice: `docs/validation/live-escrow-settlement-slice.md`"),
        "runtime proof index must link the live escrow settlement slice"
    );
}
