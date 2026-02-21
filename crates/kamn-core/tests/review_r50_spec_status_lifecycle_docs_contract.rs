const SPEC_5507: &str = include_str!("../../../specs/5507/spec.md");
const SPEC_5509: &str = include_str!("../../../specs/5509/spec.md");
const SPEC_5513: &str = include_str!("../../../specs/5513/spec.md");

fn status_line(doc: &str) -> &str {
    doc.lines()
        .find(|line| line.trim_start().starts_with("- Status:"))
        .unwrap_or("")
        .trim()
}

#[test]
fn functional_r50_closed_task_specs_report_implemented_status() {
    assert!(
        SPEC_5507.contains("- Status: Implemented"),
        "specs/5507/spec.md must report Implemented status"
    );
    assert!(
        SPEC_5509.contains("- Status: Implemented"),
        "specs/5509/spec.md must report Implemented status"
    );
    assert!(
        SPEC_5513.contains("- Status: Implemented"),
        "specs/5513/spec.md must report Implemented status"
    );
}

#[test]
fn integration_r50_closed_task_specs_do_not_regress_to_accepted_status() {
    assert_ne!(
        status_line(SPEC_5507),
        "- Status: Accepted",
        "specs/5507/spec.md must not regress to Accepted once merged"
    );
    assert_ne!(
        status_line(SPEC_5509),
        "- Status: Accepted",
        "specs/5509/spec.md must not regress to Accepted once merged"
    );
    assert_ne!(
        status_line(SPEC_5513),
        "- Status: Accepted",
        "specs/5513/spec.md must not regress to Accepted once merged"
    );
}
