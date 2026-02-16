use std::fs;
use std::time::{Duration, Instant};

fn read_repo_file(path: &str) -> String {
    let root = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{root}/{path}");
    fs::read_to_string(&full_path).unwrap_or_else(|error| {
        panic!("failed to read {path}: {error}");
    })
}

fn ad_hoc_output_macros(source: &str) -> Vec<&str> {
    source
        .lines()
        .map(str::trim)
        .filter(|line| line.contains("println!(") || line.contains("eprintln!("))
        .collect()
}

fn assert_no_ad_hoc_output_macros(path: &str) {
    let source = read_repo_file(path);
    let ad_hoc = ad_hoc_output_macros(source.as_str());
    assert!(
        ad_hoc.is_empty(),
        "{path} must not use ad-hoc println/eprintln macros in critical paths; found: {ad_hoc:?}"
    );
}

#[test]
fn unit_runtime_output_contract_scanner_detects_ad_hoc_print_macros() {
    let sample = r#"
        println!("demo");
        eprintln!("demo");
    "#;
    let detected = ad_hoc_output_macros(sample);
    assert_eq!(detected.len(), 2);
}

#[test]
fn functional_runtime_output_contract_enforces_critical_runtime_modules() {
    let critical_paths = [
        "src/runtime_kolme_live.rs",
        "src/runtime_orchestration.rs",
        "src/runtime_orchestration/daemon_phase.rs",
        "src/signer.rs",
    ];
    for path in critical_paths {
        assert_no_ad_hoc_output_macros(path);
    }
}

#[test]
fn integration_runtime_output_contract_enforces_main_entrypoint_path() {
    // Regression: #4122
    assert_no_ad_hoc_output_macros("src/main.rs");
}

#[test]
fn regression_runtime_output_contract_main_failure_path_keeps_structured_error_event() {
    // Regression: #4123
    let main_rs = read_repo_file("src/main.rs");
    assert!(
        main_rs.contains("log_error("),
        "main failure path must keep structured error logging"
    );
    assert!(
        main_rs.contains("\"node.runtime.execute.failed\""),
        "main failure path must keep deterministic execute-failed event marker"
    );
    assert!(
        main_rs.contains("write_stderr_line(error_message.as_str())"),
        "main failure path must route stderr writes through deterministic helper"
    );
}

#[test]
fn performance_runtime_output_contract_scanner_stays_bounded() {
    let paths = [
        "src/main.rs",
        "src/runtime_kolme_live.rs",
        "src/runtime_orchestration.rs",
        "src/runtime_orchestration/daemon_phase.rs",
        "src/signer.rs",
    ];
    let started = Instant::now();
    for _ in 0..100 {
        for path in paths {
            let source = read_repo_file(path);
            let _ = ad_hoc_output_macros(source.as_str());
        }
    }
    assert!(
        started.elapsed() < Duration::from_secs(3),
        "output contract scan exceeded 3s budget for bounded source corpus"
    );
}
