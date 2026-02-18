#!/usr/bin/env python3
"""Validate cutover contract-lane CI exclusion policy markers."""

from __future__ import annotations

import argparse
import json
import time
from pathlib import Path
from typing import Any

REASON_TAXONOMY_VERSION = "kamn.ci.cutover-ci-exclusion-policy-reason-taxonomy.v1"
REASON_CODES = [
    "cutover_contract_lane_missing_in_ci_fast_gate",
    "cutover_rollback_deep_lane_leaked_into_ci_fast_gate",
    "cutover_contract_test_missing_in_ci_tools",
    "cutover_deep_lane_test_leaked_into_ci_tools",
    "ci_strategy_cutover_exclusion_markers_missing",
    "ci_strategy_cutover_policy_command_missing",
    "runtime_budget_exceeded",
]
REASON_CODES_CSV = ",".join(REASON_CODES)

CONTRACT_LANE_COMMAND = "bash scripts/cutover/run_cutover_rollback_contract_lane.sh"
DEEP_LANE_COMMAND = "bash scripts/cutover/run_cutover_rollback_deep_lane.sh"
CI_TOOLS_CONTRACT_TEST = "scripts/cutover/test_run_cutover_rollback_contract_lane.sh"
CI_TOOLS_POLICY_TEST = "scripts/cutover/test_check_cutover_ci_exclusion_policy.sh"
STRATEGY_REQUIRED_MARKERS = (
    "cutover_ci_exclusion_policy_reason_taxonomy_version="
    "kamn.ci.cutover-ci-exclusion-policy-reason-taxonomy.v1",
    "cutover_ci_exclusion_policy_reason_codes_csv="
    "cutover_contract_lane_missing_in_ci_fast_gate,cutover_rollback_deep_lane_"
    "leaked_into_ci_fast_gate,cutover_contract_test_missing_in_ci_tools,"
    "cutover_deep_lane_test_leaked_into_ci_tools,ci_strategy_cutover_exclusion_"
    "markers_missing,ci_strategy_cutover_policy_command_missing,runtime_budget_exceeded",
    "cutover_rollback_deep_lane_local_only=true",
    "cutover_rollback_deep_lane_excluded_from_ci_fast_gate=true",
)
STRATEGY_POLICY_COMMAND = (
    "python3 scripts/cutover/check_cutover_ci_exclusion_policy.py --workflow-file "
    ".github/workflows/ci-fast-gate.yml --ci-tools-file scripts/ci/test_ci_tools.sh "
    "--strategy-doc docs/ci/strategy.md --max-seconds 120 --output-json "
    "/tmp/cutover-ci-exclusion-policy-report.json"
)


def _read_text(path: Path, label: str) -> str:
    if not path.is_file():
        raise SystemExit(f"{label} not found: {path}")
    return path.read_text(encoding="utf-8")


def _write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Validate cutover rollback contract-lane CI exclusion policy."
    )
    parser.add_argument("--workflow-file", required=True)
    parser.add_argument("--ci-tools-file", required=True)
    parser.add_argument("--strategy-doc", required=True)
    parser.add_argument("--max-seconds", type=int, default=120)
    parser.add_argument("--output-json", default="")
    return parser


