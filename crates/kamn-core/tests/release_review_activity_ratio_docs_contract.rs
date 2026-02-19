use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const REVIEW_README: &str = include_str!("../../../docs/review/README.md");

const REQUIRED_MARKERS: [&str; 6] = [
    "governance_feature_activity_ratio_schema_version",
    "governance_activity_commit_count",
    "feature_activity_commit_count",
    "activity_total_commit_count",
    "governance_activity_commit_ratio",
    "feature_activity_commit_ratio",
];

const REQUIRED_SCHEMA_VERSION: &str = "kamn.review.governance-feature-activity-ratio.v1";

fn review_docs_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("docs")
        .join("review")
}

fn release_number_from_review_filename(path: &Path) -> Option<u32> {
    let file_name = path.file_name()?.to_str()?;
    if !file_name.starts_with("gaps-and-issues-r") || !file_name.ends_with(".md") {
        return None;
    }
    let suffix = file_name
        .trim_start_matches("gaps-and-issues-r")
        .trim_end_matches(".md");
    suffix.parse::<u32>().ok()
}

fn r43_plus_review_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    let docs_dir = review_docs_dir();
    for entry in fs::read_dir(docs_dir).expect("docs/review directory should be readable") {
        let entry = entry.expect("docs/review entry should be readable");
        let path = entry.path();
        let Some(release) = release_number_from_review_filename(path.as_path()) else {
            continue;
        };
        if release >= 43 {
            files.push(path);
        }
    }
    files.sort();
    files
}

fn parse_marker_lines(doc: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for line in doc.lines() {
        let trimmed = line.trim();
        let candidate = trimmed.strip_prefix("- ").unwrap_or(trimmed);
        let Some((key, value)) = candidate.split_once('=') else {
            continue;
        };
        let key = key.trim();
        if key.is_empty() {
            continue;
        }
        map.insert(key.to_owned(), value.trim().to_owned());
    }
    map
}

#[test]
fn functional_release_review_activity_ratio_markers_present_for_r43_and_later() {
    let files = r43_plus_review_files();
    assert!(
        !files.is_empty(),
        "expected at least one R43+ gaps-and-issues review file"
    );

    for file in files {
        let doc = fs::read_to_string(file.as_path())
            .unwrap_or_else(|error| panic!("failed reading {}: {error}", file.display()));
        let markers = parse_marker_lines(doc.as_str());

        for key in REQUIRED_MARKERS {
            assert!(
                markers.contains_key(key),
                "{} missing required activity-ratio marker `{key}`",
                file.display()
            );
        }
        assert_eq!(
            markers
                .get("governance_feature_activity_ratio_schema_version")
                .map(String::as_str),
            Some(REQUIRED_SCHEMA_VERSION),
            "{} must declare required activity-ratio schema version",
            file.display()
        );
    }
}

#[test]
fn integration_release_review_activity_ratio_markers_are_numeric_and_consistent() {
    assert!(REVIEW_README.contains("governance_feature_activity_ratio_schema_version"));
    assert!(REVIEW_README.contains("kamn.review.governance-feature-activity-ratio.v1"));

    for file in r43_plus_review_files() {
        let doc = fs::read_to_string(file.as_path())
            .unwrap_or_else(|error| panic!("failed reading {}: {error}", file.display()));
        let markers = parse_marker_lines(doc.as_str());

        let governance_count: u64 = markers
            .get("governance_activity_commit_count")
            .unwrap_or_else(|| {
                panic!(
                    "{} missing governance_activity_commit_count",
                    file.display()
                )
            })
            .parse()
            .unwrap_or_else(|error| {
                panic!(
                    "{} governance_activity_commit_count must be an integer: {error}",
                    file.display()
                )
            });
        let feature_count: u64 = markers
            .get("feature_activity_commit_count")
            .unwrap_or_else(|| panic!("{} missing feature_activity_commit_count", file.display()))
            .parse()
            .unwrap_or_else(|error| {
                panic!(
                    "{} feature_activity_commit_count must be an integer: {error}",
                    file.display()
                )
            });
        let total_count: u64 = markers
            .get("activity_total_commit_count")
            .unwrap_or_else(|| panic!("{} missing activity_total_commit_count", file.display()))
            .parse()
            .unwrap_or_else(|error| {
                panic!(
                    "{} activity_total_commit_count must be an integer: {error}",
                    file.display()
                )
            });
        assert_eq!(
            governance_count + feature_count,
            total_count,
            "{} must keep governance+feature commit counts equal to total",
            file.display()
        );
        assert!(
            total_count > 0,
            "{} activity_total_commit_count must be positive",
            file.display()
        );

        let governance_ratio: f64 = markers
            .get("governance_activity_commit_ratio")
            .unwrap_or_else(|| {
                panic!(
                    "{} missing governance_activity_commit_ratio",
                    file.display()
                )
            })
            .parse()
            .unwrap_or_else(|error| {
                panic!(
                    "{} governance_activity_commit_ratio must be numeric: {error}",
                    file.display()
                )
            });
        let feature_ratio: f64 = markers
            .get("feature_activity_commit_ratio")
            .unwrap_or_else(|| panic!("{} missing feature_activity_commit_ratio", file.display()))
            .parse()
            .unwrap_or_else(|error| {
                panic!(
                    "{} feature_activity_commit_ratio must be numeric: {error}",
                    file.display()
                )
            });

        let expected_governance = governance_count as f64 / total_count as f64;
        let expected_feature = feature_count as f64 / total_count as f64;

        assert!(
            (governance_ratio - expected_governance).abs() <= 0.001,
            "{} governance ratio marker must match commit-count ratio",
            file.display()
        );
        assert!(
            (feature_ratio - expected_feature).abs() <= 0.001,
            "{} feature ratio marker must match commit-count ratio",
            file.display()
        );
        assert!(
            (governance_ratio + feature_ratio - 1.0).abs() <= 0.001,
            "{} governance+feature ratio markers must sum to 1.0",
            file.display()
        );
    }
}
