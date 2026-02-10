#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path


EXPECTED_SCHEMA_VERSION = "kamn.kolme.fork-compatibility-report.v1"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Evaluate fail-closed policy checks for Kolme fork compatibility evidence."
    )
    parser.add_argument("--report-file", required=True)
    parser.add_argument("--expected-upstream-release-tag", required=True)
    parser.add_argument("--expected-fork-release-tag", required=True)
    parser.add_argument("--expected-fork-repo", required=True)
    parser.add_argument("--expected-final-decision", required=True, choices=["GO", "NO-GO"])
    parser.add_argument("--require-reason-code", action="append", default=[])
    parser.add_argument("--ci-fast-gate", required=True, choices=["PASS", "FAIL"])
    parser.add_argument("--output-json", default="")
    return parser.parse_args()


def evaluate(args: argparse.Namespace, report: dict[str, object]) -> tuple[str, list[str]]:
    reason_codes: list[str] = []

    if report.get("schema_version") != EXPECTED_SCHEMA_VERSION:
        reason_codes.append("report_schema_invalid")

    upstream_release_tag = report.get("upstream_release_tag")
    if not isinstance(upstream_release_tag, str) or not upstream_release_tag.strip():
        reason_codes.append("report_upstream_release_tag_missing")
    elif upstream_release_tag != args.expected_upstream_release_tag:
        reason_codes.append("report_upstream_release_tag_mismatch")

    fork_release_tag = report.get("fork_release_tag")
    if not isinstance(fork_release_tag, str) or not fork_release_tag.strip():
        reason_codes.append("report_fork_release_tag_missing")
    elif fork_release_tag != args.expected_fork_release_tag:
        reason_codes.append("report_fork_release_tag_mismatch")

    fork_repo = report.get("fork_repo")
    if not isinstance(fork_repo, str) or not fork_repo.strip():
        reason_codes.append("report_fork_repo_missing")
    elif fork_repo != args.expected_fork_repo:
        reason_codes.append("report_fork_repo_mismatch")

    report_final_decision = report.get("final_decision")
    if report_final_decision not in {"GO", "NO-GO"}:
        reason_codes.append("report_final_decision_invalid")
    elif report_final_decision != args.expected_final_decision:
        reason_codes.append("report_final_decision_mismatch")

    report_reason_codes = report.get("reason_codes")
    if not isinstance(report_reason_codes, list):
        reason_codes.append("report_reason_codes_invalid")
        report_reason_codes = []

    for item in report_reason_codes:
        if not isinstance(item, str) or not item.strip():
            reason_codes.append("report_reason_codes_invalid")
            break

    for required_code in args.require_reason_code:
        if required_code not in report_reason_codes:
            reason_codes.append(f"required_reason_code_missing:{required_code}")

    if report_final_decision == "GO" and report_reason_codes:
        reason_codes.append("report_go_with_reason_codes")
    if report_final_decision == "NO-GO" and not report_reason_codes:
        reason_codes.append("report_no_go_missing_reason_codes")

    if args.ci_fast_gate != "PASS":
        reason_codes.append("ci_fast_gate_failed")

    final_decision = "GO" if not reason_codes else "NO-GO"
    return final_decision, reason_codes


def main() -> int:
    args = parse_args()
    report_path = Path(args.report_file).resolve()
    report = json.loads(report_path.read_text(encoding="utf-8"))
    final_decision, reason_codes = evaluate(args, report)

    policy_report = {
        "schema_version": "kamn.kolme.fork-compatibility-policy-report.v1",
        "report_file": str(report_path),
        "expected_upstream_release_tag": args.expected_upstream_release_tag,
        "expected_fork_release_tag": args.expected_fork_release_tag,
        "expected_fork_repo": args.expected_fork_repo,
        "expected_final_decision": args.expected_final_decision,
        "required_reason_codes": args.require_reason_code,
        "ci_fast_gate": args.ci_fast_gate,
        "reason_codes": reason_codes,
        "final_decision": final_decision,
    }

    if args.output_json:
        output_file = Path(args.output_json).resolve()
        output_file.parent.mkdir(parents=True, exist_ok=True)
        output_file.write_text(
            json.dumps(policy_report, sort_keys=True, indent=2) + "\n", encoding="utf-8"
        )

    status = "ok" if final_decision == "GO" else "fail"
    failed_checks = ",".join(reason_codes) if reason_codes else "none"
    print(f"status={status}")
    print(f"report_file={report_path}")
    print(f"final_decision={final_decision}")
    print(f"failed_checks={failed_checks}")
    if args.output_json:
        print(f"policy_report_file={Path(args.output_json).resolve()}")

    return 0 if final_decision == "GO" else 1


if __name__ == "__main__":
    raise SystemExit(main())
