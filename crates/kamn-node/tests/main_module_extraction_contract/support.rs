use std::fs;

pub(crate) fn read_repo_file(path: &str) -> String {
    let root = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{root}/{path}");
    fs::read_to_string(&full_path).unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

pub(crate) fn assert_contains_all(source: &str, checks: &[(&str, &str)]) {
    for (needle, message) in checks {
        assert!(source.contains(needle), "{message}");
    }
}

pub(crate) fn assert_not_contains_all(source: &str, checks: &[(&str, &str)]) {
    for (needle, message) in checks {
        assert!(!source.contains(needle), "{message}");
    }
}

pub(crate) fn line_count(source: &str) -> usize {
    source.lines().count()
}

pub(crate) fn count_lines_with_prefix(source: &str, prefix: &str) -> usize {
    source
        .lines()
        .filter(|line| line.trim_start().starts_with(prefix))
        .count()
}
