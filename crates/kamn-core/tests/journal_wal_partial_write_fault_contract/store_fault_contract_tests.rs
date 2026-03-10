use crate::support::{parse_fixture, run_case, TempDir};

#[test]
fn integration_partial_write_fault_matrix_matches_store_contracts() {
    let (_, cases) = parse_fixture().expect("fixture should parse");
    let temp_dir = TempDir::new("journal-wal-partial-write");
    for case in cases {
        run_case(&case, &temp_dir);
    }
}
