#!/usr/bin/env python3
"""Fail-closed checker for test-layering policy/doc contract drift."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

REQUIRED_POLICY_MARKERS = (
    "policy_schema_version=kamn.test-layering-policy.v1",
    "unit_hotspots_required=true",
    "integration_coverage_reduction_allowed=false",
    "ci_fast_gate_cost_budget_required=true",
    "layering_drift_contract=enabled",
)

REQUIRED_STRATEGY_SNIPPETS = (
    "python3 scripts/ci/check_test_layering_policy.py",
    "bash scripts/ci/test_check_test_layering_policy.sh",
    "docs/planning/test_layering_policy.md",
    "layering_marker_missing",
    "Regression: #2694",
)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Validate test-layering policy and CI strategy contract markers."
    )
    parser.add_argument(
        "--policy-doc",
        default="docs/planning/test_layering_policy.md",
        help="Path to the test-layering policy document.",
    )
    parser.add_argument(
        "--strategy-doc",
        default="docs/ci/strategy.md",
        help="Path to the CI strategy document.",
    )
    parser.add_argument(
        "--output-json",
        required=True,
        help="Path to write checker decision report JSON.",
    )
    return parser


def read_text(path: Path) -> str:
    if not path.is_file():
        raise FileNotFoundError(path)
    return path.read_text(encoding="utf-8")


def collect_failures(
    policy_text: str,
    strategy_text: str,
) -> list[str]:
    failures: list[str] = []
    for marker in REQUIRED_POLICY_MARKERS:
        if marker not in policy_text:
            failures.append(f"layering_marker_missing:{marker}")
    for snippet in REQUIRED_STRATEGY_SNIPPETS:
        if snippet not in strategy_text:
            failures.append(f"strategy_snippet_missing:{snippet}")
    return failures


def write_report(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True), encoding="utf-8")


def main() -> int:
    args = build_parser().parse_args()
    policy_doc = Path(args.policy_doc).resolve()
    strategy_doc = Path(args.strategy_doc).resolve()
    output_json = Path(args.output_json).resolve()

    failures: list[str] = []
    policy_text = ""
    strategy_text = ""

    try:
        policy_text = read_text(policy_doc)
    except FileNotFoundError:
        failures.append(f"policy_doc_missing:{policy_doc}")

    try:
        strategy_text = read_text(strategy_doc)
    except FileNotFoundError:
        failures.append(f"strategy_doc_missing:{strategy_doc}")

    if policy_text and strategy_text:
        failures.extend(collect_failures(policy_text, strategy_text))

    report = {
        "policy_doc": str(policy_doc),
        "strategy_doc": str(strategy_doc),
        "policy_schema_version": "kamn.test-layering-policy.v1",
        "policy_markers_checked": len(REQUIRED_POLICY_MARKERS),
        "strategy_snippets_checked": len(REQUIRED_STRATEGY_SNIPPETS),
        "final_decision": "GO" if not failures else "NO-GO",
        "reason_codes": failures,
    }
    write_report(output_json, report)

    if failures:
        print("test-layering policy check failed:", file=sys.stderr)
        for reason in failures:
            print(reason, file=sys.stderr)
        return 1

    print("test-layering policy check passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
