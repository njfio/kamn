use super::super::support::{assert_checklist_contains_all, assert_checklist_lacks_all};

const REGRESSION_REQUIRES_CANONICAL_DID_EXAMPLES_MARKERS: &[&str] = &[
    "--collector-did kamn:did:auditor-001",
    "--subject-did kamn:did:subject-001",
    "--subject-did kamn:did:agent-001",
    "--reviewer-did kamn:did:reviewer-001",
];

const REGRESSION_REQUIRES_CANONICAL_DID_EXAMPLES_MARKERS_FORBIDDEN: &[&str] = &[
    "did:kamn:",
];

#[test]
fn regression_requires_canonical_did_examples() {
    assert_checklist_contains_all(REGRESSION_REQUIRES_CANONICAL_DID_EXAMPLES_MARKERS, "regression_requires_canonical_did_examples");
    assert_checklist_lacks_all(REGRESSION_REQUIRES_CANONICAL_DID_EXAMPLES_MARKERS_FORBIDDEN, "regression_requires_canonical_did_examples");
}
