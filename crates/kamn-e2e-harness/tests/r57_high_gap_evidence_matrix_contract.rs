use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
struct MatrixGap {
    checks: Vec<(String, String)>,
    workflow_markers: Vec<String>,
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .to_path_buf()
}

fn parse_matrix_fixture() -> (BTreeMap<String, String>, BTreeMap<String, MatrixGap>) {
    let root = repo_root();
    let fixture =
        std::fs::read_to_string(root.join("fixtures/runtime/r57_high_gap_evidence_matrix.txt"))
            .expect("r57 high-gap evidence matrix fixture should exist");

    let mut metadata = BTreeMap::new();
    let mut gaps: BTreeMap<String, MatrixGap> = BTreeMap::new();

    for line in fixture.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            metadata.insert(key.trim().to_owned(), value.trim().to_owned());
            continue;
        }
        if let Some(payload) = line.strip_prefix("gap|") {
            let gap_id = payload.trim();
            if !gap_id.is_empty() {
                gaps.entry(gap_id.to_owned()).or_default();
            }
            continue;
        }
        if let Some(payload) = line.strip_prefix("check|") {
            let mut parts = payload.split('|');
            let gap_id = parts.next().unwrap_or_default().trim();
            let path = parts.next().unwrap_or_default().trim();
            let test_fn = parts.next().unwrap_or_default().trim();
            if !gap_id.is_empty() && !path.is_empty() && !test_fn.is_empty() {
                gaps.entry(gap_id.to_owned())
                    .or_default()
                    .checks
                    .push((path.to_owned(), test_fn.to_owned()));
            }
            continue;
        }
        if let Some(payload) = line.strip_prefix("workflow_marker|") {
            let mut parts = payload.split('|');
            let gap_id = parts.next().unwrap_or_default().trim();
            let marker = parts.next().unwrap_or_default().trim();
            if !gap_id.is_empty() && !marker.is_empty() {
                gaps.entry(gap_id.to_owned())
                    .or_default()
                    .workflow_markers
                    .push(marker.to_owned());
            }
        }
    }

    (metadata, gaps)
}

#[test]
fn spec_c01_matrix_schema_and_required_gap_ids_are_present() {
    let (metadata, gaps) = parse_matrix_fixture();
    assert_eq!(
        metadata
            .get("r57_high_gap_evidence_matrix_schema_version")
            .map(String::as_str),
        Some("kamn.runtime.r57-high-gap-evidence-matrix.v1")
    );

    let observed_ids: BTreeSet<String> = gaps.keys().cloned().collect();
    let expected_ids: BTreeSet<String> = [
        "service_api_created_relayed_delivered_persistence",
        "service_api_relay_delivery_gate_before_projection",
        "e2e_live_workflow_wiring_and_fail_closed_execution",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    assert_eq!(observed_ids, expected_ids);
}

#[test]
fn spec_c02_matrix_references_existing_contract_tests() {
    let root = repo_root();
    let (_, gaps) = parse_matrix_fixture();
    for (gap_id, gap) in gaps {
        assert!(
            !gap.checks.is_empty(),
            "matrix gap should include at least one check: {gap_id}"
        );
        for (path, test_fn) in gap.checks {
            let source = std::fs::read_to_string(root.join(path.as_str()))
                .expect("matrix check path should exist");
            assert!(
                source.contains(format!("fn {test_fn}").as_str()),
                "expected matrix check fn marker in {path}: {test_fn}"
            );
        }
    }
}

#[test]
fn spec_c03_matrix_workflow_markers_exist_in_live_workflow() {
    let root = repo_root();
    let (_, gaps) = parse_matrix_fixture();
    let workflow = std::fs::read_to_string(root.join(".github/workflows/e2e-live.yml"))
        .expect("e2e-live workflow should exist");
    for (gap_id, gap) in gaps {
        for marker in gap.workflow_markers {
            assert!(
                workflow.contains(marker.as_str()),
                "expected workflow marker from matrix to exist for {gap_id}: {marker}"
            );
        }
    }
}
