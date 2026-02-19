use std::collections::BTreeMap;

fn repo_path(relative: &str) -> String {
    format!("{}/{}", env!("CARGO_MANIFEST_DIR"), relative)
}

fn harness_source() -> String {
    std::fs::read_to_string(repo_path("tests/docs_contract_matrix_wave2_harness.rs"))
        .expect("docs_contract_matrix_wave2_harness.rs should exist")
}

fn parse_document_labels(source: &str) -> Vec<String> {
    source
        .lines()
        .map(str::trim)
        .filter_map(|line| {
            line.strip_prefix("document_label: \"")
                .and_then(|rest| rest.strip_suffix("\","))
                .map(ToOwned::to_owned)
        })
        .collect()
}

#[test]
fn regression_wave2_document_label_coverage_remains_complete() {
    // Regression: #5217
    let labels = parse_document_labels(harness_source().as_str());
    assert_eq!(labels.len(), 13, "wave2 document-label inventory drifted");

    let mut counts = BTreeMap::<String, usize>::new();
    for label in labels {
        *counts.entry(label).or_default() += 1;
    }

    for required in [
        "docs/ci/strategy.md",
        "docs/plans/2026-02-08-production-service-roadmap.md",
        "docs/foundation/data-governance-retention.md",
        "docs/foundation/key-management-and-encryption.md",
        "docs/planning/sdk-parity-wave.md",
        "docs/foundation/group-sender-key-rotation.md",
        "docs/ops/incident-readiness.md",
        "docs/foundation/python-sdk-beta.md",
        "docs/foundation/typescript-sdk-beta.md",
        "docs/foundation/service-marketplace-discovery.md",
        ".github/pull_request_template.md",
        "docs/testing/structure.md",
    ] {
        assert!(
            counts.contains_key(required),
            "missing required wave2 document label: {required}"
        );
    }

    assert_eq!(
        counts.get("docs/ci/strategy.md"),
        Some(&2),
        "docs/ci/strategy.md should back two wave2 cases (TLS + shell-surface policy markers)"
    );
}
