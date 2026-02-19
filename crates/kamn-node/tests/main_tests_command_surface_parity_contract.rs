use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

const REASON_TAXONOMY_VERSION: &str =
    "kamn.node.main-tests-command-surface-parity-reason-taxonomy.v1";
const REASON_CODES_CSV: &str =
    "runtime_test_selector_symbol_missing,runtime_test_selector_command_missing,command_surface_parity_marker_missing";

const REQUIRED_SELECTORS: &[&str] = &[
    "main_tests::runtime_tests::functional_runtime_kolme_live_retries_transient_submit_and_finality_unavailable_errors",
    "main_tests::runtime_tests::regression_runtime_kolme_live_submit_malformed_response_fails_fast_without_retry",
    "main_tests::runtime_tests::performance_runtime_kolme_live_retry_recovery_stays_within_budget",
    "main_tests::runtime_tests::integration_runtime_full_emits_ordered_bootstrap_readiness_markers",
];

fn repo_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn read_repo_file(relative: &str) -> String {
    let path = repo_path(relative);
    fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!("failed to read {}: {}", path.display(), error);
    })
}

fn runtime_test_sources() -> Vec<(String, String)> {
    let mut sources = vec![(
        "src/main_tests/runtime_tests.rs".to_owned(),
        read_repo_file("src/main_tests/runtime_tests.rs"),
    )];
    let mut fragment_paths = Vec::new();
    for entry in fs::read_dir(repo_path("src/main_tests/runtime_tests")).unwrap_or_else(|error| {
        panic!("failed to read runtime_tests fragments: {}", error);
    }) {
        let entry = entry.unwrap_or_else(|error| {
            panic!("failed to read runtime_tests fragment entry: {}", error);
        });
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "rs") {
            fragment_paths.push(path);
        }
    }
    fragment_paths.sort();
    for path in fragment_paths {
        let relative = path
            .strip_prefix(repo_path(""))
            .unwrap_or_else(|error| panic!("failed to relativize fragment path: {}", error))
            .to_string_lossy()
            .to_string();
        let source = fs::read_to_string(&path).unwrap_or_else(|error| {
            panic!("failed to read {}: {}", path.display(), error);
        });
        sources.push((relative, source));
    }
    sources
}

fn selector_fn_name(selector: &str) -> &str {
    selector
        .rsplit("::")
        .next()
        .expect("selector should contain function name")
}

fn source_contains_fn_symbol(sources: &[(String, String)], fn_name: &str) -> bool {
    let needle = format!("fn {fn_name}(");
    sources.iter().any(|(_, source)| source.contains(&needle))
}

#[test]
fn docs_define_main_tests_command_surface_parity_markers() {
    let doc = read_repo_file("../../docs/ci/strategy.md");
    assert!(
        doc.contains("## Main Tests Command-Surface Parity Contract"),
        "reason_taxonomy_version={REASON_TAXONOMY_VERSION} reason_codes_csv={REASON_CODES_CSV} reason_code=command_surface_parity_marker_missing"
    );
    assert!(
        doc.contains(format!(
            "main_tests_command_surface_parity_reason_taxonomy_version={REASON_TAXONOMY_VERSION}"
        )
        .as_str()),
        "reason_taxonomy_version={REASON_TAXONOMY_VERSION} reason_codes_csv={REASON_CODES_CSV} reason_code=command_surface_parity_marker_missing"
    );
    assert!(
        doc.contains(format!("main_tests_command_surface_parity_reason_codes_csv={REASON_CODES_CSV}").as_str()),
        "reason_taxonomy_version={REASON_TAXONOMY_VERSION} reason_codes_csv={REASON_CODES_CSV} reason_code=command_surface_parity_marker_missing"
    );
    assert!(
        doc.contains("main_tests_command_surface_parity_status=verified"),
        "reason_taxonomy_version={REASON_TAXONOMY_VERSION} reason_codes_csv={REASON_CODES_CSV} reason_code=command_surface_parity_marker_missing"
    );
    assert!(
        doc.contains(
            "cargo test -p kamn-node --test main_tests_command_surface_parity_contract -- --nocapture"
        ),
        "reason_taxonomy_version={REASON_TAXONOMY_VERSION} reason_codes_csv={REASON_CODES_CSV} reason_code=command_surface_parity_marker_missing"
    );
}

#[test]
fn selector_symbols_and_commands_remain_in_runtime_test_command_surface() {
    let sources = runtime_test_sources();
    let doc = read_repo_file("../../docs/ci/strategy.md");

    let mut missing_symbols = Vec::new();
    let mut missing_commands = Vec::new();
    let mut reason_codes = BTreeSet::new();

    for selector in REQUIRED_SELECTORS {
        let fn_name = selector_fn_name(selector);
        if !source_contains_fn_symbol(&sources, fn_name) {
            missing_symbols.push(selector.to_string());
            reason_codes.insert("runtime_test_selector_symbol_missing");
        }
        let expected_command = format!("cargo test -p kamn-node {selector} -- --exact");
        if !doc.contains(expected_command.as_str()) {
            missing_commands.push(expected_command);
            reason_codes.insert("runtime_test_selector_command_missing");
        }
    }

    if !missing_symbols.is_empty() || !missing_commands.is_empty() {
        let reason_codes_value = if reason_codes.is_empty() {
            "none".to_owned()
        } else {
            reason_codes.into_iter().collect::<Vec<_>>().join(",")
        };
        panic!(
            "reason_taxonomy_version={} reason_codes_csv={} reason_codes={} missing_symbols={:?} missing_commands={:?}",
            REASON_TAXONOMY_VERSION,
            REASON_CODES_CSV,
            reason_codes_value,
            missing_symbols,
            missing_commands
        );
    }
}
