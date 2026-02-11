const DOC: &str = include_str!("../../../docs/planning/kolme_runtime_commit_extraction_plan.md");

#[test]
fn doc_contains_scope_boundaries_and_target_modules() {
    assert!(DOC.contains("# Kolme Runtime Commit Extraction Plan"));
    assert!(DOC.contains("## Scope Boundary"));
    assert!(DOC.contains("## Target Module Boundaries"));
    assert!(DOC.contains("`kamn-kolme`"));
}

#[test]
fn regression_requires_phase_gates_and_validation_matrix_markers() {
    assert!(DOC.contains("## Phase 1 - Transport and endpoint parsing extraction"));
    assert!(DOC.contains("## Phase 2 - Finality and block-fallback extraction"));
    assert!(DOC.contains("## Phase 3 - Adapter and lifecycle orchestration extraction"));
    assert!(DOC.contains("## Phase 1 Progress"));
    assert!(DOC.contains("## Phase 2 Progress"));
    assert!(DOC.contains("#1820"));
    assert!(DOC.contains("#1826"));
    assert!(DOC.contains("#1836"));
    assert!(DOC.contains("#1838"));
    assert!(DOC.contains("#1840"));
    assert!(DOC.contains("#1842"));
    assert!(DOC.contains("#1844"));
    assert!(DOC.contains("#1846"));
    assert!(DOC.contains("## Validation Matrix"));
    assert!(DOC.contains("Regression: #1814"));
}
