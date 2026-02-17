#!/usr/bin/env python3
"""Validate low-cost CI smoke convergence for partition-finality governance."""

from __future__ import annotations

import argparse
import json
import re
from dataclasses import dataclass
from pathlib import Path

REPORT_SCHEMA_VERSION = "kamn.ci.partition-finality-ci-smoke-convergence-report.v1"
REASON_TAXONOMY_VERSION = (
    "kamn.ci.partition-finality-ci-smoke-convergence-reason-taxonomy.v1"
)
REASON_CODES_CSV = (
    "workflow_file_missing,"
    "ci_tools_file_missing,"
    "strategy_doc_missing,"
    "plan_doc_missing,"
    "libp2p_validate_ci_smoke_composition_missing,"
    "libp2p_policy_ci_smoke_composition_missing,"
    "libp2p_contract_lane_ci_smoke_composition_missing,"
    "libp2p_evidence_ci_smoke_composition_missing,"
    "partition_finality_run_mode_command_leaked_in_fast_mode,"
    "ci_fast_gate_partition_finality_run_mode_not_excluded,"
    "ci_strategy_partition_finality_convergence_markers_missing,"
    "production_plan_partition_finality_convergence_markers_missing,"
    "partition_finality_ci_smoke_seconds_exceeded"
)
REASON_CODES_ORDER = tuple(REASON_CODES_CSV.split(","))

CI_SMOKE_MAX_SECONDS = 120
LOCAL_HEAVY_MAX_SECONDS = 900

LIBP2P_VALIDATE_CI_SMOKE_COMMAND = (
    'bash "$ROOT_DIR/scripts/runtime/test_validate_libp2p_convergence_process_isolated_live.sh"'
)
LIBP2P_POLICY_CI_SMOKE_COMMAND = (
    'bash "$ROOT_DIR/scripts/runtime/test_check_libp2p_convergence_process_isolated_live_policy.sh"'
)
LIBP2P_CONTRACT_CI_SMOKE_COMMAND = (
    'bash "$ROOT_DIR/scripts/runtime/test_validate_libp2p_convergence_process_isolated_live_contract_lane.sh"'
)
LIBP2P_EVIDENCE_CI_SMOKE_COMMAND = (
    'bash "$ROOT_DIR/scripts/runtime/test_check_libp2p_convergence_process_isolated_live_evidence_convergence.sh"'
)
DEEP_LOCAL_HEAVY_COMMAND = (
    'bash "$ROOT_DIR/scripts/runtime/validate_libp2p_convergence_process_isolated_live.sh" '
    "--mode run --lane-profile deep"
)
DEEP_WORKFLOW_COMMAND = (
    "bash scripts/runtime/validate_libp2p_convergence_process_isolated_live.sh "
    "--mode run --lane-profile deep"
)

STRATEGY_REQUIRED_MARKERS = (
    "### Partition-Finality CI smoke convergence governance",
    "python3 scripts/ci/check_partition_finality_ci_smoke_convergence.py",
    "bash scripts/ci/test_check_partition_finality_ci_smoke_convergence.sh",
    "partition_finality_ci_smoke_reason_taxonomy_version=kamn.ci.partition-finality-ci-smoke-convergence-reason-taxonomy.v1",
    "partition_finality_ci_smoke_reason_codes_csv=libp2p_validate_ci_smoke_composition_missing,libp2p_policy_ci_smoke_composition_missing,libp2p_contract_lane_ci_smoke_composition_missing,libp2p_evidence_ci_smoke_composition_missing,partition_finality_run_mode_command_leaked_in_fast_mode,ci_fast_gate_partition_finality_run_mode_not_excluded,ci_strategy_partition_finality_convergence_markers_missing,production_plan_partition_finality_convergence_markers_missing,partition_finality_ci_smoke_seconds_exceeded",
    "partition_finality_ci_smoke_max_seconds=120",
    "partition_finality_local_heavy_max_seconds=900",
    "partition_finality_ci_smoke_lane_cost_profile=low",
    "partition_finality_local_heavy_execution_mode=opt_in",
)

PLAN_REQUIRED_MARKERS = (
    "### R27.30 Partition-Finality CI Smoke Governance Closure",
    "Active chain: `#4250 -> #4254 -> (#4261, #4262)`.",
    "partition_finality_ci_smoke_convergence_status=verified",
    "partition_finality_ci_smoke_reason_taxonomy_version=kamn.ci.partition-finality-ci-smoke-convergence-reason-taxonomy.v1",
    "partition_finality_ci_smoke_max_seconds=120",
    "partition_finality_local_heavy_max_seconds=900",
)


@dataclass
class CheckInputs:
    workflow_file: Path
    ci_tools_file: Path
    strategy_doc: Path
    plan_doc: Path
    max_seconds: int
    output_json: Path | None


def parse_args() -> CheckInputs:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--workflow-file", type=Path, required=True)
    parser.add_argument("--ci-tools-file", type=Path, required=True)
    parser.add_argument("--strategy-doc", type=Path, required=True)
    parser.add_argument("--plan-doc", type=Path, required=True)
    parser.add_argument("--max-seconds", type=int, default=CI_SMOKE_MAX_SECONDS)
    parser.add_argument("--output-json", type=Path)
    args = parser.parse_args()
    return CheckInputs(
        workflow_file=args.workflow_file,
        ci_tools_file=args.ci_tools_file,
        strategy_doc=args.strategy_doc,
        plan_doc=args.plan_doc,
        max_seconds=args.max_seconds,
        output_json=args.output_json,
    )


