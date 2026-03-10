use crate::support::models::CurrentSurface;
use crate::support::paths::{fail, repo_path};
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn current_surface() -> CurrentSurface {
    let shell_test_file_count = count_shell_tests();
    let crate_files = crate_test_files();
    let docs_rust_test_file_count = count_docs_rust_tests(&crate_files);
    let rust_test_file_count = crate_files.len() as i64 - docs_rust_test_file_count;
    if rust_test_file_count <= 0 {
        fail(
            "threshold_value_invalid",
            "rust test file count must be > 0 when computing shell/rust ratio",
        );
    }
    CurrentSurface {
        shell_test_file_count,
        rust_test_file_count,
        docs_rust_test_file_count,
        shell_to_rust_ratio: shell_test_file_count as f64 / rust_test_file_count as f64,
    }
}

pub(crate) fn is_docs_governance_rust_test_file(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
        return false;
    };
    name.ends_with("_docs.rs")
        || name.contains("docs_contract")
        || name.contains("doc_contract")
        || name.contains("missing_docs_policy")
}

fn count_shell_tests() -> i64 {
    let mut shell_files = Vec::new();
    walk_files(&repo_path("scripts"), &mut shell_files);
    shell_files
        .iter()
        .filter(|path| is_shell_test_file(path))
        .count() as i64
}

fn crate_test_files() -> Vec<PathBuf> {
    let mut crate_files = Vec::new();
    walk_files(&repo_path("crates"), &mut crate_files);
    crate_files
        .into_iter()
        .filter(|path| {
            path.extension().is_some_and(|extension| extension == "rs")
                && path.iter().any(|component| component.to_string_lossy() == "tests")
        })
        .collect()
}

fn count_docs_rust_tests(crate_files: &[PathBuf]) -> i64 {
    crate_files
        .iter()
        .filter(|path| is_docs_governance_rust_test_file(path))
        .count() as i64
}

fn is_shell_test_file(path: &Path) -> bool {
    path.extension().is_some_and(|extension| extension == "sh")
        && path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("test_"))
}

fn walk_files(root: &Path, output: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(root).unwrap_or_else(|error| {
        fail(
            "threshold_value_invalid",
            &format!("failed to read directory {}: {}", root.display(), error),
        )
    });
    for entry in entries {
        let entry = entry.unwrap_or_else(|error| {
            fail(
                "threshold_value_invalid",
                &format!("failed to read dir entry in {}: {}", root.display(), error),
            )
        });
        let path = entry.path();
        if path.is_dir() {
            walk_files(&path, output);
        } else if path.is_file() {
            output.push(path);
        }
    }
}
