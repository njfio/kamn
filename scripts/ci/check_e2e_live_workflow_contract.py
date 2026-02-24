#!/usr/bin/env python3
"""Fail-closed contract checker for .github/workflows/e2e-live.yml."""

from __future__ import annotations

import argparse
from pathlib import Path

REASON_TAXONOMY_VERSION = "kamn.ci.e2e-live-workflow-contract-reason-taxonomy.v1"
REASON_CODES_CSV = (
    "workflow_file_missing,"
    "strategy_doc_missing,"
    "sdk_direct_job_missing,"
    "sdk_direct_live_toggle_missing,"
    "sdk_direct_external_execution_flag_missing,"
    "sdk_direct_scenarios_not_full_matrix,"
    "kolme_bootstrap_step_missing,"
    "kamn_runtime_bootstrap_missing,"
    "service_health_wait_marker_missing,"
    "ci_strategy_markers_missing"
)
REASON_CODES_ORDER = tuple(REASON_CODES_CSV.split(","))
SDK_DIRECT_FULL_SCENARIOS = (
    "--scenarios "
    "S-01,S-02,S-03,S-04,S-05,S-06,S-07,S-08,S-09,S-10,S-11,S-12,S-13,S-14,S-15"
)
STRATEGY_REQUIRED_MARKERS = (
    "## E2E Live Workflow Contract",
    "python3 scripts/ci/check_e2e_live_workflow_contract.py",
    "bash scripts/ci/test_check_e2e_live_workflow_contract.sh",
    "e2e_live_workflow_reason_taxonomy_version=kamn.ci.e2e-live-workflow-contract-reason-taxonomy.v1",
    "e2e_live_workflow_reason_codes_csv=workflow_file_missing,strategy_doc_missing,sdk_direct_job_missing,sdk_direct_live_toggle_missing,sdk_direct_external_execution_flag_missing,sdk_direct_scenarios_not_full_matrix,kolme_bootstrap_step_missing,kamn_runtime_bootstrap_missing,service_health_wait_marker_missing,ci_strategy_markers_missing",
    "e2e_live_workflow_contract_status=verified|violation",
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate that e2e-live workflow keeps local live execution invariants."
    )
    parser.add_argument(
        "--workflow-file",
        default=".github/workflows/e2e-live.yml",
    )
    parser.add_argument(
        "--strategy-doc",
        default="docs/ci/strategy.md",
    )
    return parser.parse_args()


def normalize_reason_codes(codes: list[str]) -> list[str]:
    observed = set(codes)
    return [code for code in REASON_CODES_ORDER if code in observed]


def reason_codes_value(codes: list[str]) -> str:
    if not codes:
        return "none"
    return ",".join(codes)


def get_sdk_direct_section(workflow_text: str) -> str:
    start = workflow_text.find("  e2e-sdk-direct:")
    if start < 0:
        return ""
    mcp_start = workflow_text.find("  e2e-mcp-agent:", start)
    if mcp_start < 0:
        return workflow_text[start:]
    return workflow_text[start:mcp_start]


def add_reason(codes: list[str], reason: str) -> None:
    if reason not in codes:
        codes.append(reason)


def main() -> int:
    args = parse_args()
    workflow_path = Path(args.workflow_file)
    strategy_path = Path(args.strategy_doc)
    raw_reasons: list[str] = []

    workflow_text = ""
    if not workflow_path.is_file():
        add_reason(raw_reasons, "workflow_file_missing")
    else:
        workflow_text = workflow_path.read_text(encoding="utf-8")

    strategy_text = ""
    if not strategy_path.is_file():
        add_reason(raw_reasons, "strategy_doc_missing")
    else:
        strategy_text = strategy_path.read_text(encoding="utf-8")

    sdk_section = ""
    if workflow_text:
        sdk_section = get_sdk_direct_section(workflow_text)
        if not sdk_section:
            add_reason(raw_reasons, "sdk_direct_job_missing")

    if sdk_section:
        if 'KAMN_E2E_SDK_DIRECT_LIVE: "1"' not in sdk_section:
            add_reason(raw_reasons, "sdk_direct_live_toggle_missing")

        if "--enable-external-execution" not in sdk_section:
            add_reason(raw_reasons, "sdk_direct_external_execution_flag_missing")

        if SDK_DIRECT_FULL_SCENARIOS not in sdk_section:
            add_reason(raw_reasons, "sdk_direct_scenarios_not_full_matrix")

        if (
            "git clone https://github.com/fpco/kolme /tmp/kolme" not in sdk_section
            or "/tmp/kolme/target/release/example-p2p" not in sdk_section
            or "api-server" not in sdk_section
        ):
            add_reason(raw_reasons, "kolme_bootstrap_step_missing")

        if (
            "--role processor" not in sdk_section
            or "--role listener" not in sdk_section
            or "--role approver" not in sdk_section
        ):
            add_reason(raw_reasons, "kamn_runtime_bootstrap_missing")

        if (
            "http://127.0.0.1:8080/healthz" not in sdk_section
            or "http://127.0.0.1:8081/healthz" not in sdk_section
            or "http://127.0.0.1:8082/healthz" not in sdk_section
            or "wait_for_port 127.0.0.1 3000" not in sdk_section
            or 'wait_for_http "http://127.0.0.1:3000/healthz"' not in sdk_section
        ):
            add_reason(raw_reasons, "service_health_wait_marker_missing")

    if strategy_text:
        if any(marker not in strategy_text for marker in STRATEGY_REQUIRED_MARKERS):
            add_reason(raw_reasons, "ci_strategy_markers_missing")

    reasons = normalize_reason_codes(raw_reasons)
    reason_value = reason_codes_value(reasons)
    status = "pass" if not reasons else "fail"
    final_decision = "GO" if status == "pass" else "NO-GO"
    contract_status = "verified" if status == "pass" else "violation"

    print(f"status={status}")
    print(f"final_decision={final_decision}")
    print(f"reason_taxonomy_version={REASON_TAXONOMY_VERSION}")
    print(f"reason_codes_csv={REASON_CODES_CSV}")
    print(f"reason_codes_value={reason_value}")
    print(f"e2e_live_workflow_contract_status={contract_status}")

    return 0 if status == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