def main(argv: list[str]) -> int:
    args = _build_parser().parse_args(argv)
    if args.max_seconds < 0:
        raise SystemExit("--max-seconds must be >= 0")

    started = time.time()
    workflow_file = Path(args.workflow_file)
    ci_tools_file = Path(args.ci_tools_file)
    strategy_doc = Path(args.strategy_doc)

    workflow_text = _read_text(workflow_file, "workflow file")
    ci_tools_text = _read_text(ci_tools_file, "ci tools file")
    strategy_text = _read_text(strategy_doc, "strategy doc")

    reasons: list[str] = []
    if CONTRACT_LANE_COMMAND not in workflow_text:
        reasons.append("cutover_contract_lane_missing_in_ci_fast_gate")
    if DEEP_LANE_COMMAND in workflow_text:
        reasons.append("cutover_rollback_deep_lane_leaked_into_ci_fast_gate")
    if CI_TOOLS_CONTRACT_TEST not in ci_tools_text:
        reasons.append("cutover_contract_test_missing_in_ci_tools")
    if DEEP_LANE_COMMAND in ci_tools_text:
        reasons.append("cutover_deep_lane_test_leaked_into_ci_tools")
    if CI_TOOLS_POLICY_TEST not in ci_tools_text:
        reasons.append("cutover_contract_test_missing_in_ci_tools")

    if any(marker not in strategy_text for marker in STRATEGY_REQUIRED_MARKERS):
        reasons.append("ci_strategy_cutover_exclusion_markers_missing")
    if STRATEGY_POLICY_COMMAND not in strategy_text:
        reasons.append("ci_strategy_cutover_policy_command_missing")

    elapsed_seconds = int(time.time() - started)
    if elapsed_seconds > args.max_seconds:
        reasons.append("runtime_budget_exceeded")

    unique_reasons = [reason for reason in REASON_CODES if reason in set(reasons)]
    status = "pass" if not unique_reasons else "fail"
    final_decision = "GO" if not unique_reasons else "NO-GO"
    reason_codes_value = "none" if not unique_reasons else ",".join(unique_reasons)
    docs_contract_status = (
        "failed"
        if (
            "ci_strategy_cutover_exclusion_markers_missing" in unique_reasons
            or "ci_strategy_cutover_policy_command_missing" in unique_reasons
        )
        else "verified"
    )

    payload = {
        "schema_version": "kamn.ci.cutover-ci-exclusion-policy-report.v1",
        "status": status,
        "final_decision": final_decision,
        "reason_taxonomy_version": REASON_TAXONOMY_VERSION,
        "reason_codes_csv": REASON_CODES_CSV,
        "reason_codes_value": reason_codes_value,
        "reason_codes": unique_reasons,
        "workflow_file": str(workflow_file),
        "ci_tools_file": str(ci_tools_file),
        "strategy_doc": str(strategy_doc),
        "contract_lane_command_required": CONTRACT_LANE_COMMAND,
        "deep_lane_command_forbidden": DEEP_LANE_COMMAND,
        "cutover_contract_lane_in_ci_fast_gate": CONTRACT_LANE_COMMAND in workflow_text,
        "cutover_deep_lane_excluded_from_ci_fast_gate": DEEP_LANE_COMMAND not in workflow_text,
        "cutover_contract_test_in_ci_tools": CI_TOOLS_CONTRACT_TEST in ci_tools_text,
        "cutover_policy_test_in_ci_tools": CI_TOOLS_POLICY_TEST in ci_tools_text,
        "cutover_deep_lane_excluded_from_ci_tools": DEEP_LANE_COMMAND not in ci_tools_text,
        "docs_contract_status": docs_contract_status,
        "elapsed_seconds": elapsed_seconds,
        "max_seconds": args.max_seconds,
    }

    if args.output_json:
        _write_json(Path(args.output_json), payload)

    print(f"status={status}")
    print(f"final_decision={final_decision}")
    print(f"reason_taxonomy_version={REASON_TAXONOMY_VERSION}")
    print(f"reason_codes_csv={REASON_CODES_CSV}")
    print(f"reason_codes_value={reason_codes_value}")
    print(
        "cutover_contract_lane_in_ci_fast_gate="
        f"{'true' if payload['cutover_contract_lane_in_ci_fast_gate'] else 'false'}"
    )
    print(
        "cutover_deep_lane_excluded_from_ci_fast_gate="
        f"{'true' if payload['cutover_deep_lane_excluded_from_ci_fast_gate'] else 'false'}"
    )
    print(
        "cutover_contract_test_in_ci_tools="
        f"{'true' if payload['cutover_contract_test_in_ci_tools'] else 'false'}"
    )
    print(
        "cutover_policy_test_in_ci_tools="
        f"{'true' if payload['cutover_policy_test_in_ci_tools'] else 'false'}"
    )
    print(
        "cutover_deep_lane_excluded_from_ci_tools="
        f"{'true' if payload['cutover_deep_lane_excluded_from_ci_tools'] else 'false'}"
    )
    print(f"docs_contract_status={docs_contract_status}")
    if args.output_json:
        print(f"report_file={Path(args.output_json).resolve()}")

    return 0 if final_decision == "GO" else 1


if __name__ == "__main__":
    raise SystemExit(main(__import__("sys").argv[1:]))
