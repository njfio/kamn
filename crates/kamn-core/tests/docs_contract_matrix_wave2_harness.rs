const CI_STRATEGY_DOC: &str = include_str!("../../../docs/ci/strategy.md");
const PERSISTENCE_ROADMAP_DOC: &str =
    include_str!("../../../docs/plans/2026-02-08-production-service-roadmap.md");
const DATA_GOVERNANCE_RETENTION_DOC: &str =
    include_str!("../../../docs/foundation/data-governance-retention.md");
const KEY_MANAGEMENT_ENCRYPTION_DOC: &str =
    include_str!("../../../docs/foundation/key-management-and-encryption.md");
const SDK_PARITY_WAVE_DOC: &str = include_str!("../../../docs/planning/sdk-parity-wave.md");
const GROUP_SENDER_KEY_ROTATION_DOC: &str =
    include_str!("../../../docs/foundation/group-sender-key-rotation.md");
const INCIDENT_READINESS_DOC: &str = include_str!("../../../docs/ops/incident-readiness.md");
const PYTHON_SDK_BETA_DOC: &str = include_str!("../../../docs/foundation/python-sdk-beta.md");
const TYPESCRIPT_SDK_BETA_DOC: &str =
    include_str!("../../../docs/foundation/typescript-sdk-beta.md");
const SERVICE_MARKETPLACE_DOC: &str =
    include_str!("../../../docs/foundation/service-marketplace-discovery.md");
const PR_TEMPLATE_DOC: &str = include_str!("../../../.github/pull_request_template.md");
const TESTING_STRUCTURE_DOC: &str = include_str!("../../../docs/testing/structure.md");

struct DocContractCase {
    case_id: &'static str,
    document_label: &'static str,
    document: &'static str,
    required_markers: &'static [&'static str],
}

