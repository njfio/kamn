#!/usr/bin/env python3
"""Enforce downward-only shell-surface threshold updates with exception linkage."""

from __future__ import annotations

import argparse
import datetime as dt
import json
import os
import re
import subprocess
import sys
from pathlib import Path
from typing import Any

REPORT_SCHEMA_VERSION = "kamn.ci.shell-surface-threshold-ratchet-report.v1"
REASON_TAXONOMY_VERSION = "kamn.ci.shell-surface-threshold-ratchet-reason-taxonomy.v1"
EXCEPTION_SCHEMA_VERSION = "kamn.ci.shell-surface-threshold-ratchet-exception.v1"
REASON_CODES_CSV = (
    "shell_surface_threshold_ratchet_argument_invalid,"
    "shell_surface_threshold_ratchet_exception_applied,"
    "shell_surface_threshold_ratchet_exception_file_invalid,"
    "shell_surface_threshold_ratchet_git_history_unavailable,"
    "shell_surface_threshold_ratchet_output_json_required,"
    "shell_surface_threshold_ratchet_output_write_failed,"
    "shell_surface_threshold_ratchet_regression_unwaived,"
    "shell_surface_threshold_ratchet_threshold_order_invalid,"
    "shell_surface_threshold_ratchet_threshold_parse_invalid"
)
ISSUE_REF_PATTERN = re.compile(r"^#[0-9]+$")
ALLOWED_THRESHOLD_KEYS = {
    "HARD_SHELL_LOC_MAX",
    "WARN_SHELL_RUST_RATIO_MAX",
    "FAIL_SHELL_RUST_RATIO_MAX",
}


def fail(message: str) -> None:
    raise RuntimeError(message)


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Check that shell-surface threshold updates only ratchet downward unless a "
            "valid tracked exception is provided."
        )
    )
    parser.add_argument("--repo-root", default=".", help="Repository root.")
    parser.add_argument(
        "--hard-ceiling-file",
        default=".ci/shell-loc-hard-ceiling.env",
        help="Current hard-ceiling env file.",
    )
    parser.add_argument(
        "--ratio-threshold-file",
        default=".ci/shell-rust-ratio-guardrail.env",
        help="Current shell-rust ratio guardrail env file.",
    )
    parser.add_argument(
        "--baseline-hard-ceiling-file",
        default="",
        help="Optional explicit baseline hard-ceiling env file (bypasses git baseline lookup).",
    )
    parser.add_argument(
        "--baseline-ratio-threshold-file",
        default="",
        help="Optional explicit baseline ratio env file (bypasses git baseline lookup).",
    )
    parser.add_argument(
        "--base-ref",
        default="",
        help="Base ref for git baseline lookup (defaults to GITHUB_BASE_REF or main).",
    )
    parser.add_argument(
        "--ratchet-exception-file",
        default=".ci/shell-surface-threshold-ratchet-exception.json",
        help="Optional exception metadata file used only when ratchet regressions are detected.",
    )
    parser.add_argument(
        "--output-json",
        required=True,
        help="Path to write checker report JSON.",
    )
    return parser.parse_args(argv)


def resolve_path(*, repo_root: Path, value: str) -> Path:
    path = Path(value)
    if path.is_absolute():
        return path
    return (repo_root / path).resolve()


def to_repo_relative(path: Path, repo_root: Path) -> str:
    try:
        return path.resolve().relative_to(repo_root.resolve()).as_posix()
    except ValueError:
        return path.as_posix()


def parse_env_text(*, text: str, source_label: str, required_keys: set[str]) -> dict[str, str]:
    parsed: dict[str, str] = {}
    for line in text.splitlines():
        trimmed = line.strip()
        if not trimmed or trimmed.startswith("#"):
            continue
        if "=" not in trimmed:
            continue
        key, value = trimmed.split("=", 1)
        parsed[key.strip()] = value.strip()
    missing_keys = sorted(required_keys.difference(parsed.keys()))
    if missing_keys:
        fail(f"{source_label} missing required keys: {','.join(missing_keys)}")
    return parsed


