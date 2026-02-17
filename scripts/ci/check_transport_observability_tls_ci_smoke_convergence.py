#!/usr/bin/env python3
"""Validate low-cost CI smoke convergence across transport, observability, and TLS contracts."""

from __future__ import annotations

import argparse
import json
import re
from dataclasses import dataclass
from pathlib import Path

REPORT_SCHEMA_VERSION = "kamn.ci.transport-observability-tls-ci-smoke-convergence-report.v1"
REASON_TAXONOMY_VERSION = "kamn.ci.transport-observability-tls-ci-smoke-convergence-reason-taxonomy.v1"
REASON_CODES_CSV = (
    "workflow_file_missing,"
    "ci_tools_file_missing,"
    "strategy_doc_missing,"
    "plan_doc_missing,"
    "transport_ci_smoke_composition_missing,"
    "observability_ci_smoke_composition_missing,"
    "tls_ci_smoke_composition_missing,"
    "transport_local_heavy_command_leaked_in_fast_mode,"
    "observability_local_heavy_command_leaked_in_fast_mode,"
    "tls_local_heavy_command_leaked_in_fast_mode,"
    "ci_fast_gate_transport_run_mode_not_excluded,"
    "ci_fast_gate_observability_run_mode_not_excluded,"
    "ci_fast_gate_tls_deep_lane_not_excluded,"
    "ci_strategy_convergence_markers_missing,"
    "production_plan_convergence_markers_missing,"
    "transport_observability_tls_ci_smoke_seconds_exceeded"
)
REASON_CODES_ORDER = tuple(REASON_CODES_CSV.split(","))

CI_SMOKE_MAX_SECONDS = 120
LOCAL_HEAVY_MAX_SECONDS = 900

TRANSPORT_CI_SMOKE_COMMAND = 'bash "$ROOT_DIR/scripts/ci/test_live_transport_fault_matrix_ci_exclusion_policy.sh"'
OBSERVABILITY_CI_SMOKE_COMMAND = 'bash "$ROOT_DIR/scripts/ci/test_check_observability_endpoint_drift_contract.sh"'
TLS_CI_SMOKE_COMMAND = 'bash "$ROOT_DIR/scripts/ci/test_check_kamn_core_live_https_dependency_posture.sh"'

TRANSPORT_LOCAL_HEAVY_COMMAND = (
    'bash "$ROOT_DIR/scripts/runtime/test_validate_live_transport_fault_matrix_live_contract_lane.sh"'
)
OBSERVABILITY_LOCAL_HEAVY_COMMAND = (
    'bash "$ROOT_DIR/scripts/runtime/test_validate_local_observability_scrape_live_contract_lane.sh"'
)
TLS_LOCAL_HEAVY_COMMAND = 'bash "$ROOT_DIR/scripts/deploy/test_run_gonogo_evidence_contract_lane.sh"'

TRANSPORT_FAST_WORKFLOW_RUN_MODE = "bash scripts/runtime/validate_live_transport_fault_matrix_live.sh --mode run"
OBSERVABILITY_FAST_WORKFLOW_RUN_MODE = "bash scripts/runtime/validate_local_observability_scrape_live.sh --mode run"
TLS_FAST_WORKFLOW_DEEP_MODE = "bash scripts/deploy/run_gonogo_evidence_deep_lane.sh"

STRATEGY_REQUIRED_MARKERS = (
    "### Transport/Observability/TLS CI smoke convergence governance",
    "bash scripts/ci/test_check_transport_observability_tls_ci_smoke_convergence.sh",
    "transport_observability_tls_reason_taxonomy_version=kamn.ci.transport-observability-tls-ci-smoke-convergence-reason-taxonomy.v1",
    "transport_observability_tls_reason_codes_csv=transport_ci_smoke_composition_missing,observability_ci_smoke_composition_missing,tls_ci_smoke_composition_missing,transport_local_heavy_command_leaked_in_fast_mode,observability_local_heavy_command_leaked_in_fast_mode,tls_local_heavy_command_leaked_in_fast_mode,ci_fast_gate_transport_run_mode_not_excluded,ci_fast_gate_observability_run_mode_not_excluded,ci_fast_gate_tls_deep_lane_not_excluded,ci_strategy_convergence_markers_missing,production_plan_convergence_markers_missing,transport_observability_tls_ci_smoke_seconds_exceeded",
    "transport_observability_tls_ci_smoke_max_seconds=120",
    "transport_observability_tls_local_heavy_max_seconds=900",
    "transport_observability_tls_ci_smoke_lane_cost_profile=low",
    "transport_observability_tls_local_heavy_execution_mode=opt_in",
)

