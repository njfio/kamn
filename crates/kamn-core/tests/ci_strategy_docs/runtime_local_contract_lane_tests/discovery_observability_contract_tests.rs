use super::support::assert_runtime_local_contract_lane_markers;
use super::super::DOC;
use super::super::fairness_deletion_support::assert_contains_all;

#[test]
fn doc_contains_runtime_libp2p_three_node_discovery_contract_lane_ci_mode_markers() {
    assert_runtime_local_contract_lane_markers(
        "## Runtime Libp2p Three-Node Discovery Live Validation Contract Lane",
        &[
            "validate_libp2p_three_node_discovery_live.sh --mode dry-run --output-json /tmp/libp2p-three-node-discovery-live-summary.json",
            "KAMN_LIBP2P_THREE_NODE_DISCOVERY_LIVE_OPT_IN=1 bash scripts/runtime/validate_libp2p_three_node_discovery_live.sh --mode run --output-json /tmp/libp2p-three-node-discovery-live-summary.json",
            "check_libp2p_three_node_discovery_live_policy.sh --report-file /tmp/libp2p-three-node-discovery-live-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/libp2p-three-node-discovery-live-policy.json",
            "validate_libp2p_three_node_discovery_live_contract_lane.sh --output-json /tmp/libp2p-three-node-discovery-live-contract-lane-report.json --policy-output-json /tmp/libp2p-three-node-discovery-live-policy.json",
            "test_validate_libp2p_three_node_discovery_live.sh",
            "test_check_libp2p_three_node_discovery_live_policy.sh",
            "test_validate_libp2p_three_node_discovery_live_contract_lane.sh",
        ],
        "libp2p three-node discovery run-mode commands remain excluded from ci-fast-gate and ci-tools fast mode.",
        &[
            "libp2p_three_node_discovery_policy_marker_missing:three_node_discovery_status",
            "MissingKademliaBootstrapSeeds",
        ],
        "runtime libp2p discovery",
    );
    assert_contains_all(
        DOC,
        &["Kademlia bootstrap contracts are covered by `cargo test -p kamn-core --test p2p_kademlia_bootstrap`."],
        "runtime libp2p discovery",
    );
}

#[test]
fn doc_contains_runtime_local_observability_scrape_contract_lane_ci_mode_markers() {
    assert_runtime_local_contract_lane_markers(
        "## Runtime Local Observability Scrape Contract Lane",
        &[
            "validate_local_observability_scrape_live.sh --mode dry-run --output-json /tmp/local-observability-scrape-live-summary.json",
            "KAMN_LOCAL_OBSERVABILITY_SCRAPE_OPT_IN=1 bash scripts/runtime/validate_local_observability_scrape_live.sh --mode run --output-json /tmp/local-observability-scrape-live-summary.json",
            "check_local_observability_scrape_live_policy.sh --report-file /tmp/local-observability-scrape-live-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/local-observability-scrape-live-policy.json",
            "validate_local_observability_scrape_live_contract_lane.sh --output-json /tmp/local-observability-scrape-live-contract-lane-report.json --policy-output-json /tmp/local-observability-scrape-live-policy.json",
            "test_validate_local_observability_scrape_live_contract_lane.sh",
            "test_check_local_observability_scrape_live_policy.sh",
        ],
        "local observability scrape run-mode commands remain excluded from ci-fast-gate and ci-tools fast mode.",
        &[
            "docs/observability/streaming.md",
            "schema_version=kamn.runtime.local-observability-scrape-live-report.v1",
            "schema_version=kamn.runtime.local-observability-scrape-live-policy-report.v1",
            "schema_version=kamn.runtime.local-observability-scrape-live-contract-lane-report.v1",
            "local_observability_scrape_policy_marker_missing:scrape_probe_status",
        ],
        "runtime local observability scrape",
    );
}
