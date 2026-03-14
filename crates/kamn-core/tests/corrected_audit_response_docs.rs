use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate dir")
        .parent()
        .expect("repo root")
        .to_path_buf()
}

fn read_text(path: &str) -> String {
    let full_path = repo_root().join(path);
    fs::read_to_string(&full_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", full_path.display()))
}

#[test]
fn corrected_audit_response_doc_exists_with_required_proof_anchors_and_conclusions() {
    let doc = read_text("docs/review/corrected-audit-response-2026-03-14.md");

    for marker in [
        "docs/validation/working-vertical-slice.md",
        "docs/validation/sdk-tcp-vertical-slice.md",
        "Accurate Claims",
        "Stale Or Incorrect Claims",
        "Unproven Claims",
        "build-health blockers from the earlier audit are fixed on current main",
        "Rust LOC under crates/: 93370",
        "direct #[test] count under crates/: 5058",
        "AGENTS size debt remains real",
    ] {
        assert!(doc.contains(marker), "missing marker: {marker}");
    }
}

#[test]
fn corrected_audit_response_doc_is_linked_from_review_readme() {
    let readme = read_text("docs/review/README.md");
    assert!(
        readme.contains("corrected-audit-response-2026-03-14.md"),
        "review README must link the corrected audit response doc"
    );
}
