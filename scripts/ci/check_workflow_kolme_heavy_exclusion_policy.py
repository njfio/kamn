#!/usr/bin/env python3
"""Validate ci-fast-gate workflow policy for Kolme local-heavy exclusion."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import List

LOCAL_HEAVY_LANE_COMMANDS = [
    "bash scripts/framework/test_assert_local_heavy_opt_in.sh",
    "bash scripts/kolme/test_run_local_kolme_fork_rust_test_matrix_lane.sh",
    "bash scripts/kolme/test_check_local_kolme_fork_rust_test_matrix_policy.sh",
    "bash scripts/kolme/test_run_local_kolme_fork_rust_test_matrix_contract_lane.sh",
    "bash scripts/kolme/test_run_local_bootstrap_health_checks.sh",
    "bash scripts/kolme/test_run_local_e2e_integration_lane.sh",
    "bash scripts/kolme/test_run_local_heavy_validation_matrix.sh",
    "bash scripts/kolme/test_run_local_runtime_commit_live_lane.sh",
    "bash scripts/kolme/test_run_local_runtime_commit_live_finality_evidence_contract_lane.sh",
    "bash scripts/kolme/test_run_local_native_api_parity_live_proof_contract_lane.sh",
    "bash scripts/kolme/test_run_local_kamn_live_runtime_integration_contract_lane.sh",
    "bash scripts/kolme/test_check_local_kamn_live_runtime_real_node_profile_policy.sh",
    "bash scripts/kolme/test_run_local_kamn_live_runtime_real_node_profile_contract_lane.sh",
]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Validate that workflow policy keeps Kolme local-heavy lanes excluded by default "
            "and only enabled through explicit workflow_dispatch opt-in."
        )
    )
    parser.add_argument("--workflow-file", required=True, type=Path)
    parser.add_argument("--output-json", type=Path)
    return parser.parse_args()


def extract_step_block(text: str, step_name: str) -> str:
    step_match = re.search(
        rf"- name:\s*{re.escape(step_name)}(?P<body>(?:\n\s{{6,}}.*)+)",
        text,
    )
    if not step_match:
        return ""
    return step_match.group("body")


def main() -> int:
    args = parse_args()

    if not args.workflow_file.exists():
        print(f"workflow file not found: {args.workflow_file}", file=sys.stderr)
        return 2

    text = args.workflow_file.read_text(encoding="utf-8")
    failed_checks: List[str] = []

    if "workflow_dispatch:" not in text:
        failed_checks.append("workflow_dispatch_trigger_missing")

    input_block_match = re.search(
        r"run_kolme_local_heavy_contract_tests:\n(?P<body>(?:\s{6,}.*\n)+)", text
    )
    if not input_block_match:
        failed_checks.append("workflow_dispatch_input_missing")
    else:
        input_block = input_block_match.group("body")
        if "default: 'false'" not in input_block and 'default: "false"' not in input_block:
            failed_checks.append("workflow_dispatch_input_default_not_false")

    if "CI_ENABLE_KOLME_LOCAL_HEAVY_CONTRACT_TESTS:" not in text:
        failed_checks.append("selector_opt_in_env_missing")

    if "github.event.inputs.run_kolme_local_heavy_contract_tests" not in text:
        failed_checks.append("selector_opt_in_env_not_bound_to_dispatch_input")

    if re.search(
        r"CI_ENABLE_KOLME_LOCAL_HEAVY_CONTRACT_TESTS:\s*['\"]?true['\"]?\s*$",
        text,
        flags=re.MULTILINE,
    ):
        failed_checks.append("selector_opt_in_env_forced_true_literal")

    heavy_step_block = extract_step_block(text, "Run Kolme local-heavy contract lane")
    if not heavy_step_block:
        failed_checks.append("local_heavy_lane_step_missing")
    else:
        if not re.search(
            r"\n\s+if:\s*steps\.scope\.outputs\.run_kolme_local_heavy_contract_tests == 'true'\s*$",
            heavy_step_block,
            flags=re.MULTILINE,
        ):
            failed_checks.append("local_heavy_lane_not_selector_gated")
        missing_local_heavy_commands = [
            command for command in LOCAL_HEAVY_LANE_COMMANDS if command not in heavy_step_block
        ]
        if missing_local_heavy_commands:
            failed_checks.append("local_heavy_lane_commands_missing")

    version_lane_block = extract_step_block(text, "Run Kolme version compatibility contract lane")
    if version_lane_block:
        leaked_commands = [command for command in LOCAL_HEAVY_LANE_COMMANDS if command in version_lane_block]
        if leaked_commands:
            failed_checks.append("local_heavy_lane_commands_in_version_lane")

    if failed_checks:
        status = "fail"
        final_decision = "NO-GO"
    else:
        status = "pass"
        final_decision = "GO"

    report = {
        "schema_version": "kamn.ci.workflow-kolme-heavy-exclusion-policy-report.v1",
        "workflow_file": str(args.workflow_file),
        "status": status,
        "final_decision": final_decision,
        "failed_checks": failed_checks,
    }

    if args.output_json:
        args.output_json.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    print(f"status={status}")
    print(f"final_decision={final_decision}")
    if failed_checks:
        print(f"failed_checks={','.join(failed_checks)}")
        return 1

    print("failed_checks=none")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
