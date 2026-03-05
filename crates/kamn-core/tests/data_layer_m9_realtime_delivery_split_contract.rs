use std::fs;

const CONTROLS_BACKPRESSURE_ROOT_MARKERS: [&str; 9] = [
    "#[path = \"data_layer_m9_realtime_delivery/controls_backpressure_cases.rs\"]",
    "mod controls_backpressure_cases;",
    "controls_backpressure_cases::run_spec_c10_dispatch_with_controls_maps_anti_spam_rejections_to_stable_reason_codes();",
    "controls_backpressure_cases::run_spec_c11_dispatch_with_controls_allows_member_sender_when_anti_spam_accepts();",
    "controls_backpressure_cases::run_spec_c12_runtime_backpressure_projection_maps_accept_slow_reject_and_purge_actions();",
    "controls_backpressure_cases::run_spec_c13_runtime_backpressure_projection_fails_closed_for_invalid_policy_and_input();",
    "fn spec_c10_dispatch_with_controls_maps_anti_spam_rejections_to_stable_reason_codes()",
    "fn spec_c11_dispatch_with_controls_allows_member_sender_when_anti_spam_accepts()",
    "fn spec_c12_runtime_backpressure_projection_maps_accept_slow_reject_and_purge_actions()",
];

const CONTROLS_BACKPRESSURE_CASES_MARKERS: [&str; 4] = [
    "pub(super) fn run_spec_c10_dispatch_with_controls_maps_anti_spam_rejections_to_stable_reason_codes(",
    "pub(super) fn run_spec_c11_dispatch_with_controls_allows_member_sender_when_anti_spam_accepts(",
    "pub(super) fn run_spec_c12_runtime_backpressure_projection_maps_accept_slow_reject_and_purge_actions(",
    "pub(super) fn run_spec_c13_runtime_backpressure_projection_fails_closed_for_invalid_policy_and_input(",
];

fn read_repo_file(path: &str) -> String {
    let root = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{root}/{path}");
    fs::read_to_string(&full_path).unwrap_or_else(|error| {
        panic!("failed to read {path}: {error}");
    })
}

#[test]
fn spec_c01_controls_backpressure_tests_delegate_to_cases_module() {
    let root = read_repo_file("tests/data_layer_m9_realtime_delivery.rs");
    let cases =
        read_repo_file("tests/data_layer_m9_realtime_delivery/controls_backpressure_cases.rs");

    for marker in CONTROLS_BACKPRESSURE_ROOT_MARKERS {
        assert!(
            root.contains(marker),
            "root realtime-delivery contract should contain controls/backpressure delegation marker: {marker}"
        );
    }

    for marker in CONTROLS_BACKPRESSURE_CASES_MARKERS {
        assert!(
            cases.contains(marker),
            "controls/backpressure cases module should define marker: {marker}"
        );
    }
}
