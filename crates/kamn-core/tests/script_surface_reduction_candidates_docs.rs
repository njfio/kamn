use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const README_DOC: &str = include_str!("../../../docs/developer/readme-contract-reference.md");
const DOC_PATH: &str = "docs/developer/script-surface-reduction-candidates.md";
const SH_THRESHOLD_MAX_LOC: usize = 25;
const PY_THRESHOLD_MAX_LOC: usize = 40;
const TRACKED_CI_STATS: CategoryStats = CategoryStats {
    total: 214,
    short: 21,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct CategoryStats {
    total: usize,
    short: usize,
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root canonical path must resolve")
}

fn read_doc() -> String {
    fs::read_to_string(repo_root().join(DOC_PATH))
        .unwrap_or_else(|error| panic!("failed reading {DOC_PATH}: {error}"))
}

fn marker_value<'a>(doc: &'a str, key: &str) -> Option<&'a str> {
    doc.lines()
        .find_map(|line| line.strip_prefix(&format!("{key}=")))
        .map(str::trim)
}

fn parse_usize_marker(doc: &str, key: &str) -> usize {
    marker_value(doc, key)
        .unwrap_or_else(|| panic!("missing marker: {key}"))
        .parse::<usize>()
        .unwrap_or_else(|error| panic!("invalid usize marker {key}: {error}"))
}

fn script_threshold(path: &Path) -> Option<usize> {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("sh") => Some(SH_THRESHOLD_MAX_LOC),
        Some("py") => Some(PY_THRESHOLD_MAX_LOC),
        _ => None,
    }
}

fn line_count(path: &Path) -> usize {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed reading {}: {error}", path.display()))
        .lines()
        .count()
}

fn category_for(path: &Path, scripts_root: &Path) -> String {
    let rel = path
        .strip_prefix(scripts_root)
        .unwrap_or_else(|error| panic!("failed stripping scripts root prefix: {error}"));
    rel.components()
        .next()
        .and_then(|part| part.as_os_str().to_str())
        .unwrap_or_else(|| panic!("missing scripts category for path: {}", path.display()))
        .to_owned()
}

fn walk_scripts(dir: &Path, scripts_root: &Path, stats: &mut BTreeMap<String, CategoryStats>) {
    for entry in fs::read_dir(dir).unwrap_or_else(|error| panic!("read_dir failed: {error}")) {
        let entry = entry.unwrap_or_else(|error| panic!("dir entry failed: {error}"));
        let path = entry.path();
        let file_type = entry
            .file_type()
            .unwrap_or_else(|error| panic!("failed reading file type: {error}"));

        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            walk_scripts(&path, scripts_root, stats);
            continue;
        }
        if !file_type.is_file() {
            continue;
        }

        let Some(threshold) = script_threshold(&path) else {
            continue;
        };

        let category = category_for(&path, scripts_root);
        let loc = line_count(&path);
        let entry = stats.entry(category).or_default();
        entry.total += 1;
        if loc <= threshold {
            entry.short += 1;
        }
    }
}

fn collect_filesystem_stats() -> BTreeMap<String, CategoryStats> {
    let scripts_root = repo_root().join("scripts");
    let mut stats: BTreeMap<String, CategoryStats> = BTreeMap::new();
    walk_scripts(&scripts_root, &scripts_root, &mut stats);
    stats
}

fn parse_table_count(cell: &str, field: &str) -> usize {
    cell.trim_matches('`')
        .parse::<usize>()
        .unwrap_or_else(|error| panic!("invalid {field} cell value {cell}: {error}"))
}

fn expected_ratio(total: usize, short: usize) -> String {
    format!("{:.2}%", (short as f64 / total as f64) * 100.0)
}

fn parse_doc_table_stats(doc: &str) -> BTreeMap<String, CategoryStats> {
    let mut stats = BTreeMap::new();
    for line in doc.lines().filter(|line| line.starts_with("| `scripts/")) {
        let cells: Vec<&str> = line.split('|').map(str::trim).collect();
        if cells.len() != 6 {
            panic!("unexpected table row shape: {line}");
        }
        let category = cells[1]
            .trim_matches('`')
            .strip_prefix("scripts/")
            .unwrap_or_else(|| panic!("invalid category cell: {}", cells[1]))
            .to_owned();
        let total = parse_table_count(cells[2], "total");
        let short = parse_table_count(cells[3], "short");
        assert_eq!(
            cells[4].trim_matches('`'),
            expected_ratio(total, short),
            "ratio cell drifted"
        );
        let prev = stats.insert(category.clone(), CategoryStats { total, short });
        assert!(prev.is_none(), "duplicate category row found: {category}");
    }
    stats
}

#[test]
fn candidate_doc_counts_match_filesystem_inventory() {
    let doc = read_doc();
    assert!(doc.contains("# Script Surface Reduction Candidates"));
    assert!(doc.contains("script_surface_short_wrapper_schema_version="));
    assert!(doc.contains("script_surface_short_wrapper_generated_on="));
    assert!(doc.contains("script_surface_short_wrapper_shell_threshold_max_loc="));
    assert!(doc.contains("script_surface_short_wrapper_python_threshold_max_loc="));
    assert!(doc.contains("script_surface_short_wrapper_category_count="));
    assert!(doc.contains("script_surface_short_wrapper_total_candidates="));
    assert!(doc.contains("script_surface_short_wrapper_priority_categories_csv="));
    assert!(doc.contains("## Regeneration Commands"));
    assert!(README_DOC.contains("docs/developer/script-surface-reduction-candidates.md"));
    assert!(README_DOC
        .contains("cargo test -p kamn-core --test script_surface_reduction_candidates_docs"));
    assert_eq!(
        parse_usize_marker(&doc, "script_surface_short_wrapper_shell_threshold_max_loc"),
        SH_THRESHOLD_MAX_LOC
    );
    assert_eq!(
        parse_usize_marker(
            &doc,
            "script_surface_short_wrapper_python_threshold_max_loc"
        ),
        PY_THRESHOLD_MAX_LOC
    );

    let filesystem = collect_filesystem_stats();
    let filesystem_total_short: usize = filesystem.values().map(|value| value.short).sum();
    let doc_table = parse_doc_table_stats(&doc);
    let ci_row = doc_table
        .get("ci")
        .unwrap_or_else(|| panic!("missing scripts/ci row in reduction candidates doc"));

    assert_eq!(
        parse_usize_marker(&doc, "script_surface_short_wrapper_category_count"),
        filesystem.len()
    );
    assert_eq!(
        parse_usize_marker(&doc, "script_surface_short_wrapper_total_candidates"),
        filesystem_total_short
    );
    assert_eq!(
        *ci_row, TRACKED_CI_STATS,
        "tracked scripts/ci candidate row drifted"
    );
    assert_eq!(doc_table, filesystem, "candidate table drifted");

    let categories = marker_value(&doc, "script_surface_short_wrapper_priority_categories_csv")
        .unwrap_or_else(|| {
            panic!("missing marker: script_surface_short_wrapper_priority_categories_csv")
        })
        .split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty());
    for category in categories {
        let stats = filesystem
            .get(category)
            .unwrap_or_else(|| panic!("priority category missing from filesystem set: {category}"));
        assert!(
            stats.short > 0,
            "priority category must have short wrappers"
        );
    }
}
