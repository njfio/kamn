use std::fs;
use std::path::PathBuf;

const TARGET_HELPERS: &[&str] = &[
    "env_var_or_default",
    "env_var_or_else",
    "is_live_bound_scenario_id",
    "parse_bool_flag",
    "parse_s15_budget_env_u128",
    "validate_s15_latency_budget_samples",
    "percentile_index",
    "validate_s07_replay_reason_marker",
    "live_s07_probe_agent_suffix",
    "validate_s12_content_id_match",
    "validate_s12_content_field_coherence",
    "validate_s13_bridge_id_match",
    "validate_s13_bridge_field_coherence",
];

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_driver_source(file_name: &str) -> String {
    fs::read_to_string(crate_root().join("src").join("drivers").join(file_name))
        .expect("driver source must be readable")
}

fn count_function_definitions(source: &str, fn_name: &str) -> usize {
    source.match_indices(&format!("fn {fn_name}")).count()
}

#[test]
fn spec_c01_i11_helpers_are_single_sourced_in_shared_helpers_module() {
    let shared_source = read_driver_source("shared_helpers.rs");

    for helper in TARGET_HELPERS {
        let shared_count = count_function_definitions(shared_source.as_str(), helper);
        assert_eq!(
            shared_count, 1,
            "shared helper module must define helper exactly once: {helper}"
        );
    }
}

#[test]
fn spec_c02_i11_driver_modules_do_not_redeclare_shared_helpers() {
    let drivers = ["sdk_direct.rs", "cli_scripted.rs", "mcp_agent.rs"];

    for driver in drivers {
        let source = read_driver_source(driver);
        for helper in TARGET_HELPERS {
            let count = count_function_definitions(source.as_str(), helper);
            assert_eq!(
                count, 0,
                "driver {driver} redeclares shared helper {helper}"
            );
        }
    }
}
