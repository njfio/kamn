use std::fs;
use std::path::Path;

const ROOT: &str = "tests/main_module_extraction_contract.rs";
const ROOT_BUDGET: usize = 180;
const REQUIRED_MODULE_MARKERS: &[&str] = &[
    "#[path = \"main_module_extraction_contract/support.rs\"]",
    "#[path = \"main_module_extraction_contract/main_root_contract_tests.rs\"]",
    "#[path = \"main_module_extraction_contract/runtime_model_contract_tests.rs\"]",
    "#[path = \"main_module_extraction_contract/runtime_boundary_contract_tests.rs\"]",
    "#[path = \"main_module_extraction_contract/test_shell_budget_contract_tests.rs\"]",
];
const REQUIRED_FILES: &[&str] = &[
    "tests/main_module_extraction_contract/support.rs",
    "tests/main_module_extraction_contract/main_root_contract_tests.rs",
    "tests/main_module_extraction_contract/runtime_model_contract_tests.rs",
    "tests/main_module_extraction_contract/runtime_boundary_contract_tests.rs",
    "tests/main_module_extraction_contract/test_shell_budget_contract_tests.rs",
];

#[test]
fn main_module_extraction_contract_root_is_extracted() {
    let source = fs::read_to_string(ROOT)
        .expect("root main-module extraction contract file should be readable");
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
        let file = Path::new(path);
        assert!(file.exists(), "expected extracted file `{path}` to exist");
        let line_count = fs::read_to_string(file)
            .unwrap_or_else(|error| panic!("failed to read `{path}`: {error}"))
            .lines()
            .count();
        assert!(
            line_count <= 200,
            "expected extracted file `{path}` to stay within 200 lines, got {line_count}"
        );
    }
}
