use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const README_DOC: &str = include_str!("../../../docs/developer/readme-contract-reference.md");
const INDEX_PATH: &str = "docs/developer/script-surface-index.md";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Counts {
    sh: usize,
    py: usize,
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root canonical path must resolve")
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

fn read_index_doc() -> String {
    fs::read_to_string(repo_root().join(INDEX_PATH))
        .unwrap_or_else(|error| panic!("failed reading {INDEX_PATH}: {error}"))
}

fn accumulate(counts: &mut Counts, ext: &str) {
    match ext {
        "sh" => counts.sh += 1,
        "py" => counts.py += 1,
        _ => {}
    }
}

fn visit_scripts(
    dir: &Path,
    scripts_root: &Path,
    totals: &mut Counts,
    by_category: &mut BTreeMap<String, Counts>,
) {
    for entry in fs::read_dir(dir).unwrap_or_else(|error| panic!("read_dir failed: {error}")) {
        let entry = entry.unwrap_or_else(|error| panic!("dir entry failed: {error}"));
        let path = entry.path();

        if path.is_dir() {
            visit_scripts(&path, scripts_root, totals, by_category);
            continue;
        }

        let ext = match path.extension().and_then(|ext| ext.to_str()) {
            Some("sh") => "sh",
            Some("py") => "py",
            _ => continue,
        };

        let rel = path
            .strip_prefix(scripts_root)
            .unwrap_or_else(|error| panic!("failed stripping scripts root prefix: {error}"));
        let category = rel
            .components()
            .next()
            .and_then(|part| part.as_os_str().to_str())
            .unwrap_or_else(|| panic!("missing scripts category for path: {}", path.display()));

        accumulate(totals, ext);
        let counts = by_category.entry(category.to_owned()).or_default();
        accumulate(counts, ext);
    }
}

fn collect_filesystem_inventory() -> (Counts, BTreeMap<String, Counts>) {
    let scripts_root = repo_root().join("scripts");
    let mut totals = Counts::default();
    let mut by_category: BTreeMap<String, Counts> = BTreeMap::new();

    visit_scripts(&scripts_root, &scripts_root, &mut totals, &mut by_category);

    (totals, by_category)
}

fn parse_table_count(cell: &str, field: &str) -> usize {
    cell.trim_matches('`')
        .parse::<usize>()
        .unwrap_or_else(|error| panic!("invalid {field} cell value {cell}: {error}"))
}

fn parse_index_category_rows(doc: &str) -> BTreeMap<String, Counts> {
    let mut rows = BTreeMap::new();

    for line in doc.lines().filter(|line| line.starts_with("| `scripts/")) {
        let cells: Vec<&str> = line.split('|').map(str::trim).collect();
        if cells.len() != 6 {
            panic!("unexpected inventory row shape: {line}");
        }

        let category_cell = cells[1].trim_matches('`');
        let category = category_cell
            .strip_prefix("scripts/")
            .unwrap_or_else(|| panic!("invalid category cell: {category_cell}"));
        let sh = parse_table_count(cells[2], "sh");
        let py = parse_table_count(cells[3], "py");
        let total = parse_table_count(cells[4], "total");

        assert_eq!(
            sh + py,
            total,
            "table row total mismatch for category {category}"
        );

        let previous = rows.insert(category.to_owned(), Counts { sh, py });
        assert!(previous.is_none(), "duplicate category row found: {category}");
    }

    rows
}

#[test]
fn index_declares_required_markers_and_sections() {
    let index_doc = read_index_doc();

    assert!(index_doc.contains("# Script Surface Index"));
    assert!(index_doc.contains("script_surface_inventory_schema_version="));
    assert!(index_doc.contains("script_surface_inventory_generated_on="));
    assert!(index_doc.contains("script_surface_inventory_total_sh_files="));
    assert!(index_doc.contains("script_surface_inventory_total_py_files="));
    assert!(index_doc.contains("script_surface_inventory_total_files="));
    assert!(index_doc.contains("script_surface_inventory_category_count="));
    assert!(index_doc.contains("script_surface_inventory_primary_categories_csv="));
    assert!(index_doc.contains("## Regeneration Commands"));
    assert!(index_doc.contains("find scripts -type f -name '*.sh'"));
    assert!(index_doc.contains("find scripts -type f -name '*.py'"));
}

#[test]
fn index_inventory_matches_filesystem_counts() {
    let index_doc = read_index_doc();
    let recorded_sh = parse_usize_marker(&index_doc, "script_surface_inventory_total_sh_files");
    let recorded_py = parse_usize_marker(&index_doc, "script_surface_inventory_total_py_files");
    let recorded_total = parse_usize_marker(&index_doc, "script_surface_inventory_total_files");
    let recorded_category_count =
        parse_usize_marker(&index_doc, "script_surface_inventory_category_count");

    let recorded_rows = parse_index_category_rows(&index_doc);
    let (filesystem_totals, filesystem_rows) = collect_filesystem_inventory();

    assert_eq!(recorded_sh, filesystem_totals.sh, "sh total drifted");
    assert_eq!(recorded_py, filesystem_totals.py, "py total drifted");
    assert_eq!(
        recorded_total,
        filesystem_totals.sh + filesystem_totals.py,
        "combined total marker drifted"
    );
    assert_eq!(
        recorded_category_count,
        filesystem_rows.len(),
        "category count marker drifted"
    );
    assert_eq!(recorded_rows, filesystem_rows, "category table drifted");
}

#[test]
fn readme_contract_reference_links_inventory_index() {
    assert!(README_DOC.contains("docs/developer/script-surface-index.md"));
}
