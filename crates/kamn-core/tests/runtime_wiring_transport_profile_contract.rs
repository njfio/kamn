use std::fs;

fn read_repo_file(path: &str) -> String {
    let root = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{root}/{path}");
    fs::read_to_string(&full_path).unwrap_or_else(|error| {
        panic!("failed to read {path}: {error}");
    })
}

#[test]
fn runtime_wiring_transport_profile_contract_requires_dedicated_integration_surface() {
    let integration = read_repo_file("tests/runtime_wiring_transport_profile_integration.rs");
    assert!(
        integration.contains("fn integration_default_runtime_wiring_uses_in_memory_transport_markers()"),
        "integration surface must cover default in-memory runtime wiring markers"
    );
    assert!(
        integration.contains("fn integration_live_transport_profile_emits_provider_and_compile_mode_markers()"),
        "integration surface must cover live transport profile markers"
    );
    assert!(
        integration.contains("fn integration_gossip_disabled_runtime_wiring_uses_disabled_marker_only()"),
        "integration surface must cover gossip-disabled runtime wiring markers"
    );
    assert!(
        integration.contains("fn integration_role_specific_runtime_wiring_components_stay_stable()"),
        "integration surface must cover role-specific runtime wiring components"
    );
    assert!(
        integration.contains("fn integration_feature_gate_name_and_compile_mode_marker_align()"),
        "integration surface must cover feature gate and compile mode alignment"
    );
}
