use crate::support::{parse_case_line, parse_fixture};

#[test]
fn unit_partial_write_fixture_parser_rejects_malformed_case_columns() {
    let error = parse_case_line("broken|line")
        .expect_err("malformed fixture case line should fail parsing deterministically");
    assert!(
        error.contains("expected 5 columns"),
        "unexpected malformed-line parser error: {error}"
    );
}

#[test]
fn functional_partial_write_fixture_covers_required_fault_modes() {
    let (_, cases) = parse_fixture().expect("fixture should parse");
    assert_required_fault_modes(&cases);
    assert_required_stores(&cases);
}

fn assert_required_fault_modes(cases: &[crate::support::FixtureCase]) {
    for fault_mode in [
        "partial_snapshot_file_write",
        "partial_journal_tail_write",
        "partial_snapshot_without_journal",
    ] {
        assert!(
            cases.iter().any(|case| case.fault_mode == fault_mode),
            "fixture must include fault mode '{fault_mode}'"
        );
    }
}

fn assert_required_stores(cases: &[crate::support::FixtureCase]) {
    for store in ["channel", "message_lifecycle", "task_operation"] {
        assert!(
            cases.iter().any(|case| case.store == store),
            "fixture must include store row for '{store}'"
        );
    }
}
