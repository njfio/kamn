const ARCH_NAV_INDEX: &str = include_str!("../../../docs/architecture/README.md");
const CI_STRATEGY_DOC: &str = include_str!("../../../docs/ci/strategy.md");
const KOLME_DEVNET_OPS_DOC: &str = include_str!("../../../docs/planning/kolme-devnet-ops.md");
const NODE_MODULE_MAP_DOC: &str =
    include_str!("../../../docs/architecture/kamn-node-module-map.md");
const OBSERVABILITY_STREAM_DOC: &str = include_str!("../../../docs/observability/streaming.md");
const REPO_README: &str = include_str!("../../../README.md");

struct DocContractCase {
    case_id: &'static str,
    document_label: &'static str,
    document: &'static str,
    required_markers: &'static [&'static str],
}

const DOC_CONTRACT_CASES: &[DocContractCase] = &[
    DocContractCase {
        case_id: "architecture_navigation_index_schema_and_diagrams",
        document_label: "docs/architecture/README.md",
        document: ARCH_NAV_INDEX,
        required_markers: &[
            "schema_version=kamn.docs.architecture-navigation-index.v1",
            "diagram_catalog_status=active",
            "diagram:runtime-layout",
            "diagram:service-runtime",
            "diagram:block-pipeline",
            "diagram:p2p-transport",
            "diagram:kolme-live-integration",
            "diagram:signer-lifecycle",
        ],
    },
    DocContractCase {
        case_id: "architecture_navigation_index_required_artifact_links",
        document_label: "docs/architecture/README.md",
        document: ARCH_NAV_INDEX,
        required_markers: &[
            "docs/architecture/kamn-core-module-map.md",
            "docs/architecture/kamn-node-module-map.md",
            "docs/architecture/runtime-layout.md",
            "docs/architecture/service-runtime.md",
            "docs/architecture/block-pipeline.md",
            "docs/architecture/p2p-transport.md",
            "docs/architecture/kolme-live-integration.md",
            "docs/architecture/kolme-runtime-commit.md",
            "docs/architecture/persistence-backends.md",
            "docs/architecture/did-chain-adapter.md",
            "docs/architecture/adr-kamn-core-live-tls-transport.md",
        ],
    },
    DocContractCase {
        case_id: "readme_references_architecture_navigation_guard",
        document_label: "README.md",
        document: REPO_README,
        required_markers: &["docs/architecture/README.md"],
    },
    DocContractCase {
        case_id: "ci_strategy_references_architecture_navigation_guard",
        document_label: "docs/ci/strategy.md",
        document: CI_STRATEGY_DOC,
        required_markers: &[
            "architecture navigation index guard",
            "cargo test -p kamn-node --test architecture_navigation_docs",
        ],
    },
    DocContractCase {
        case_id: "kolme_devnet_ops_transport_retry_validation_contract_markers",
        document_label: "docs/planning/kolme-devnet-ops.md",
        document: KOLME_DEVNET_OPS_DOC,
        required_markers: &[
            "transport_retry_validation_contract_version=v1",
            "kolme.live.submit.retry",
            "kolme.live.finality.retry",
            "kolme.live.submit.retry.terminal",
            "kolme.live.finality.retry.terminal",
            "terminal_decision=attempt_ceiling_reached",
            "terminal_decision=malformed_response_fail_fast",
            "retry_reconnect_marker_contract_status=verified",
        ],
    },
    DocContractCase {
        case_id: "kolme_devnet_ops_transport_retry_validation_commands",
        document_label: "docs/planning/kolme-devnet-ops.md",
        document: KOLME_DEVNET_OPS_DOC,
        required_markers: &[
            "main_tests::runtime_tests::functional_kolme_live_retry_emits_structured_retry_markers",
            "main_tests::runtime_tests::regression_runtime_kolme_live_submit_retry_exhaustion_emits_terminal_decision_marker",
            "main_tests::runtime_tests::functional_kolme_live_finality_retry_exhaustion_emits_terminal_decision_marker",
        ],
    },
    DocContractCase {
        case_id: "node_module_map_ownership_boundaries",
        document_label: "docs/architecture/kamn-node-module-map.md",
        document: NODE_MODULE_MAP_DOC,
        required_markers: &[
            "# KAMN Node Module Map",
            "src/main.rs",
            "src/cli.rs",
            "src/daemon_shutdown.rs",
            "src/runtime_kolme_live.rs",
            "src/signer.rs",
            "src/wire_payload.rs",
            "main.rs orchestrates only",
        ],
    },
    DocContractCase {
        case_id: "node_module_map_decomposition_regression_markers",
        document_label: "docs/architecture/kamn-node-module-map.md",
        document: NODE_MODULE_MAP_DOC,
        required_markers: &[
            "Regression: #2606",
            "Do not reintroduce parser implementation into src/main.rs",
        ],
    },
    DocContractCase {
        case_id: "observability_streaming_payload_schema_contract",
        document_label: "docs/observability/streaming.md",
        document: OBSERVABILITY_STREAM_DOC,
        required_markers: &[
            "GET /metrics.stream",
            "application/x-ndjson",
            "schema_version=\"kamn.runtime.observability.stream.v1\"",
            "readiness_reason_code",
        ],
    },
    DocContractCase {
        case_id: "observability_streaming_backpressure_and_reconnect_contract",
        document_label: "docs/observability/streaming.md",
        document: OBSERVABILITY_STREAM_DOC,
        required_markers: &[
            "stream_reconnect_churn_status=verified",
            "queue_bound_budget_status=verified",
            "scrape_failure_taxonomy_status=verified",
            "scrape_failure_taxonomy_csv=readiness_failure_drill_status,stream_reconnect_churn_status,queue_bound_budget_status",
        ],
    },
    DocContractCase {
        case_id: "observability_streaming_low_cost_validation_lane_commands",
        document_label: "docs/observability/streaming.md",
        document: OBSERVABILITY_STREAM_DOC,
        required_markers: &[
            "validate_local_observability_scrape_live.sh --mode dry-run",
            "check_local_observability_scrape_live_policy.sh",
            "validate_local_observability_scrape_live_contract_lane.sh",
        ],
    },
];

#[test]
fn functional_doc_contract_harness_required_markers_remain_present() {
    for case in DOC_CONTRACT_CASES {
        for marker in case.required_markers {
            assert!(
                case.document.contains(marker),
                "missing marker in {} for case {}: {}",
                case.document_label,
                case.case_id,
                marker
            );
        }
    }
}

#[test]
fn regression_doc_contract_harness_case_inventory_remains_stable() {
    // Regression: #5184
    assert_eq!(DOC_CONTRACT_CASES.len(), 11);
    assert!(DOC_CONTRACT_CASES
        .iter()
        .all(|case| !case.required_markers.is_empty()));
    let total_marker_count: usize = DOC_CONTRACT_CASES
        .iter()
        .map(|case| case.required_markers.len())
        .sum();
    assert_eq!(total_marker_count, 54);
}
