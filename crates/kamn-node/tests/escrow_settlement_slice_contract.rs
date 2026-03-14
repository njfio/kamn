use std::fs;

const DOC: &str = "docs/validation/escrow-settlement-slice.md";
const INDEX: &str = "docs/validation/current-proven-runtime-slices.md";
const REQUIRED_DOC_MARKERS: &[&str] = &[
    "# Escrow Settlement Slice",
    "integration_service_api_endpoint_persists_task_and_escrow_state_across_routes",
    "integration_service_api_endpoint_persists_task_and_escrow_state_across_restart",
    "service-api escrow lifecycle persistence",
    "not bridge finality",
    "not external chain settlement",
];
const REQUIRED_INDEX_MARKERS: &[&str] = &[
    "escrow settlement slice: `docs/validation/escrow-settlement-slice.md`",
    "proves service-api escrow lifecycle persistence through fund, release, and restart-visible released state",
];

#[test]
fn escrow_settlement_validation_doc_exists_and_stays_bounded() {
    let doc = fs::read_to_string(DOC).expect("escrow settlement validation doc should exist");

    for marker in REQUIRED_DOC_MARKERS {
        assert!(
            doc.contains(marker),
            "escrow settlement validation doc missing marker: {marker}"
        );
    }
}

#[test]
fn runtime_proof_index_includes_escrow_settlement_slice() {
    let index = fs::read_to_string(INDEX).expect("runtime proof index should exist");

    for marker in REQUIRED_INDEX_MARKERS {
        assert!(
            index.contains(marker),
            "runtime proof index missing escrow settlement marker: {marker}"
        );
    }
}
