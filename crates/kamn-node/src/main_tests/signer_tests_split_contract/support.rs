use std::fs;

pub(super) fn read_repo_file(path: &str) -> String {
    let root = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{root}/{path}");
    fs::read_to_string(&full_path).unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

pub(super) fn line_count(path: &str) -> usize {
    read_repo_file(path).lines().count()
}
