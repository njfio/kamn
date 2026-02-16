#!/usr/bin/env python3
"""Enforce kamn-core missing-docs graduation velocity thresholds."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

THROUGHPUT_REPORT_SCHEMA_VERSION = "kamn.ci.kamn-core-missing-docs-throughput-report.v1"
VELOCITY_BASELINE_SCHEMA_VERSION = "kamn.ci.kamn-core-missing-docs-velocity-baseline.v1"
VELOCITY_THRESHOLD_SCHEMA_VERSION = "kamn.ci.kamn-core-missing-docs-velocity-thresholds.v1"
VELOCITY_POLICY_SCHEMA_VERSION = "kamn.ci.kamn-core-missing-docs-velocity-policy.v1"
VELOCITY_REASON_TAXONOMY_VERSION = (
    "kamn.ci.kamn-core-missing-docs-velocity-reason-taxonomy.v1"
)
VELOCITY_REASON_CODES_CSV = ",".join(
    [
        "allowlist_fully_graduated",
        "baseline_window_not_elapsed",
        "ci_local_docs_velocity_window_boundary_exceeded",
        "multiple_policy_violations",
        "stagnation_window_exceeded",
        "velocity_target_met",
        "velocity_threshold_config_invalid",
        "velocity_window_under_threshold",
        "window_not_elapsed",
    ]
)
CI_LOCAL_MAX_VELOCITY_WINDOW_COMMITS = 240
DEFAULT_CRATE_NAME = "kamn-core"


def load_json_object(path: Path, *, label: str) -> dict[str, Any]:
    if not path.is_file():
        raise ValueError(f"{label} not found: {path}")
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise ValueError(f"{label} must be a JSON object: {path}")
    return payload


def require_int(payload: dict[str, Any], key: str, *, label: str) -> int:
    value = payload.get(key)
    if not isinstance(value, int):
        raise ValueError(f"{label} {key} must be an integer")
    return value


def require_number(payload: dict[str, Any], key: str, *, label: str) -> float:
    value = payload.get(key)
    if not isinstance(value, (int, float)):
        raise ValueError(f"{label} {key} must be a number")
    return float(value)


def require_bool(payload: dict[str, Any], key: str, *, label: str) -> bool:
    value = payload.get(key)
    if not isinstance(value, bool):
        raise ValueError(f"{label} {key} must be a boolean")
    return value


def validate_throughput_report(payload: dict[str, Any]) -> dict[str, Any]:
    if payload.get("schema_version") != THROUGHPUT_REPORT_SCHEMA_VERSION:
        raise ValueError(
            "throughput report schema_version must be "
            f"{THROUGHPUT_REPORT_SCHEMA_VERSION}"
        )

    crate = payload.get("crate")
    if crate != DEFAULT_CRATE_NAME:
        raise ValueError(f"throughput report crate must be {DEFAULT_CRATE_NAME}")

    commit_count = require_int(payload, "commit_count", label="throughput report")
    if commit_count <= 0:
        raise ValueError("throughput report commit_count must be positive")

    graduated_module_count = require_int(
        payload, "graduated_module_count", label="throughput report"
    )
    if graduated_module_count < 0:
        raise ValueError("throughput report graduated_module_count must be non-negative")

    allowlisted_module_count = require_int(
        payload, "allowlisted_module_count", label="throughput report"
    )
    if allowlisted_module_count < 0:
        raise ValueError("throughput report allowlisted_module_count must be non-negative")

    target_modules_per_100_commits = require_int(
        payload, "target_modules_per_100_commits", label="throughput report"
    )
    if target_modules_per_100_commits <= 0:
        raise ValueError("throughput report target_modules_per_100_commits must be positive")

    observed_modules_per_100_commits = require_number(
        payload, "observed_modules_per_100_commits", label="throughput report"
    )
    if observed_modules_per_100_commits < 0:
        raise ValueError(
            "throughput report observed_modules_per_100_commits must be non-negative"
        )

    return {
        "commit_count": commit_count,
        "graduated_module_count": graduated_module_count,
        "allowlisted_module_count": allowlisted_module_count,
        "target_modules_per_100_commits": target_modules_per_100_commits,
        "observed_modules_per_100_commits": round(observed_modules_per_100_commits, 4),
    }


def validate_baseline(payload: dict[str, Any]) -> dict[str, int]:
    if payload.get("schema_version") != VELOCITY_BASELINE_SCHEMA_VERSION:
        raise ValueError(
            "velocity baseline schema_version must be "
            f"{VELOCITY_BASELINE_SCHEMA_VERSION}"
        )

    commit_count = require_int(payload, "commit_count", label="velocity baseline")
    if commit_count <= 0:
        raise ValueError("velocity baseline commit_count must be positive")

    graduated_module_count = require_int(
        payload, "graduated_module_count", label="velocity baseline"
    )
    if graduated_module_count < 0:
        raise ValueError("velocity baseline graduated_module_count must be non-negative")

    allowlisted_module_count = require_int(
        payload, "allowlisted_module_count", label="velocity baseline"
    )
    if allowlisted_module_count < 0:
        raise ValueError("velocity baseline allowlisted_module_count must be non-negative")

    return {
        "commit_count": commit_count,
        "graduated_module_count": graduated_module_count,
        "allowlisted_module_count": allowlisted_module_count,
    }


def validate_thresholds(payload: dict[str, Any]) -> dict[str, Any]:
    if payload.get("schema_version") != VELOCITY_THRESHOLD_SCHEMA_VERSION:
        raise ValueError(
            "velocity thresholds schema_version must be "
            f"{VELOCITY_THRESHOLD_SCHEMA_VERSION}"
        )

    max_commits_without_graduation = require_int(
        payload, "max_commits_without_graduation", label="velocity thresholds"
    )
    if max_commits_without_graduation <= 0:
        raise ValueError("velocity thresholds max_commits_without_graduation must be positive")

    velocity_window_commits = require_int(
        payload, "velocity_window_commits", label="velocity thresholds"
    )
    if velocity_window_commits <= 0:
        raise ValueError("velocity thresholds velocity_window_commits must be positive")
    if velocity_window_commits > CI_LOCAL_MAX_VELOCITY_WINDOW_COMMITS:
        raise ValueError("ci_local_docs_velocity_window_boundary_exceeded")

    min_modules_per_100_commits = require_number(
        payload, "min_modules_per_100_commits", label="velocity thresholds"
    )
    if min_modules_per_100_commits < 0:
        raise ValueError(
            "velocity thresholds min_modules_per_100_commits must be non-negative"
        )

    enforce_window_target = require_bool(
        payload, "enforce_window_target", label="velocity thresholds"
    )

    return {
        "max_commits_without_graduation": max_commits_without_graduation,
        "velocity_window_commits": velocity_window_commits,
        "min_modules_per_100_commits": round(min_modules_per_100_commits, 4),
        "enforce_window_target": enforce_window_target,
    }


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def evaluate_velocity_policy(
    *,
    report: dict[str, Any],
    baseline: dict[str, int],
    thresholds: dict[str, Any],
    report_file: Path,
    baseline_file: Path,
    threshold_file: Path,
) -> tuple[dict[str, Any], int]:
    commit_delta = report["commit_count"] - baseline["commit_count"]
    if commit_delta < 0:
        raise ValueError(
            "baseline commit_count is newer than throughput report "
            f"({baseline['commit_count']} > {report['commit_count']})"
        )

    graduated_module_delta = report["graduated_module_count"] - baseline["graduated_module_count"]
    if graduated_module_delta < 0:
        raise ValueError(
            "graduated_module_count regressed below baseline "
            f"({report['graduated_module_count']} < {baseline['graduated_module_count']})"
        )

    allowlisted_module_delta = (
        report["allowlisted_module_count"] - baseline["allowlisted_module_count"]
    )
    allowlist_exhausted = report["allowlisted_module_count"] == 0

    if commit_delta == 0:
        observed_window_modules_per_100_commits = 0.0
    else:
        observed_window_modules_per_100_commits = round(
            (graduated_module_delta * 100.0) / float(commit_delta), 4
        )

    if allowlist_exhausted:
        stagnation_window_exceeded = False
        window_target_applicable = False
        window_target_met = True
        violations: list[str] = []
        status = "pass"
        final_decision = "GO"
        reason_key = "allowlist_fully_graduated"
    else:
        stagnation_window_exceeded = (
            commit_delta >= thresholds["max_commits_without_graduation"]
            and graduated_module_delta == 0
        )
        window_target_applicable = commit_delta >= thresholds["velocity_window_commits"]
        window_target_met = (
            observed_window_modules_per_100_commits
            >= thresholds["min_modules_per_100_commits"]
        )

        violations = []
        if stagnation_window_exceeded:
            violations.append(
                "stagnation window exceeded: "
                f"commit_delta={commit_delta} "
                f"max_commits_without_graduation={thresholds['max_commits_without_graduation']}"
            )
        if (
            thresholds["enforce_window_target"]
            and window_target_applicable
            and not window_target_met
        ):
            violations.append(
                "velocity window target under threshold: "
                f"observed_modules_per_100_commits={observed_window_modules_per_100_commits} "
                f"min_modules_per_100_commits={thresholds['min_modules_per_100_commits']}"
            )

        status = "pass" if not violations else "fail"
        final_decision = "GO" if status == "pass" else "HOLD"

        if status == "pass":
            if commit_delta == 0:
                reason_key = "baseline_window_not_elapsed"
            elif not window_target_applicable:
                reason_key = "window_not_elapsed"
            else:
                reason_key = "velocity_target_met"
        elif len(violations) == 1 and violations[0].startswith("stagnation window exceeded"):
            reason_key = "stagnation_window_exceeded"
        elif len(violations) == 1:
            reason_key = "velocity_window_under_threshold"
        else:
            reason_key = "multiple_policy_violations"

    policy = {
        "schema_version": VELOCITY_POLICY_SCHEMA_VERSION,
        "crate": DEFAULT_CRATE_NAME,
        "status": status,
        "final_decision": final_decision,
        "reason_taxonomy_version": VELOCITY_REASON_TAXONOMY_VERSION,
        "reason_codes_csv": VELOCITY_REASON_CODES_CSV,
        "reason_key": reason_key,
        "reason_codes_value": reason_key,
        "report_file": str(report_file),
        "baseline_file": str(baseline_file),
        "threshold_file": str(threshold_file),
        "report_commit_count": report["commit_count"],
        "baseline_commit_count": baseline["commit_count"],
        "commit_delta": commit_delta,
        "report_graduated_module_count": report["graduated_module_count"],
        "baseline_graduated_module_count": baseline["graduated_module_count"],
        "graduated_module_delta": graduated_module_delta,
        "report_allowlisted_module_count": report["allowlisted_module_count"],
        "baseline_allowlisted_module_count": baseline["allowlisted_module_count"],
        "allowlisted_module_delta": allowlisted_module_delta,
        "allowlist_exhausted": allowlist_exhausted,
        "report_target_modules_per_100_commits": report["target_modules_per_100_commits"],
        "report_observed_modules_per_100_commits": report["observed_modules_per_100_commits"],
        "velocity_window_commits": thresholds["velocity_window_commits"],
        "observed_window_modules_per_100_commits": observed_window_modules_per_100_commits,
        "min_modules_per_100_commits": thresholds["min_modules_per_100_commits"],
        "window_target_applicable": window_target_applicable,
        "window_target_met": window_target_met,
        "max_commits_without_graduation": thresholds["max_commits_without_graduation"],
        "stagnation_window_exceeded": stagnation_window_exceeded,
        "violations": violations,
        "violation_count": len(violations),
    }
    return policy, 0 if status == "pass" else 1


def command_check(args: argparse.Namespace) -> int:
    report_path = Path(args.report_file).resolve()
    baseline_path = Path(args.baseline_file).resolve()
    threshold_path = Path(args.threshold_file).resolve()
    output_path = Path(args.output_json).resolve()

    report_payload = load_json_object(report_path, label="throughput report")
    baseline_payload = load_json_object(baseline_path, label="velocity baseline")
    threshold_payload = load_json_object(threshold_path, label="velocity thresholds")

    report = validate_throughput_report(report_payload)
    baseline = validate_baseline(baseline_payload)
    thresholds = validate_thresholds(threshold_payload)

    policy, exit_code = evaluate_velocity_policy(
        report=report,
        baseline=baseline,
        thresholds=thresholds,
        report_file=report_path,
        baseline_file=baseline_path,
        threshold_file=threshold_path,
    )
    write_json(output_path, policy)

    output_stream = sys.stdout if exit_code == 0 else sys.stderr
    print(f"status={policy['status']}", file=output_stream)
    print(f"final_decision={policy['final_decision']}", file=output_stream)
    print(
        f"reason_taxonomy_version={policy['reason_taxonomy_version']}",
        file=output_stream,
    )
    print(f"reason_codes_csv={policy['reason_codes_csv']}", file=output_stream)
    print(f"reason_key={policy['reason_key']}", file=output_stream)
    print(f"reason_codes_value={policy['reason_codes_value']}", file=output_stream)
    print(f"commit_delta={policy['commit_delta']}", file=output_stream)
    print(f"graduated_module_delta={policy['graduated_module_delta']}", file=output_stream)
    print(
        "observed_window_modules_per_100_commits="
        f"{policy['observed_window_modules_per_100_commits']}",
        file=output_stream,
    )
    print(f"violation_count={policy['violation_count']}", file=output_stream)
    for violation in policy["violations"]:
        print(violation, file=output_stream)
    print(f"policy_report={output_path}", file=output_stream)

    return exit_code


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Enforce kamn-core missing-docs velocity/stagnation thresholds."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    check = subparsers.add_parser("check")
    check.add_argument("--report-file", required=True)
    check.add_argument("--baseline-file", required=True)
    check.add_argument("--threshold-file", required=True)
    check.add_argument("--output-json", required=True)

    return parser


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()
    try:
        if args.command == "check":
            return command_check(args)
    except ValueError as error:
        reason_code = (
            "ci_local_docs_velocity_window_boundary_exceeded"
            if str(error) == "ci_local_docs_velocity_window_boundary_exceeded"
            else "velocity_threshold_config_invalid"
        )
        print("status=fail", file=sys.stderr)
        print("final_decision=HOLD", file=sys.stderr)
        print(
            f"reason_taxonomy_version={VELOCITY_REASON_TAXONOMY_VERSION}",
            file=sys.stderr,
        )
        print(f"reason_codes_csv={VELOCITY_REASON_CODES_CSV}", file=sys.stderr)
        print(f"reason_codes_value={reason_code}", file=sys.stderr)
        print(f"missing-docs velocity guard failed: {error}", file=sys.stderr)
        return 1
    raise AssertionError(f"unsupported command: {args.command}")


if __name__ == "__main__":
    raise SystemExit(main())
