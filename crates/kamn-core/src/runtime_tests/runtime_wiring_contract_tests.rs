use super::*;

#[test]
fn processor_wiring_contains_block_producer() {
    let wiring = build_runtime_wiring(&sample_config(NodeRole::Processor));
    assert!(wiring.role_components.contains(&"block-producer"));
}

#[test]
fn listener_wiring_contains_external_listener() {
    let wiring = build_runtime_wiring(&sample_config(NodeRole::Listener));
    assert!(wiring.role_components.contains(&"external-listener"));
}

#[test]
fn approver_wiring_contains_quorum_approver() {
    let wiring = build_runtime_wiring(&sample_config(NodeRole::Approver));
    assert!(wiring.role_components.contains(&"quorum-approver"));
}

#[test]
fn regression_runtime_source_routes_network_fault_domain_via_dedicated_module() {
    let runtime_source = include_str!("../runtime.rs");
    let declaration = [
        "#[path = \"runtime_network_fault.rs\"]",
        "mod runtime_network_fault;",
    ]
    .join("\n");
    assert!(
        runtime_source.contains(&declaration),
        "expected runtime module declaration for network fault extraction"
    );
    assert!(
        runtime_source.contains("pub use runtime_network_fault::{"),
        "expected runtime re-export surface for extracted network fault APIs"
    );
    for symbol in [
        "simulate_daemon_network_fault",
        "DeterministicNetworkFaultSimulator",
        "NetworkFaultSimulationError",
        "NetworkFaultSimulationInput",
        "NetworkFaultSimulationReport",
    ] {
        assert!(
            runtime_source.contains(symbol),
            "expected runtime network fault re-export to include `{symbol}`"
        );
    }
    assert!(
        runtime_source.contains("};"),
        "expected re-export block terminator to remain present"
    );
}

#[test]
fn regression_runtime_source_routes_tests_via_dedicated_module_file() {
    let runtime_source = include_str!("../runtime.rs");
    let declaration = [
        "#[cfg(test)]",
        "#[path = \"runtime_tests.rs\"]",
        "mod tests;",
    ]
    .join("\n");
    let inline_pattern = ["#[cfg(test)]", "mod tests {"].join("\n");
    assert!(
        runtime_source.contains(&declaration),
        "expected runtime test module declaration to route through runtime_tests.rs"
    );
    assert!(
        !runtime_source.contains(&inline_pattern),
        "expected inline runtime test body to be removed from runtime.rs"
    );
}

#[test]
fn regression_runtime_tests_source_routes_snapshot_store_domain_via_dedicated_module_file() {
    let runtime_tests_source = include_str!("../runtime_tests.rs");
    let declaration = [
        "#[path = \"runtime_tests_snapshot_store.rs\"]",
        "mod runtime_tests_snapshot_store;",
    ]
    .join("\n");
    assert!(
        runtime_tests_source.contains(&declaration),
        "expected runtime tests source to declare dedicated snapshot-store test module"
    );
    for legacy_marker in [
        "fn functional_in_memory_snapshot_store_round_trips_snapshots()",
        "fn integration_file_snapshot_store_round_trips_snapshots()",
        "fn performance_file_snapshot_store_recovery_scan_stays_within_ci_budget()",
    ] {
        assert!(
            !runtime_tests_source.contains(legacy_marker),
            "expected snapshot-store test `{legacy_marker}` to move out of runtime_tests.rs"
        );
    }
}

#[test]
fn regression_runtime_tests_source_routes_network_fault_domain_via_dedicated_module_file() {
    let runtime_tests_source = include_str!("../runtime_tests.rs");
    let declaration = [
        "#[path = \"runtime_tests_network_fault.rs\"]",
        "mod runtime_tests_network_fault;",
    ]
    .join("\n");
    assert!(
        runtime_tests_source.contains(&declaration),
        "expected runtime tests source to declare dedicated network-fault test module"
    );
    for legacy_marker in [
        "fn unit_network_fault_simulation_rejects_zero_queue_capacity()",
        "fn integration_daemon_network_fault_simulation_reports_overflow_and_degradation()",
        "fn performance_network_fault_simulation_pr_lane_stays_within_budget()",
    ] {
        assert!(
            !runtime_tests_source.contains(legacy_marker),
            "expected network-fault test `{legacy_marker}` to move out of runtime_tests.rs"
        );
    }
}
