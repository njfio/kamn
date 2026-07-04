use std::fs;
use std::path::PathBuf;
use std::process::Command;

use super::baseline_threshold_support::{Baseline, CurrentSurface, Evaluation, Thresholds};
use super::source_path_support::{fail, is_test_only_source_path, repo_path, repo_root};
use super::token_scan_support::count_expect_occurrences_excluding_cfg_test;

pub fn current_surface() -> CurrentSurface {
    let files = tracked_source_files();
    let production_rs_file_count = files.len() as i64;
    let production_expect_count = count_expect_surface(files.as_slice());
    if production_rs_file_count < 0 || production_expect_count < 0 {
        fail(
            "census_value_invalid",
            "production source census produced negative counts",
        );
    }
    CurrentSurface {
        production_rs_file_count,
        production_expect_count,
    }
}

pub fn evaluate_policy(
    baseline: &Baseline,
    thresholds: &Thresholds,
    current: &CurrentSurface,
) -> Evaluation {
    let expect_delta = current.production_expect_count - baseline.production_expect_count;
    if expect_delta <= thresholds.allowed_expect_delta_max {
        return Evaluation {
            final_decision: "GO",
            reason_codes: vec!["none"],
        };
    }
    Evaluation {
        final_decision: "NO-GO",
        reason_codes: vec![
            "expect_delta_exceeded",
            "expect_threshold_exceeded_unwaived",
        ],
    }
}

fn tracked_source_files() -> Vec<PathBuf> {
    let stdout = tracked_source_stdout();
    let mut files = stdout
        .lines()
        .filter_map(to_tracked_source_path)
        .map(repo_path)
        .collect::<Vec<_>>();
    files.sort();
    files
}

fn tracked_source_stdout() -> String {
    let output = Command::new("git")
        .args(["ls-tree", "-r", "--name-only", "HEAD", "crates"])
        .current_dir(repo_root())
        .output()
        .unwrap_or_else(|error| fail("census_command_failed", &format!("git ls-tree: {error}")));
    if !output.status.success() {
        let status = output.status;
        fail(
            "census_command_failed",
            &format!("git ls-tree exited with status {status}"),
        );
    }
    String::from_utf8(output.stdout).unwrap_or_else(|error| {
        fail(
            "census_command_failed",
            &format!("git ls-tree output is not utf8: {error}"),
        )
    })
}

fn to_tracked_source_path(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    if trimmed.is_empty() || !trimmed.ends_with(".rs") {
        return None;
    }
    if !trimmed.contains("/src/") || trimmed.contains("/tests/") {
        return None;
    }
    if trimmed.ends_with("_tests.rs") || is_test_only_source_path(trimmed) {
        return None;
    }
    Some(trimmed)
}

fn count_expect_surface(files: &[PathBuf]) -> i64 {
    files.iter().map(count_expects_in_file).sum()
}

fn count_expects_in_file(file: &PathBuf) -> i64 {
    let raw = fs::read_to_string(file).unwrap_or_else(|error| {
        let display_path = file.display();
        fail(
            "census_value_invalid",
            &format!("failed to read source file {display_path}: {error}"),
        )
    });
    count_expect_occurrences_excluding_cfg_test(&raw)
}
