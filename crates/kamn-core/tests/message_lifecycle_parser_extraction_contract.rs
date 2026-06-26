use std::fs;

const SOURCE_FILE: &str = "src/message_lifecycle/snapshot_codec/parse.rs";
const TARGET_FUNCTION: &str = "fn parse_message_lifecycle_snapshot_payload(";
const TARGET_HELPERS: &[&str] = &[
    "fn parse_message_lifecycle_snapshot_schema(",
    "fn parse_message_lifecycle_snapshot_record(",
    "fn parse_message_lifecycle_snapshot_status_history(",
];
const MAX_COORDINATOR_LINES: usize = 25;

fn read_source_file(path: &str) -> String {
    fs::read_to_string(format!("{}/{}", env!("CARGO_MANIFEST_DIR"), path))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

fn function_line_count(source: &str, header: &str) -> usize {
    let start = source
        .find(header)
        .unwrap_or_else(|| panic!("missing function header: {header}"));
    let body = &source[start..];
    let open = body
        .find('{')
        .unwrap_or_else(|| panic!("missing opening brace for {header}"));
    let mut depth = 0_i32;
    for (offset, ch) in body[open..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return body[..open + offset + 1].lines().count();
                }
            }
            _ => {}
        }
    }
    panic!("missing closing brace for {header}");
}

#[test]
fn spec_c01_message_lifecycle_snapshot_parser_is_coordinator_sized() {
    let source = read_source_file(SOURCE_FILE);
    let line_count = function_line_count(&source, TARGET_FUNCTION);
    assert!(
        line_count <= MAX_COORDINATOR_LINES,
        "expected {TARGET_FUNCTION} to stay within {MAX_COORDINATOR_LINES} lines, found {line_count}"
    );
}

#[test]
fn spec_c02_message_lifecycle_snapshot_parser_declares_extracted_helpers() {
    let source = read_source_file(SOURCE_FILE);
    for helper in TARGET_HELPERS {
        assert!(
            source.contains(helper),
            "expected helper declaration missing: {helper}"
        );
    }
}