const DOC_CONTRACT_CASES: &[DocContractCase] = &[
    DocContractCase {
        case_id: "tls_feature_gate_ci_contract_markers",
        document_label: "docs/ci/strategy.md",
        document: CI_STRATEGY_DOC,
        required_markers: &[
            "## Kolme Live HTTPS Feature-Gate Contract",
            "`live-https` feature remains enabled by default",
            "cargo check -p kamn-core --features live-https",
            "cargo check -p kamn-core --no-default-features",
        ],
    },
    DocContractCase {
        case_id: "persistence_live_validation_roadmap_contract_markers",
        document_label: "docs/plans/2026-02-08-production-service-roadmap.md",
        document: PERSISTENCE_ROADMAP_DOC,
        required_markers: &[
            "Task #3078",
            "Task #3082",
            "channel-snapshot-store:file-default",
            "message-lifecycle-snapshot-store:file-default",
            "runtime-snapshot-store:file-default",
        ],
    },
    DocContractCase {
        case_id: "data_governance_retention_contract_markers",
        document_label: "docs/foundation/data-governance-retention.md",
        document: DATA_GOVERNANCE_RETENTION_DOC,
        required_markers: &[
            "# Data Governance Retention and Redaction Contracts",
            "run_channel_retention_redaction_contract_lane.sh",
            "channel_retention_redaction_contract.py",
            "kamn.channel.retention-redaction-evidence.v1",
            "replay-safe reason-code drift is rejected (`Regression: #930`)",
            "check_channel_retention_redaction_policy.sh",
        ],
    },
    DocContractCase {
        case_id: "key_management_encryption_contract_markers",
        document_label: "docs/foundation/key-management-and-encryption.md",
        document: KEY_MANAGEMENT_ENCRYPTION_DOC,
        required_markers: &[
            "# Key Management and Encryption Contract Rules",
            "run_key_hierarchy_invariant_contract_lane.sh",
            "key_lifecycle_invariant_contract.py",
            "kamn.key-lifecycle.invariant-evidence.v1",
            "replay/stale key activation drift is rejected (`Regression: #931`)",
            "check_key_lifecycle_invariant_policy.sh",
        ],
    },
    DocContractCase {
        case_id: "sdk_example_fixture_drift_contract_markers",
        document_label: "docs/planning/sdk-parity-wave.md",
        document: SDK_PARITY_WAVE_DOC,
        required_markers: &[
            "## SDK Example Fixture Drift Checker Contract (Issue #940)",
            "check_example_fixture_drift.py",
            "run_example_fixture_drift_contract_lane.sh",
            "check_example_fixture_drift_policy.sh",
            "fixtures/sdk_parity/register_validation_snapshot.json",
            "Regression: #940",
        ],
    },
    DocContractCase {
        case_id: "group_sender_key_rotation_contract_markers",
        document_label: "docs/foundation/group-sender-key-rotation.md",
        document: GROUP_SENDER_KEY_ROTATION_DOC,
        required_markers: &[
            "# Group Sender-Key Replay and Ratchet Contract Rules",
            "run_group_sender_replay_ratchet_contract_lane.sh",
            "group_sender_replay_ratchet_contract.py",
            "kamn.group-sender.replay-ratchet-evidence.v1",
            "stale-generation and nonce replay payloads are rejected (`Regression: #932`)",
            "check_group_sender_replay_ratchet_policy.sh",
        ],
    },
    DocContractCase {
        case_id: "incident_readiness_contract_markers",
        document_label: "docs/ops/incident-readiness.md",
        document: INCIDENT_READINESS_DOC,
        required_markers: &[
            "## Go/No-Go Incident Readiness Bundle Convergence Gate (Issue #4470)",
            "--incident-readiness-report-file",
            "reason_taxonomy_version=kamn.release.gonogo-incident-readiness-convergence-reason-taxonomy.v1",
            "reason_codes_csv=none|<csv>",
            "Mismatch and tamper failure cases",
            "gonogo_incident_readiness_reason_taxonomy_schema_mismatch",
            "gonogo_incident_readiness_freshness_window_exceeded",
            "incident readiness gate convergence mismatch",
            "Regression: #4469",
        ],
    },
    DocContractCase {
        case_id: "python_sdk_beta_contract_markers",
        document_label: "docs/foundation/python-sdk-beta.md",
        document: PYTHON_SDK_BETA_DOC,
        required_markers: &[
            "## Scope Delivered",
            "LiveTransportConfig",
            "LiveKAMNClient",
            "TransportModeMismatchError",
            "bash scripts/sdk/run_live_transport_parity_contract_lane.sh",
            "bash scripts/sdk/run_live_transport_parity_deep_lane.sh",
            "python3 -m unittest tests/python/test_sdk.py",
            "contract drift (`Regression: #620`)",
        ],
    },
    DocContractCase {
        case_id: "typescript_sdk_beta_contract_markers",
        document_label: "docs/foundation/typescript-sdk-beta.md",
        document: TYPESCRIPT_SDK_BETA_DOC,
        required_markers: &[
            "## Scope Delivered",
            "packages/kamn-schema",
            "packages/kamn-sdk",
            "validateCanonicalMessageEnvelope(...)",
            "KAMNClient",
            "LiveTransportKAMNClient",
            "TransportModeMismatchError",
            "## Fast and Cost-Effective Validation",
            "node --experimental-strip-types --test",
            "bash scripts/sdk/run_live_transport_parity_contract_lane.sh",
            "npm --prefix packages/kamn-schema test",
            "npm --prefix packages/kamn-sdk test",
            "nonce must be a positive integer.",
            "proof verification method must be bound to sender DID",
            "`TransportModeMismatchError` (`Regression: #620`)",
        ],
    },
    DocContractCase {
        case_id: "service_marketplace_contract_markers",
        document_label: "docs/foundation/service-marketplace-discovery.md",
        document: SERVICE_MARKETPLACE_DOC,
        required_markers: &[
            "## Scope Delivered",
            "ServiceListing",
            "MarketplaceSearchFilter",
            "ServiceMarketplaceEngine",
            "NegotiationThreadHook",
            "## Listing Validation Rules",
            "negotiation channel type of `Marketplace`.",
            "provider DID membership in the negotiation channel.",
            "## Discovery and Negotiation Rules",
            "exact tag membership.",
        ],
    },
    DocContractCase {
        case_id: "shell_surface_governance_pr_template_contract_markers",
        document_label: ".github/pull_request_template.md",
        document: PR_TEMPLATE_DOC,
        required_markers: &[
            "shell_surface_mitigation_issue: #<issue-id>|None",
            "regressed_with_waiver requires shell_surface_mitigation_issue to link #<issue-id>",
        ],
    },
    DocContractCase {
        case_id: "shell_surface_governance_ci_strategy_contract_markers",
        document_label: "docs/ci/strategy.md",
        document: CI_STRATEGY_DOC,
        required_markers: &[
            "shell_surface_mitigation_issue",
            "#<issue-id>|None",
            "regressed_with_waiver` requires `shell_surface_mitigation_issue` to link `#<issue-id>",
        ],
    },
    DocContractCase {
        case_id: "testing_structure_contract_markers",
        document_label: "docs/testing/structure.md",
        document: TESTING_STRUCTURE_DOC,
        required_markers: &[
            "## Main Tests Decomposition Drift Cases (Issue #4452)",
            "main_tests_decomposition_reason_taxonomy_version=kamn.testing.main-tests-decomposition-reason-taxonomy.v1",
            "main_tests_decomposition_reason_codes_csv=main_tests_domain_module_missing,main_tests_inline_monolith_reintroduced,main_tests_structural_budget_boundary_exceeded",
            "main_tests_decomposition_status=verified",
            "main_tests_structural_budget_status=verified",
            "cargo test -p kamn-node --test main_module_extraction_contract -- --nocapture",
            "cargo test -p kamn-core --test docs_contract_matrix_wave2_harness -- --nocapture",
            "bash scripts/ci/test_check_test_harness_loc_soft_budget.sh",
            "bash scripts/ci/test_run_test_harness_loc_soft_budget_contract_lane.sh",
        ],
    },
];

#[test]
fn functional_wave2_doc_contract_cases_require_markers() {
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
fn regression_wave2_doc_contract_case_inventory_remains_stable() {
    // Regression: #5217
    assert_eq!(DOC_CONTRACT_CASES.len(), 13);
    assert!(DOC_CONTRACT_CASES
        .iter()
        .all(|case| !case.required_markers.is_empty()));
    let total_marker_count: usize = DOC_CONTRACT_CASES
        .iter()
        .map(|case| case.required_markers.len())
        .sum();
    assert_eq!(total_marker_count, 89);
}
