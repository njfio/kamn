const DOC: &str = include_str!("../../../docs/foundation/rust-sdk-alpha.md");

struct DocContractCase {
    case_id: &'static str,
    document_label: &'static str,
    document: &'static str,
    required_markers: &'static [&'static str],
}

const DOC_CONTRACT_CASES: &[DocContractCase] = &[
    DocContractCase {
        case_id: "sdk_live_transport_scope_markers",
        document_label: "docs/foundation/rust-sdk-alpha.md",
        document: DOC,
        required_markers: &[
            "LiveTransportKamnClient",
            "LiveTransportConfig",
            "TransportMode",
            "KamnTransport",
        ],
    },
    DocContractCase {
        case_id: "sdk_live_transport_validation_command_markers",
        document_label: "docs/foundation/rust-sdk-alpha.md",
        document: DOC,
        required_markers: &[
            "bash scripts/sdk/run_local_e2e_demo.sh",
            "bash scripts/sdk/run_localhost_signed_demo.sh",
            "bash scripts/sdk/run_tcp_signed_relay_demo.sh",
            "bash scripts/sdk/run_tcp_failover_reconnect_matrix.sh --lane fast",
            "bash scripts/sdk/run_rust_live_transport_contract_lane.sh",
            "bash scripts/sdk/run_rust_live_transport_deep_lane.sh",
        ],
    },
    DocContractCase {
        case_id: "sdk_regression_transport_mode_mismatch_guard",
        document_label: "docs/foundation/rust-sdk-alpha.md",
        document: DOC,
        required_markers: &["mismatch rejection (`Regression: #620`)"],
    },
    DocContractCase {
        case_id: "sdk_regression_local_e2e_demo_markers",
        document_label: "docs/foundation/rust-sdk-alpha.md",
        document: DOC,
        required_markers: &["`Regression: #770`", "status=ok", "escrow_id=<id>"],
    },
    DocContractCase {
        case_id: "sdk_schema_compatibility_contract_lane_markers",
        document_label: "docs/foundation/rust-sdk-alpha.md",
        document: DOC,
        required_markers: &[
            "## SDK Schema Compatibility Contract",
            "run_sdk_schema_compatibility_contract_lane.sh",
            "check_sdk_schema_compatibility_policy.sh",
            "fixtures/sdk_parity/register_validation_cases.json",
            "kamn.sdk.parity.matrix.v1",
        ],
    },
    DocContractCase {
        case_id: "sdk_live_transport_smoke_parity_budget_contract_lane_markers",
        document_label: "docs/foundation/rust-sdk-alpha.md",
        document: DOC,
        required_markers: &[
            "## Live Transport Smoke Parity Budget Contract",
            "run_live_transport_smoke_parity_lane.sh",
            "check_live_transport_smoke_parity_policy.sh",
            "run_live_transport_smoke_parity_contract_lane.sh",
            "KAMN_SDK_SMOKE_PARITY_MAX_SECONDS",
            "KAMN_SDK_SMOKE_PARITY_MAX_RETRIES",
            "kamn.sdk.live-transport-smoke-parity-report.v1",
        ],
    },
    DocContractCase {
        case_id: "sdk_regression_localhost_signed_demo_markers",
        document_label: "docs/foundation/rust-sdk-alpha.md",
        document: DOC,
        required_markers: &[
            "`Regression: #807`",
            "verified=true",
            "signature=sig:ed25519:baseline-v1:...",
        ],
    },
    DocContractCase {
        case_id: "sdk_regression_tcp_signed_relay_demo_markers",
        document_label: "docs/foundation/rust-sdk-alpha.md",
        document: DOC,
        required_markers: &[
            "`Regression: #822`",
            "adapter=tcp",
            "tcp_signed_relay_listener",
            "tcp_signed_relay_sender",
        ],
    },
    DocContractCase {
        case_id: "sdk_regression_tcp_handshake_replay_guard_markers",
        document_label: "docs/foundation/rust-sdk-alpha.md",
        document: DOC,
        required_markers: &[
            "`Regression: #823`",
            "Forged handshake frames are rejected",
            "conflict: tcp handshake replay detected",
        ],
    },
    DocContractCase {
        case_id: "sdk_regression_tcp_failover_reconnect_matrix_markers",
        document_label: "docs/foundation/rust-sdk-alpha.md",
        document: DOC,
        required_markers: &[
            "`Regression: #824`",
            "kamn.sdk.tcp-failover-reconnect.matrix.v1",
            "fixtures/sdk_failover_reconnect/reconnect_drift_signatures.txt",
            "KAMN_TCP_FAILOVER_DEEP_CADENCE=scheduled",
        ],
    },
    DocContractCase {
        case_id: "sdk_regression_schema_compatibility_drift_guard_markers",
        document_label: "docs/foundation/rust-sdk-alpha.md",
        document: DOC,
        required_markers: &[
            "schema-version drift, case mismatch, or tampered reason codes force `NO-GO` (`Regression: #937`).",
        ],
    },
    DocContractCase {
        case_id: "sdk_regression_smoke_parity_budget_guard_markers",
        document_label: "docs/foundation/rust-sdk-alpha.md",
        document: DOC,
        required_markers: &[
            "retry-budget exhaustion, runtime-budget breaches, or transport parity drift force `NO-GO` (`Regression: #938`).",
        ],
    },
];

#[test]
fn functional_rust_sdk_alpha_doc_contract_cases_require_markers() {
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
fn regression_rust_sdk_alpha_doc_contract_case_inventory_remains_stable() {
    // Regression: #5193
    assert_eq!(DOC_CONTRACT_CASES.len(), 12);
    assert!(DOC_CONTRACT_CASES
        .iter()
        .all(|case| !case.required_markers.is_empty()));
    let total_marker_count: usize = DOC_CONTRACT_CASES
        .iter()
        .map(|case| case.required_markers.len())
        .sum();
    assert_eq!(total_marker_count, 42);
}