def parse_thresholds(
    *,
    hard_env: dict[str, str],
    ratio_env: dict[str, str],
    source_label: str,
) -> dict[str, float]:
    try:
        hard_max = int(hard_env["HARD_SHELL_LOC_MAX"])
    except ValueError as exc:
        fail(f"{source_label} HARD_SHELL_LOC_MAX must be an integer: {exc}")
    try:
        warn_ratio = float(ratio_env["WARN_SHELL_RUST_RATIO_MAX"])
        fail_ratio = float(ratio_env["FAIL_SHELL_RUST_RATIO_MAX"])
    except ValueError as exc:
        fail(f"{source_label} ratio threshold values must be decimals: {exc}")

    if hard_max <= 0:
        fail(f"{source_label} HARD_SHELL_LOC_MAX must be > 0")
    if warn_ratio <= 0 or fail_ratio <= 0:
        fail(f"{source_label} ratio threshold values must be > 0")
    if warn_ratio > fail_ratio:
        fail(
            f"{source_label} WARN_SHELL_RUST_RATIO_MAX ({warn_ratio}) must be <= "
            f"FAIL_SHELL_RUST_RATIO_MAX ({fail_ratio})"
        )

    return {
        "HARD_SHELL_LOC_MAX": float(hard_max),
        "WARN_SHELL_RUST_RATIO_MAX": warn_ratio,
        "FAIL_SHELL_RUST_RATIO_MAX": fail_ratio,
    }


def load_thresholds_from_files(
    *,
    hard_file: Path,
    ratio_file: Path,
    source_label: str,
) -> dict[str, float]:
    if not hard_file.is_file():
        fail(f"{source_label} hard-ceiling file not found: {hard_file}")
    if not ratio_file.is_file():
        fail(f"{source_label} ratio-threshold file not found: {ratio_file}")

    hard_env = parse_env_text(
        text=hard_file.read_text(encoding="utf-8"),
        source_label=source_label,
        required_keys={"HARD_SHELL_LOC_MAX"},
    )
    ratio_env = parse_env_text(
        text=ratio_file.read_text(encoding="utf-8"),
        source_label=source_label,
        required_keys={"WARN_SHELL_RUST_RATIO_MAX", "FAIL_SHELL_RUST_RATIO_MAX"},
    )
    return parse_thresholds(hard_env=hard_env, ratio_env=ratio_env, source_label=source_label)


def load_thresholds_from_git_base(
    *,
    repo_root: Path,
    base_ref: str,
    hard_file: Path,
    ratio_file: Path,
) -> tuple[dict[str, float], str]:
    if not hard_file.is_file() or not ratio_file.is_file():
        fail("threshold files must exist in working tree before git baseline lookup")

    base_target = ""
    if base_ref:
        for candidate in (f"origin/{base_ref}", base_ref):
            result = subprocess.run(
                ["git", "-C", str(repo_root), "rev-parse", "--verify", candidate],
                check=False,
                text=True,
                capture_output=True,
            )
            if result.returncode == 0:
                base_target = candidate
                break
    else:
        for candidate in ("origin/main", "main", "HEAD~1"):
            result = subprocess.run(
                ["git", "-C", str(repo_root), "rev-parse", "--verify", candidate],
                check=False,
                text=True,
                capture_output=True,
            )
            if result.returncode == 0:
                base_target = candidate
                break

    if not base_target:
        fail("unable to resolve base ref for ratchet baseline lookup")

    merge_base = subprocess.run(
        ["git", "-C", str(repo_root), "merge-base", "HEAD", base_target],
        check=False,
        text=True,
        capture_output=True,
    )
    if merge_base.returncode != 0:
        fail("unable to compute merge-base for ratchet baseline lookup")
    base_commit = merge_base.stdout.strip()
    if not base_commit:
        fail("merge-base resolved to an empty commit id")

    hard_rel = to_repo_relative(hard_file, repo_root)
    ratio_rel = to_repo_relative(ratio_file, repo_root)
    hard_show = subprocess.run(
        ["git", "-C", str(repo_root), "show", f"{base_commit}:{hard_rel}"],
        check=False,
        text=True,
        capture_output=True,
    )
    if hard_show.returncode != 0:
        fail(f"unable to read base hard-ceiling file from git history: {hard_rel}")
    ratio_show = subprocess.run(
        ["git", "-C", str(repo_root), "show", f"{base_commit}:{ratio_rel}"],
        check=False,
        text=True,
        capture_output=True,
    )
    if ratio_show.returncode != 0:
        fail(f"unable to read base ratio-threshold file from git history: {ratio_rel}")

    hard_env = parse_env_text(
        text=hard_show.stdout,
        source_label=f"baseline:{base_commit}",
        required_keys={"HARD_SHELL_LOC_MAX"},
    )
    ratio_env = parse_env_text(
        text=ratio_show.stdout,
        source_label=f"baseline:{base_commit}",
        required_keys={"WARN_SHELL_RUST_RATIO_MAX", "FAIL_SHELL_RUST_RATIO_MAX"},
    )
    return (
        parse_thresholds(
            hard_env=hard_env,
            ratio_env=ratio_env,
            source_label=f"baseline:{base_commit}",
        ),
        base_commit,
    )


