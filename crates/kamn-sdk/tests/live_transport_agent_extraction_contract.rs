use std::fs;
use std::path::Path;

const ROOT: &str = "tests/live_transport_agent.rs";
const ROOT_BUDGET: usize = 180;
const REQUIRED_MODULE_MARKERS: &[&str] = &[
    "#[path = \"live_transport_agent/support.rs\"]",
    "#[path = \"live_transport_agent/transport_mode_contract_tests.rs\"]",
    "#[path = \"live_transport_agent/message_contract_tests.rs\"]",
    "#[path = \"live_transport_agent/resolve_profile_contract_tests.rs\"]",
    "#[path = \"live_transport_agent/registration_search_contract_tests.rs\"]",
    "#[path = \"live_transport_agent/channel_contract_tests.rs\"]",
];
const REQUIRED_FILES: &[&str] = &[
    "tests/live_transport_agent/support.rs",
    "tests/live_transport_agent/transport_mode_contract_tests.rs",
    "tests/live_transport_agent/message_contract_tests.rs",
    "tests/live_transport_agent/resolve_profile_contract_tests.rs",
    "tests/live_transport_agent/registration_search_contract_tests.rs",
    "tests/live_transport_agent/channel_contract_tests.rs",
];

#[test]
fn live_transport_agent_root_is_extracted() {
    let source = fs::read_to_string(ROOT).expect("root test file should be readable");
    assert_root_budget(source.as_str());
    assert_root_markers(source.as_str());
    assert_extracted_files();
}

fn assert_root_budget(source: &str) {
    let line_count = source.lines().count();
    assert!(
        line_count <= ROOT_BUDGET,
        "expected {ROOT} to be <= {ROOT_BUDGET} lines after extraction, got {line_count}"
    );
}

fn assert_root_markers(source: &str) {
    for marker in REQUIRED_MODULE_MARKERS {
        assert!(
            source.contains(marker),
            "expected root shell to contain module marker `{marker}`"
        );
    }
}

fn assert_extracted_files() {
    for path in REQUIRED_FILES {
        assert_file_exists(path);
        assert_file_budget(path);
    }
}

fn assert_file_exists(path: &str) {
    let file = Path::new(path);
    assert!(file.exists(), "expected extracted file `{path}` to exist");
}

fn assert_file_budget(path: &str) {
    let file = Path::new(path);
    let count = fs::read_to_string(file)
        .unwrap_or_else(|error| panic!("failed to read `{path}`: {error}"))
        .lines()
        .count();
    assert!(
        count <= 200,
        "expected extracted file `{path}` to stay within 200 lines, got {count}"
    );
}
