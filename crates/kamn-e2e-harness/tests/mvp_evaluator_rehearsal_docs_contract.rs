use std::path::PathBuf;

const EVIDENCE_DOC: &str = "docs/validation/2026-07-07-mvp-evaluator-rehearsal.md";
const REQUIRED_EVIDENCE: &[&str] = &[
    "# MVP Evaluator Rehearsal Evidence - 2026-07-07",
    "Issue: #7043",
    "Clean Worktree",
    "make demo-mvp",
    "verify-mvp-demo",
    "run-57898-1783463882414",
    "run-73263-1783464071530",
    "devnet-backed",
    "2dWRAChLFzqAFxpNPYAb6ZGkP6Ms6yrLJm6ZGYXG7XmM8rXy2Emmy8myhva6gtCNbpkusCrCHfGa14oR7PamHGss",
    "Secret Scan",
    "Goose Harness",
    "Pi Harness",
    "Claim Boundaries",
    "No private key, keypair JSON, env file, or private credential content was recorded",
];

#[test]
fn spec_c01_evaluator_rehearsal_records_required_evidence() {
    let doc = read_evidence_doc();

    require_all(&doc, REQUIRED_EVIDENCE);
}

#[test]
fn spec_c02_evaluator_rehearsal_orders_evidence_before_risks() {
    let doc = read_evidence_doc();

    require_order(&doc, "Clean Worktree", "Local-Only Demo");
    require_order(&doc, "Local-Only Demo", "Devnet-Required Demo");
    require_order(&doc, "Devnet-Required Demo", "Agent Harness Evaluation");
    require_order(&doc, "Agent Harness Evaluation", "Remaining Risks");
}

fn read_evidence_doc() -> String {
    std::fs::read_to_string(repo_root().join(EVIDENCE_DOC))
        .unwrap_or_else(|err| panic!("{EVIDENCE_DOC} should exist and be readable: {err}"))
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn require_all(content: &str, needles: &[&str]) {
    for needle in needles {
        assert!(
            content.contains(needle),
            "{EVIDENCE_DOC} is missing required evidence: {needle}"
        );
    }
}

fn require_order(content: &str, before: &str, after: &str) {
    let before_index = content
        .find(before)
        .unwrap_or_else(|| panic!("{EVIDENCE_DOC} is missing required heading: {before}"));
    let after_index = content
        .find(after)
        .unwrap_or_else(|| panic!("{EVIDENCE_DOC} is missing required heading: {after}"));

    assert!(
        before_index < after_index,
        "{EVIDENCE_DOC} should place `{before}` before `{after}`"
    );
}
