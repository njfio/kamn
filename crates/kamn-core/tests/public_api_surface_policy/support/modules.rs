use crate::support::paths::{fail, read_file};
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn parse_public_modules(lib_rs: &str) -> Vec<String> {
    let mut modules = lib_rs
        .lines()
        .filter_map(public_module_name)
        .collect::<Vec<_>>();
    modules.sort();
    modules
}

pub(crate) fn module_source_paths(module: &str, src_root: &Path) -> Vec<PathBuf> {
    let mut paths = root_rs_path(module, src_root)
        .into_iter()
        .collect::<Vec<_>>();
    collect_nested_paths(&mut paths, &src_root.join(module));
    paths.sort();
    paths.dedup();
    paths
}

pub(crate) fn count_public_items(path: &Path) -> usize {
    read_file(path, "module_source_missing")
        .lines()
        .filter(|line| is_public_api_item_line(line))
        .count()
}

fn public_module_name(line: &str) -> Option<String> {
    let trimmed = line.trim();
    trimmed
        .strip_prefix("pub mod ")
        .map(|rest| rest.trim_end_matches(';').to_owned())
}

fn root_rs_path(module: &str, src_root: &Path) -> Option<PathBuf> {
    let path = src_root.join(format!("{}.rs", module));
    path.is_file().then_some(path)
}

fn collect_nested_paths(paths: &mut Vec<PathBuf>, dir: &Path) {
    if !dir.is_dir() {
        return;
    }
    for entry in read_dir_entries(dir) {
        let path = entry.path();
        if path.is_dir() {
            collect_nested_paths(paths, &path);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            paths.push(path);
        }
    }
}

fn read_dir_entries(dir: &Path) -> Vec<fs::DirEntry> {
    fs::read_dir(dir)
        .unwrap_or_else(|error| {
            fail(
                "module_source_missing",
                &format!("failed to read {}: {}", dir.display(), error),
            )
        })
        .map(|entry| {
            entry.unwrap_or_else(|error| {
                fail(
                    "module_source_missing",
                    &format!("failed to read dir entry in {}: {}", dir.display(), error),
                )
            })
        })
        .collect()
}

fn is_public_api_item_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    if is_non_public_prefix(trimmed) {
        return false;
    }
    let mut tokens = trimmed.split_whitespace();
    if tokens.next() != Some("pub") {
        return false;
    }
    matches_public_tokens(tokens.next(), tokens.next())
}

fn is_non_public_prefix(trimmed: &str) -> bool {
    trimmed.starts_with("//")
        || trimmed.starts_with("pub(crate)")
        || trimmed.starts_with("pub(super)")
        || trimmed.starts_with("pub(in ")
        || trimmed.starts_with("pub(in\t")
}

fn matches_public_tokens(first: Option<&str>, second: Option<&str>) -> bool {
    match first {
        Some("fn" | "struct" | "enum" | "trait" | "type" | "const" | "static" | "mod" | "use") => {
            true
        }
        Some("async" | "unsafe") => second == Some("fn"),
        Some("extern") => true,
        _ => false,
    }
}
