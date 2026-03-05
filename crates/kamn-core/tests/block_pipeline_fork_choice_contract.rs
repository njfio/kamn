use std::fs;
use std::path::PathBuf;

const REQUIRED_TEST_MARKERS: [&str; 7] = [
    "fn unit_fork_choice_empty_head_accepts_and_seeds_canonical_head()",
    "fn unit_fork_choice_higher_block_height_replaces_canonical_head()",
    "fn unit_fork_choice_stale_block_height_rejects_and_preserves_head()",
    "fn unit_fork_choice_duplicate_candidate_rejects_and_preserves_head()",
    "fn unit_fork_choice_lower_digest_at_same_height_replaces_canonical_head()",
    "fn unit_fork_choice_higher_digest_at_same_height_rejects_and_preserves_head()",
    "fn unit_accept_all_fork_choice_hook_accepts_without_state()",
];

fn repo_file(path: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    fs::read_to_string(root.join(path))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

#[test]
fn spec_c01_direct_fork_choice_test_target_exists_with_required_markers() {
    let source = repo_file("crates/kamn-core/tests/block_pipeline_fork_choice.rs");
    for marker in REQUIRED_TEST_MARKERS {
        assert!(
            source.contains(marker),
            "direct fork-choice test target should contain marker: {marker}"
        );
    }
}
