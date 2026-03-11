use std::{fs, path::PathBuf};

const ROOT_MAX_LINES: usize = 25;
const HELPER_MARKERS: &[&str] = &[
    "fn build_runtime_evidence_identities(",
    "fn build_runtime_evidence_m0_to_m1(",
    "fn build_runtime_evidence_m2_to_m5(",
    "fn build_runtime_evidence_m6_to_m11(",
    "fn assemble_runtime_evidence_record(",
];

#[test]
fn message_store_runtime_evidence_function_stays_within_active_budget() {
    let source = read_message_store_source();
    let line_count = function_line_count(source.as_str(), "fn build_data_layer_runtime_evidence(")
        .expect("build_data_layer_runtime_evidence should exist");
    assert!(
        line_count <= ROOT_MAX_LINES,
        "build_data_layer_runtime_evidence should stay within {} lines, found {}",
        ROOT_MAX_LINES,
        line_count
    );
}

#[test]
fn message_store_runtime_evidence_declares_extracted_helper_seams() {
    let source = read_message_store_source();
    for marker in HELPER_MARKERS {
        assert!(
            source.contains(marker),
            "message_store.rs should declare extracted runtime evidence helper marker: {marker}"
        );
    }
}

fn read_message_store_source() -> String {
    fs::read_to_string(message_store_path()).expect("message_store source should read")
}

fn message_store_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("service_api_endpoint")
        .join("message_store.rs")
}

fn function_line_count(source: &str, marker: &str) -> Option<usize> {
    let start = source.find(marker)?;
    let tail = &source[start..];
    let body_start = tail.find('{')?;
    let mut depth = 0usize;
    for (offset, ch) in tail[body_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    let body = &tail[..body_start + offset + 1];
                    return Some(body.lines().count());
                }
            }
            _ => {}
        }
    }
    None
}