def parse_exception_file(
    *,
    exception_file: Path,
    threshold_violations: list[str],
) -> tuple[bool, str, str]:
    if not exception_file.is_file():
        return False, "ratchet exception file not found", ""
    try:
        payload = json.loads(exception_file.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        fail(f"exception file is not valid JSON: {exc}")
    if not isinstance(payload, dict):
        fail("exception file root must be an object")
    if payload.get("schema_version") != EXCEPTION_SCHEMA_VERSION:
        fail(f"unexpected exception schema_version: {payload.get('schema_version')}")

    reason = payload.get("reason")
    expires_on = payload.get("expires_on")
    mitigation_issue = payload.get("mitigation_issue")
    allow_threshold_keys = payload.get("allow_threshold_keys")
    if not isinstance(reason, str) or not reason.strip():
        fail("exception reason must be a non-empty string")
    if not isinstance(expires_on, str) or not expires_on.strip():
        fail("exception expires_on must be a non-empty YYYY-MM-DD value")
    try:
        expires_date = dt.date.fromisoformat(expires_on)
    except ValueError:
        fail("exception expires_on must be in YYYY-MM-DD format")
    if expires_date < dt.date.today():
        fail(f"exception expired on {expires_on}")
    if not isinstance(mitigation_issue, str) or not ISSUE_REF_PATTERN.fullmatch(
        mitigation_issue
    ):
        fail("exception mitigation_issue must be #<issue-id>")
    if not isinstance(allow_threshold_keys, list) or not allow_threshold_keys:
        fail("exception allow_threshold_keys must be a non-empty string list")
    if not all(isinstance(value, str) for value in allow_threshold_keys):
        fail("exception allow_threshold_keys entries must be strings")
    unknown_keys = sorted(set(allow_threshold_keys).difference(ALLOWED_THRESHOLD_KEYS))
    if unknown_keys:
        fail(
            "exception allow_threshold_keys contains unsupported keys: "
            + ",".join(unknown_keys)
        )
    missing = sorted(set(threshold_violations).difference(set(allow_threshold_keys)))
    if missing:
        return (
            False,
            "exception does not allow ratchet keys: " + ",".join(missing),
            mitigation_issue,
        )
    return True, reason.strip(), mitigation_issue


def write_payload(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def emit(
    *,
    status: str,
    final_decision: str,
    reason_codes: str,
    threshold_ratchet_status: str,
    threshold_violations: list[str],
    threshold_ratchet_mitigation_issue: str,
    review_required: str,
    base_commit: str,
    output_json: Path,
    error: str = "",
    ratchet_exception_reason: str = "",
) -> None:
    print(f"status={status}")
    print(f"final_decision={final_decision}")
    print(f"reason_taxonomy_version={REASON_TAXONOMY_VERSION}")
    print(f"reason_codes={reason_codes}")
    print(f"reason_codes_csv={REASON_CODES_CSV}")
    print(f"threshold_ratchet_status={threshold_ratchet_status}")
    print(
        "threshold_ratchet_violations="
        + ("none" if not threshold_violations else ",".join(threshold_violations))
    )
    print(
        "threshold_ratchet_mitigation_issue="
        + (threshold_ratchet_mitigation_issue or "none")
    )
    print(f"review_required={review_required}")
    print(f"base_commit={base_commit or 'none'}")
    if ratchet_exception_reason:
        print(f"ratchet_exception_reason={ratchet_exception_reason}")
    if error:
        print(f"error={error}")
    print(f"output_json={output_json}")


def main(argv: list[str]) -> int:
    try:
        args = parse_args(argv)
    except SystemExit:
        print("status=fail")
        print("final_decision=NO-GO")
        print(f"reason_taxonomy_version={REASON_TAXONOMY_VERSION}")
        print("reason_codes=shell_surface_threshold_ratchet_argument_invalid")
        print(f"reason_codes_csv={REASON_CODES_CSV}")
        print("threshold_ratchet_status=regressed")
        print("threshold_ratchet_violations=none")
        print("threshold_ratchet_mitigation_issue=none")
        print("review_required=false")
        print("base_commit=none")
        return 1

    output_json = args.output_json.strip() if isinstance(args.output_json, str) else ""
    if not output_json:
        print("status=fail")
        print("final_decision=NO-GO")
        print(f"reason_taxonomy_version={REASON_TAXONOMY_VERSION}")
        print("reason_codes=shell_surface_threshold_ratchet_output_json_required")
        print(f"reason_codes_csv={REASON_CODES_CSV}")
        print("threshold_ratchet_status=regressed")
        print("threshold_ratchet_violations=none")
        print("threshold_ratchet_mitigation_issue=none")
        print("review_required=false")
        print("base_commit=none")
        return 1

    repo_root = Path(args.repo_root).resolve()
    hard_file = resolve_path(repo_root=repo_root, value=args.hard_ceiling_file)
    ratio_file = resolve_path(repo_root=repo_root, value=args.ratio_threshold_file)
    baseline_hard_file = (
        resolve_path(repo_root=repo_root, value=args.baseline_hard_ceiling_file)
        if args.baseline_hard_ceiling_file
        else None
    )
    baseline_ratio_file = (
        resolve_path(repo_root=repo_root, value=args.baseline_ratio_threshold_file)
        if args.baseline_ratio_threshold_file
        else None
    )

    exception_file = resolve_path(repo_root=repo_root, value=args.ratchet_exception_file)
    output_json_path = resolve_path(repo_root=repo_root, value=output_json)

    reason_codes = "none"
    status = "pass"
    final_decision = "GO"
    threshold_ratchet_status = "within"
    threshold_violations: list[str] = []
    threshold_ratchet_mitigation_issue = ""
    review_required = "false"
    ratchet_exception_reason = ""
    error = ""
    base_commit = ""
    current_thresholds: dict[str, float] = {}
    baseline_thresholds: dict[str, float] = {}

    try:
        current_thresholds = load_thresholds_from_files(
            hard_file=hard_file,
            ratio_file=ratio_file,
            source_label="current",
        )

        if baseline_hard_file and baseline_ratio_file:
            baseline_thresholds = load_thresholds_from_files(
                hard_file=baseline_hard_file,
                ratio_file=baseline_ratio_file,
                source_label="baseline-file",
            )
            base_commit = "baseline-files"
        elif baseline_hard_file or baseline_ratio_file:
            fail(
                "both --baseline-hard-ceiling-file and "
                "--baseline-ratio-threshold-file are required together"
            )
        else:
            default_base_ref = (
                args.base_ref.strip()
                if isinstance(args.base_ref, str) and args.base_ref.strip()
                else ""
            ) or os.environ.get("GITHUB_BASE_REF", "").strip() or "main"
            baseline_thresholds, base_commit = load_thresholds_from_git_base(
                repo_root=repo_root,
                base_ref=default_base_ref,
                hard_file=hard_file,
                ratio_file=ratio_file,
            )

        if (
            current_thresholds["HARD_SHELL_LOC_MAX"]
            > baseline_thresholds["HARD_SHELL_LOC_MAX"]
        ):
            threshold_violations.append("HARD_SHELL_LOC_MAX")
        if (
            current_thresholds["WARN_SHELL_RUST_RATIO_MAX"]
            > baseline_thresholds["WARN_SHELL_RUST_RATIO_MAX"]
        ):
            threshold_violations.append("WARN_SHELL_RUST_RATIO_MAX")
        if (
            current_thresholds["FAIL_SHELL_RUST_RATIO_MAX"]
            > baseline_thresholds["FAIL_SHELL_RUST_RATIO_MAX"]
        ):
            threshold_violations.append("FAIL_SHELL_RUST_RATIO_MAX")

        if threshold_violations:
            threshold_ratchet_status = "regressed"
            try:
                exception_applied, ratchet_exception_reason, threshold_ratchet_mitigation_issue = (
                    parse_exception_file(
                        exception_file=exception_file,
                        threshold_violations=threshold_violations,
                    )
                )
            except RuntimeError as exception_error:
                status = "fail"
                final_decision = "NO-GO"
                reason_codes = "shell_surface_threshold_ratchet_exception_file_invalid"
                error = str(exception_error)
            else:
                if exception_applied:
                    status = "pass"
                    final_decision = "GO"
                    reason_codes = "shell_surface_threshold_ratchet_exception_applied"
                    threshold_ratchet_status = "exception-applied"
                    review_required = "true"
                else:
                    status = "fail"
                    final_decision = "NO-GO"
                    reason_codes = "shell_surface_threshold_ratchet_regression_unwaived"
                    error = ratchet_exception_reason
        else:
            status = "pass"
            final_decision = "GO"
            reason_codes = "none"
            threshold_ratchet_status = "within"
            review_required = "false"
    except RuntimeError as checker_error:
        status = "fail"
        final_decision = "NO-GO"
        error = str(checker_error)
        if "merge-base" in error or "base ref" in error or "git history" in error:
            reason_codes = "shell_surface_threshold_ratchet_git_history_unavailable"
        elif "must be <=" in error:
            reason_codes = "shell_surface_threshold_ratchet_threshold_order_invalid"
        elif "must be" in error or "missing required keys" in error:
            reason_codes = "shell_surface_threshold_ratchet_threshold_parse_invalid"
        else:
            reason_codes = "shell_surface_threshold_ratchet_argument_invalid"
        threshold_ratchet_status = "regressed"
        review_required = "false"

    payload: dict[str, Any] = {
        "schema_version": REPORT_SCHEMA_VERSION,
        "status": status,
        "final_decision": final_decision,
        "reason_taxonomy_version": REASON_TAXONOMY_VERSION,
        "reason_codes": reason_codes,
        "metrics": {
            "threshold_ratchet_status": threshold_ratchet_status,
            "threshold_ratchet_violations": threshold_violations,
            "threshold_ratchet_mitigation_issue": threshold_ratchet_mitigation_issue,
            "review_required": review_required == "true",
            "base_commit": base_commit,
            "current": current_thresholds,
            "baseline": baseline_thresholds,
        },
    }
    if ratchet_exception_reason:
        payload["ratchet_exception_reason"] = ratchet_exception_reason
    if error:
        payload["error"] = error

    try:
        write_payload(output_json_path, payload)
    except OSError as write_error:
        emit(
            status="fail",
            final_decision="NO-GO",
            reason_codes="shell_surface_threshold_ratchet_output_write_failed",
            threshold_ratchet_status="regressed",
            threshold_violations=threshold_violations,
            threshold_ratchet_mitigation_issue="",
            review_required="false",
            base_commit=base_commit,
            output_json=output_json_path,
            error=str(write_error),
        )
        return 1

    emit(
        status=status,
        final_decision=final_decision,
        reason_codes=reason_codes,
        threshold_ratchet_status=threshold_ratchet_status,
        threshold_violations=threshold_violations,
        threshold_ratchet_mitigation_issue=threshold_ratchet_mitigation_issue,
        review_required=review_required,
        base_commit=base_commit,
        output_json=output_json_path,
        error=error,
        ratchet_exception_reason=ratchet_exception_reason,
    )
    return 0 if status == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