PLAN_REQUIRED_MARKERS = (
    "### R27.29 Transport/Observability/TLS CI Smoke Convergence Closure",
    "Active chain: `#4293 -> #4295 -> #4299 -> (#4306, #4307)`.",
    "transport_observability_tls_ci_smoke_convergence_status=verified",
    "transport_observability_tls_reason_taxonomy_version=kamn.ci.transport-observability-tls-ci-smoke-convergence-reason-taxonomy.v1",
    "transport_observability_tls_ci_smoke_max_seconds=120",
    "transport_observability_tls_local_heavy_max_seconds=900",
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
        r'if \[ "\$\{KAMN_CI_TOOLS_FAST_MODE:-false\}" = "true" \]; then(?P<body>.*?)\n\s*exit 0\nfi',
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
        raw_reason_codes.append("transport_observability_tls_ci_smoke_seconds_exceeded")

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

    if workflow_text:
        if TRANSPORT_FAST_WORKFLOW_RUN_MODE in workflow_text:
            raw_reason_codes.append("ci_fast_gate_transport_run_mode_not_excluded")
        if OBSERVABILITY_FAST_WORKFLOW_RUN_MODE in workflow_text:
            raw_reason_codes.append("ci_fast_gate_observability_run_mode_not_excluded")
        if TLS_FAST_WORKFLOW_DEEP_MODE in workflow_text:
            raw_reason_codes.append("ci_fast_gate_tls_deep_lane_not_excluded")

    if fast_mode_block:
        if TRANSPORT_CI_SMOKE_COMMAND not in fast_mode_block:
            raw_reason_codes.append("transport_ci_smoke_composition_missing")
        if OBSERVABILITY_CI_SMOKE_COMMAND not in fast_mode_block:
            raw_reason_codes.append("observability_ci_smoke_composition_missing")
        if TLS_CI_SMOKE_COMMAND not in fast_mode_block:
            raw_reason_codes.append("tls_ci_smoke_composition_missing")

        if TRANSPORT_LOCAL_HEAVY_COMMAND in fast_mode_block:
            raw_reason_codes.append("transport_local_heavy_command_leaked_in_fast_mode")
        if OBSERVABILITY_LOCAL_HEAVY_COMMAND in fast_mode_block:
            raw_reason_codes.append("observability_local_heavy_command_leaked_in_fast_mode")
        if TLS_LOCAL_HEAVY_COMMAND in fast_mode_block:
            raw_reason_codes.append("tls_local_heavy_command_leaked_in_fast_mode")

    if strategy_text:
        if any(marker not in strategy_text for marker in STRATEGY_REQUIRED_MARKERS):
            raw_reason_codes.append("ci_strategy_convergence_markers_missing")

    if plan_text:
        if any(marker not in plan_text for marker in PLAN_REQUIRED_MARKERS):
            raw_reason_codes.append("production_plan_convergence_markers_missing")

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
        "transport_observability_tls_ci_smoke_convergence_status": convergence_status,
        "transport_observability_tls_ci_smoke_max_seconds": CI_SMOKE_MAX_SECONDS,
        "transport_observability_tls_local_heavy_max_seconds": LOCAL_HEAVY_MAX_SECONDS,
        "ci_smoke_lane_cost_profile": "low",
        "local_heavy_lane_execution_mode": "opt_in",
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
    print(
        "transport_observability_tls_ci_smoke_convergence_status="
        f"{convergence_status}"
    )
    print(f"transport_observability_tls_ci_smoke_max_seconds={CI_SMOKE_MAX_SECONDS}")
    print(f"transport_observability_tls_local_heavy_max_seconds={LOCAL_HEAVY_MAX_SECONDS}")
    print("ci_smoke_lane_cost_profile=low")
    print("local_heavy_lane_execution_mode=opt_in")

    return 0 if status == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
