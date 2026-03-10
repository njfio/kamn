use crate::support::{parse_fixture, run_case, TempDir};
use std::time::Instant;

#[test]
fn performance_partial_write_fault_matrix_stays_within_ci_budget() {
    let (_, cases) = parse_fixture().expect("fixture should parse");
    let temp_dir = TempDir::new("journal-wal-partial-write-perf");
    let started = Instant::now();
    for case in cases {
        run_case(&case, &temp_dir);
    }
    let elapsed_millis = started.elapsed().as_millis();
    assert!(
        elapsed_millis < 350,
        "partial-write fault matrix exceeded CI budget: {elapsed_millis}ms"
    );
}
