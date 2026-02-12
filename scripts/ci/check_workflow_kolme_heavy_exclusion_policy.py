#!/usr/bin/env python3
"""Validate ci-fast-gate workflow policy for Kolme local-heavy exclusion."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import List


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

    heavy_step_match = re.search(
        r"- name:\s*Run Kolme local-heavy contract lane(?P<body>(?:\n\s{6,}.*)+)",
        text,
    )
    if not heavy_step_match:
        failed_checks.append("local_heavy_lane_step_missing")
    else:
        heavy_step_block = heavy_step_match.group("body")
        if not re.search(
            r"\n\s+if:\s*steps\.scope\.outputs\.run_kolme_local_heavy_contract_tests == 'true'\s*$",
            heavy_step_block,
            flags=re.MULTILINE,
        ):
            failed_checks.append("local_heavy_lane_not_selector_gated")

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
