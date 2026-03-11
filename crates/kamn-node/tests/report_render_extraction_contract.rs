use std::fs;
use std::path::Path;

const ROOT: &str = "src/report_render.rs";
const TEXT_RENDER: &str = "src/report_render/text_render.rs";
const JSON_RENDER: &str = "src/report_render/json_render.rs";
const FUNCTION_LINE_MAX: usize = 25;
const ROOT_MARKERS: &[&str] = &["mod json_render;", "mod text_render;"];
const TARGETS: &[(&str, &str)] = &[
    (TEXT_RENDER, "pub(super) fn render_text_report("),
    (JSON_RENDER, "pub(super) fn render_json_report("),
];

#[test]
fn report_render_root_declares_extracted_modules() {
    let source = fs::read_to_string(repo_path(ROOT)).expect("report_render.rs should be readable");
    for marker in ROOT_MARKERS {
        assert!(source.contains(marker), "missing root marker: {marker}");
    }
}

#[test]
fn report_render_functions_stay_within_active_budget() {
    for (path, marker) in TARGETS {
        let source = fs::read_to_string(repo_path(path)).expect("render module should be readable");
        let line_count = function_line_count(&source, marker);
        assert!(
            line_count <= FUNCTION_LINE_MAX,
            "{marker} in {path} should stay within {FUNCTION_LINE_MAX} lines, found {line_count}"
        );
    }
}

fn function_line_count(source: &str, marker: &str) -> usize {
    let start = source
        .lines()
        .position(|line| line.starts_with(marker))
        .unwrap_or_else(|| panic!("missing function marker: {marker}"));
    let remaining = &source.lines().collect::<Vec<_>>()[start + 1..];
    let end_offset = remaining
        .iter()
        .position(|line| line.starts_with("fn ") || line.starts_with("pub(crate) fn "))
        .unwrap_or(remaining.len());
    end_offset + 1
}

fn repo_path(path: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}
