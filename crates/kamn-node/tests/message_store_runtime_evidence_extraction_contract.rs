use std::{fs, path::PathBuf};

const ROOT_MAX_LINES: usize = 35;
const ROOT_MARKERS: &[&str] = &[
    "mod context;",
    "mod m0_to_m1;",
    "mod m2_m3;",
    "mod m4_m5;",
    "mod m6_m7;",
    "mod m8_m11;",
    "mod support;",
];
const CONTEXT_MARKERS: &[&str] = &[
    "pub(super) fn build_runtime_evidence_context",
    "pub(super) fn build_runtime_evidence_identities",
];
const SUPPORT_MARKERS: &[&str] = &[
    "pub(super) fn assemble_runtime_evidence_record",
    "pub(super) fn m2_authorization_reason_code",
    "pub(super) fn data_layer_m9_ack_status_label",
    "pub(super) fn m11_decision_label",
];

#[test]
fn message_store_runtime_evidence_function_stays_within_active_budget() {
    let source = read_runtime_evidence_root();
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
fn message_store_runtime_evidence_declares_extracted_root_markers() {
    let source = read_runtime_evidence_root();
    for marker in ROOT_MARKERS {
        assert!(
            source.contains(marker),
            "runtime_evidence.rs should declare extracted runtime evidence marker: {marker}"
        );
    }
}

#[test]
fn message_store_runtime_evidence_declares_context_and_support_helpers() {
    let context = read_runtime_evidence_context();
    for marker in CONTEXT_MARKERS {
        assert!(
            context.contains(marker),
            "context.rs should declare extracted runtime evidence helper marker: {marker}"
        );
    }
    let support = read_runtime_evidence_support();
    for marker in SUPPORT_MARKERS {
        assert!(
            support.contains(marker),
            "support.rs should declare extracted runtime evidence helper marker: {marker}"
        );
    }
}

fn read_runtime_evidence_root() -> String {
    fs::read_to_string(runtime_evidence_root_path()).expect("runtime evidence root should read")
}

fn read_runtime_evidence_context() -> String {
    fs::read_to_string(runtime_evidence_context_path())
        .expect("runtime evidence context should read")
}

fn read_runtime_evidence_support() -> String {
    fs::read_to_string(runtime_evidence_support_path())
        .expect("runtime evidence support should read")
}

fn runtime_evidence_root_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("service_api_endpoint")
        .join("message_store")
        .join("runtime_evidence.rs")
}

fn runtime_evidence_context_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("service_api_endpoint")
        .join("message_store")
        .join("runtime_evidence")
        .join("context.rs")
}

fn runtime_evidence_support_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("service_api_endpoint")
        .join("message_store")
        .join("runtime_evidence")
        .join("support.rs")
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