def extract_fast_mode_block(text: str) -> str:
    fast_mode_match = re.search(
        r'if \[ "\$\{KAMN_CI_TOOLS_FAST_MODE:-false\}" = "true" \]; then(?P<body>.*?)\n\s*echo "Fast-mode CI tool regression tests passed\."\n\s*exit 0\nfi',
        text,
        flags=re.DOTALL,
    )
    if not fast_mode_match:
        return ""
    return fast_mode_match.group("body")


def normalize_reason_codes(reason_codes: list[str]) -> list[str]:
    observed = set(reason_codes)
    return [code for code in REASON_CODES_ORDER if code in observed]


def reason_codes_value(reason_codes: list[str]) -> str:
    return "none" if not reason_codes else ",".join(reason_codes)


def main() -> int:
    args = parse_args()

    raw_reason_codes: list[str] = []

    if args.max_seconds > CI_SMOKE_MAX_SECONDS:
        raw_reason_codes.append("partition_finality_ci_smoke_seconds_exceeded")

    workflow_text = ""
    if not args.workflow_file.exists():
        raw_reason_codes.append("workflow_file_missing")
    else:
        workflow_text = args.workflow_file.read_text(encoding="utf-8")

    ci_tools_text = ""
    fast_mode_block = ""
    if not args.ci_tools_file.exists():
        raw_reason_codes.append("ci_tools_file_missing")
    else:
        ci_tools_text = args.ci_tools_file.read_text(encoding="utf-8")
        fast_mode_block = extract_fast_mode_block(ci_tools_text)

    strategy_text = ""
    if not args.strategy_doc.exists():
        raw_reason_codes.append("strategy_doc_missing")
    else:
        strategy_text = args.strategy_doc.read_text(encoding="utf-8")

    plan_text = ""
    if not args.plan_doc.exists():
        raw_reason_codes.append("plan_doc_missing")
    else:
        plan_text = args.plan_doc.read_text(encoding="utf-8")

    if workflow_text and DEEP_WORKFLOW_COMMAND in workflow_text:
        raw_reason_codes.append("ci_fast_gate_partition_finality_run_mode_not_excluded")

    if fast_mode_block:
        if LIBP2P_VALIDATE_CI_SMOKE_COMMAND not in fast_mode_block:
            raw_reason_codes.append("libp2p_validate_ci_smoke_composition_missing")
        if LIBP2P_POLICY_CI_SMOKE_COMMAND not in fast_mode_block:
            raw_reason_codes.append("libp2p_policy_ci_smoke_composition_missing")
        if LIBP2P_CONTRACT_CI_SMOKE_COMMAND not in fast_mode_block:
            raw_reason_codes.append("libp2p_contract_lane_ci_smoke_composition_missing")
        if LIBP2P_EVIDENCE_CI_SMOKE_COMMAND not in fast_mode_block:
            raw_reason_codes.append("libp2p_evidence_ci_smoke_composition_missing")
        if DEEP_LOCAL_HEAVY_COMMAND in fast_mode_block:
            raw_reason_codes.append("partition_finality_run_mode_command_leaked_in_fast_mode")

    if strategy_text:
        if any(marker not in strategy_text for marker in STRATEGY_REQUIRED_MARKERS):
            raw_reason_codes.append(
                "ci_strategy_partition_finality_convergence_markers_missing"
            )

    if plan_text:
        if any(marker not in plan_text for marker in PLAN_REQUIRED_MARKERS):
            raw_reason_codes.append(
                "production_plan_partition_finality_convergence_markers_missing"
            )

    normalized_reasons = normalize_reason_codes(raw_reason_codes)
    reason_value = reason_codes_value(normalized_reasons)

    status = "pass" if not normalized_reasons else "fail"
    final_decision = "GO" if not normalized_reasons else "NO-GO"
    convergence_status = "verified" if not normalized_reasons else "violation"

    report = {
        "schema_version": REPORT_SCHEMA_VERSION,
        "status": status,
        "final_decision": final_decision,
        "reason_taxonomy_version": REASON_TAXONOMY_VERSION,
        "reason_codes_csv": REASON_CODES_CSV,
        "reason_codes": normalized_reasons,
        "reason_codes_value": reason_value,
        "partition_finality_ci_smoke_convergence_status": convergence_status,
        "partition_finality_ci_smoke_max_seconds": CI_SMOKE_MAX_SECONDS,
        "partition_finality_local_heavy_max_seconds": LOCAL_HEAVY_MAX_SECONDS,
        "partition_finality_ci_smoke_lane_cost_profile": "low",
        "partition_finality_local_heavy_execution_mode": "opt_in",
        "inputs": {
            "workflow_file": str(args.workflow_file),
            "ci_tools_file": str(args.ci_tools_file),
            "strategy_doc": str(args.strategy_doc),
            "plan_doc": str(args.plan_doc),
            "max_seconds": args.max_seconds,
        },
    }

    if args.output_json is not None:
        args.output_json.parent.mkdir(parents=True, exist_ok=True)
        args.output_json.write_text(
            json.dumps(report, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )

    print(f"status={status}")
    print(f"final_decision={final_decision}")
    print(f"reason_taxonomy_version={REASON_TAXONOMY_VERSION}")
    print(f"reason_codes_csv={REASON_CODES_CSV}")
    print(f"reason_codes_value={reason_value}")
    print(f"partition_finality_ci_smoke_convergence_status={convergence_status}")
    print(f"partition_finality_ci_smoke_max_seconds={CI_SMOKE_MAX_SECONDS}")
    print(f"partition_finality_local_heavy_max_seconds={LOCAL_HEAVY_MAX_SECONDS}")
    print("partition_finality_ci_smoke_lane_cost_profile=low")
    print("partition_finality_local_heavy_execution_mode=opt_in")

    return 0 if status == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
