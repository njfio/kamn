const DOC: &str = include_str!("../../../../docs/planning/kolme_runtime_commit_extraction_plan.md");

#[test]
fn doc_contains_scope_boundaries_and_target_modules() {
    assert!(DOC.contains("# Kolme Runtime Commit Extraction Plan"));
    assert!(DOC.contains("## Scope Boundary"));
    assert!(DOC.contains("## Target Module Boundaries"));
    assert!(DOC.contains("`kamn-kolme`"));
}

#[test]
fn regression_requires_phase_gates_and_validation_matrix_markers() {
    assert_phase_gate_markers();
    assert_progress_markers();
    assert_issue_sequence_markers();
    assert!(DOC.contains("## Validation Matrix"));
    assert!(DOC.contains("Regression: #1814"));
}

fn assert_phase_gate_markers() {
    assert!(DOC.contains("## Phase 1 - Transport and endpoint parsing extraction"));
    assert!(DOC.contains("## Phase 2 - Finality and block-fallback extraction"));
    assert!(DOC.contains("## Phase 3 - Adapter and lifecycle orchestration extraction"));
}

fn assert_progress_markers() {
    assert!(DOC.contains("## Phase 1 Progress"));
    assert!(DOC.contains("## Phase 2 Progress"));
}

fn assert_issue_sequence_markers() {
    for marker in [
        "#1820", "#1826", "#1836", "#1838", "#1840", "#1842", "#1844", "#1846", "#1848", "#1850",
        "#1852", "#1854", "#1856", "#1858", "#1860", "#1862", "#1864", "#1866", "#1868", "#1870",
        "#1872", "#1874", "#1876", "#1878", "#1880", "#1882", "#1884", "#1886", "#1888", "#1890",
        "#1892", "#1894", "#1896", "#1898", "#1900", "#1902", "#1904", "#1906", "#1908", "#1910",
        "#1912", "#1914", "#1916", "#1918", "#1920", "#1922", "#1924", "#1926", "#1928", "#1930",
        "#1932", "#1934", "#1936", "#1938", "#1940", "#1942", "#1944", "#1946", "#1948", "#1950",
        "#1952", "#1954", "#1956", "#1958", "#1960", "#1962", "#1964",
    ] {
        assert!(DOC.contains(marker));
    }
}
