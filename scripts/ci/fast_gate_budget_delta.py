#!/usr/bin/env python3
"""Generate and validate fast-gate runtime/cost delta reports."""

from __future__ import annotations

import argparse
from dataclasses import dataclass
from datetime import date, datetime, timezone
import json
from pathlib import Path
import re
import sys


SCHEMA_VERSION = "kamn.ci.fast-gate-budget-delta-report.v1"
ISSUE_REF_PATTERN = re.compile(r"^#[0-9]+$")
RATCHET_THRESHOLD_KEYS = {
    "FAST_GATE_DELTA_MAX_ELAPSED_DELTA_PCT",
    "FAST_GATE_DELTA_MAX_RUNNER_MINUTES_DELTA_PCT",
}


@dataclass(frozen=True)
class DeltaConfig:
    baseline_elapsed_seconds: int
    baseline_runner_minutes: int
    max_elapsed_delta_pct: float
    max_runner_delta_pct: float
    threshold_refreshed_on: date
    threshold_max_age_days: int


def fail(message: str) -> None:
    raise SystemExit(message)


def bool_marker(value: bool) -> str:
    return "true" if value else "false"


def load_json(path: Path) -> dict:
    if not path.is_file():
        fail(f"file not found: {path}")
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        fail(f"invalid JSON in {path}: {exc}")
    if not isinstance(payload, dict):
        fail(f"JSON object expected in {path}")
    return payload


def load_env(path: Path) -> dict[str, str]:
    if not path.is_file():
        fail(f"config file not found: {path}")
    values: dict[str, str] = {}
    for line_number, raw_line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        if "=" not in line:
            fail(f"invalid config line at {path}:{line_number}")
        key, raw_value = line.split("=", 1)
        key = key.strip()
        value = raw_value.strip()
        if not key:
            fail(f"empty config key at {path}:{line_number}")
        values[key] = value
    return values


def require_int(values: dict[str, str], key: str) -> int:
    raw = values.get(key)
    if raw is None or not raw.isdigit():
        fail(f"{key} must be a non-negative integer")
    value = int(raw)
    if value <= 0:
        fail(f"{key} must be greater than zero")
    return value


def require_float(values: dict[str, str], key: str) -> float:
    raw = values.get(key)
    if raw is None:
        fail(f"{key} is required")
    try:
        value = float(raw)
    except ValueError:
        fail(f"{key} must be a numeric value")
    if value <= 0:
        fail(f"{key} must be greater than zero")
    return value


def require_date(values: dict[str, str], key: str) -> date:
    raw = values.get(key)
    if raw is None:
        fail(f"{key} is required")
    try:
        value = date.fromisoformat(raw)
    except ValueError:
        fail(f"{key} must be in YYYY-MM-DD format")
    return value


def validate_threshold_freshness(refreshed_on: date, max_age_days: int) -> None:
    today = date.today()
    if refreshed_on > today:
        fail("FAST_GATE_DELTA_THRESHOLD_REFRESHED_ON cannot be in the future")
    age_days = (today - refreshed_on).days
    if age_days > max_age_days:
        fail(
            "threshold file stale: "
            f"FAST_GATE_DELTA_THRESHOLD_REFRESHED_ON={refreshed_on.isoformat()} "
            f"age_days={age_days} max_age_days={max_age_days}"
        )


def load_delta_config(path: Path) -> DeltaConfig:
    values = load_env(path)
    config = DeltaConfig(
        baseline_elapsed_seconds=require_int(values, "FAST_GATE_DELTA_BASELINE_ELAPSED_SECONDS"),
        baseline_runner_minutes=require_int(values, "FAST_GATE_DELTA_BASELINE_RUNNER_MINUTES"),
        max_elapsed_delta_pct=require_float(values, "FAST_GATE_DELTA_MAX_ELAPSED_DELTA_PCT"),
        max_runner_delta_pct=require_float(values, "FAST_GATE_DELTA_MAX_RUNNER_MINUTES_DELTA_PCT"),
        threshold_refreshed_on=require_date(values, "FAST_GATE_DELTA_THRESHOLD_REFRESHED_ON"),
        threshold_max_age_days=require_int(values, "FAST_GATE_DELTA_THRESHOLD_MAX_AGE_DAYS"),
    )
    validate_threshold_freshness(
        refreshed_on=config.threshold_refreshed_on,
        max_age_days=config.threshold_max_age_days,
    )
    return config


def require_payload_int(payload: dict, key: str) -> int:
    value = payload.get(key)
    if not isinstance(value, int):
        fail(f"{key} must be an integer")
    if value < 0:
        fail(f"{key} must be a non-negative integer")
    return value


def compute_delta(current: int, baseline: int) -> tuple[int, float]:
    delta = current - baseline
    delta_pct = round((delta / baseline) * 100.0, 2)
    return delta, delta_pct


def classify_local_heavy_scope(test_scope: str) -> tuple[bool, str]:
    if test_scope.startswith("kolme-local-heavy-contract"):
        return True, "contract"
    if test_scope.startswith("kolme-local-heavy-local-only"):
        return True, "local-only"
    if test_scope.startswith("kolme-local-heavy-"):
        return True, "other"
    return False, "none"


def command_generate(args: argparse.Namespace) -> int:
    current_path = Path(args.current_json)
    output_path = Path(args.output_json)
    baseline_path = Path(args.baseline_file)
    lane = args.lane

    current_payload = load_json(current_path)
    current_lane = current_payload.get("lane")
    if current_lane != lane:
        fail(f"current budget lane mismatch: expected {lane}, found {current_lane}")

    current_elapsed = require_payload_int(current_payload, "elapsed_seconds")
    current_runner_minutes = require_payload_int(current_payload, "runner_minutes")
    current_test_scope = current_payload.get("test_scope", "unknown")
    if not isinstance(current_test_scope, str):
        fail("current budget test_scope must be a string")
    local_heavy_sensitive, local_heavy_scope_class = classify_local_heavy_scope(current_test_scope)

    config = load_delta_config(baseline_path)
    elapsed_delta, elapsed_delta_pct = compute_delta(
        current=current_elapsed,
        baseline=config.baseline_elapsed_seconds,
    )
    runner_delta, runner_delta_pct = compute_delta(
        current=current_runner_minutes,
        baseline=config.baseline_runner_minutes,
    )
    positive_drift_detected = elapsed_delta > 0 or runner_delta > 0
    local_heavy_sensitive_drift_detected = local_heavy_sensitive and positive_drift_detected

    report_payload = {
        "schema_version": SCHEMA_VERSION,
        "generated_at_utc": datetime.now(timezone.utc).isoformat(timespec="seconds"),
        "lane": lane,
        "baseline_source": str(baseline_path),
        "current_budget_source": str(current_path),
        "current_budget_status": current_payload.get("status", "unknown"),
        "test_scope": current_test_scope,
        "local_heavy_sensitive": local_heavy_sensitive,
        "local_heavy_scope_class": local_heavy_scope_class,
        "positive_drift_detected": positive_drift_detected,
        "local_heavy_sensitive_drift_detected": local_heavy_sensitive_drift_detected,
        "baseline": {
            "elapsed_seconds": config.baseline_elapsed_seconds,
            "runner_minutes": config.baseline_runner_minutes,
        },
        "current": {
            "elapsed_seconds": current_elapsed,
            "runner_minutes": current_runner_minutes,
        },
        "variance": {
            "elapsed_seconds_delta": elapsed_delta,
            "elapsed_seconds_delta_pct": elapsed_delta_pct,
            "runner_minutes_delta": runner_delta,
            "runner_minutes_delta_pct": runner_delta_pct,
        },
        "thresholds": {
            "max_elapsed_delta_pct": config.max_elapsed_delta_pct,
            "max_runner_minutes_delta_pct": config.max_runner_delta_pct,
            "threshold_refreshed_on": config.threshold_refreshed_on.isoformat(),
            "threshold_max_age_days": config.threshold_max_age_days,
        },
    }

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(report_payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    print("status=generated")
    print(f"lane={lane}")
    print(f"elapsed_seconds_delta={elapsed_delta}")
    print(f"elapsed_seconds_delta_pct={elapsed_delta_pct}")
    print(f"runner_minutes_delta={runner_delta}")
    print(f"runner_minutes_delta_pct={runner_delta_pct}")
    print(f"local_heavy_sensitive={bool_marker(local_heavy_sensitive)}")
    print(f"local_heavy_scope_class={local_heavy_scope_class}")
    print(
        "local_heavy_sensitive_drift_detected="
        f"{bool_marker(local_heavy_sensitive_drift_detected)}"
    )
    return 0


def parse_waiver(waiver_path: Path, violations: list[str]) -> tuple[bool, str]:
    if not waiver_path.is_file():
        return False, "waiver file not found"

    payload = load_json(waiver_path)
    reason = payload.get("reason")
    expires_on = payload.get("expires_on")
    allow_metrics = payload.get("allow_metrics")

    if not isinstance(reason, str) or not reason.strip():
        fail("waiver reason must be a non-empty string")
    if not isinstance(expires_on, str) or not expires_on:
        fail("waiver expires_on must be a non-empty YYYY-MM-DD value")
    if not isinstance(allow_metrics, list) or not all(isinstance(metric, str) for metric in allow_metrics):
        fail("waiver allow_metrics must be a string list")
    if not allow_metrics:
        fail("waiver allow_metrics must not be empty")

    try:
        expires_date = date.fromisoformat(expires_on)
    except ValueError:
        fail("waiver expires_on must be in YYYY-MM-DD format")

    if expires_date < date.today():
        fail(f"waiver expired on {expires_on}")

    missing_metrics = sorted(set(violations).difference(allow_metrics))
    if missing_metrics:
        return False, f"waiver does not allow metrics: {','.join(missing_metrics)}"

    return True, reason.strip()


def parse_ratchet_exception(
    ratchet_exception_path: Path,
    ratchet_violations: list[str],
) -> tuple[bool, str, str]:
    if not ratchet_exception_path.is_file():
        return False, "ratchet exception file not found", ""

    payload = load_json(ratchet_exception_path)
    reason = payload.get("reason")
    expires_on = payload.get("expires_on")
    mitigation_issue = payload.get("mitigation_issue")
    allow_threshold_keys = payload.get("allow_threshold_keys")

    if not isinstance(reason, str) or not reason.strip():
        fail("ratchet exception reason must be a non-empty string")
    if not isinstance(expires_on, str) or not expires_on:
        fail("ratchet exception expires_on must be a non-empty YYYY-MM-DD value")
    try:
        expires_date = date.fromisoformat(expires_on)
    except ValueError:
        fail("ratchet exception expires_on must be in YYYY-MM-DD format")
    if expires_date < date.today():
        fail(f"ratchet exception expired on {expires_on}")

    if not isinstance(mitigation_issue, str) or not ISSUE_REF_PATTERN.fullmatch(mitigation_issue):
        fail("ratchet mitigation_issue must be #<issue-id>")

    if not isinstance(allow_threshold_keys, list) or not all(
        isinstance(key, str) for key in allow_threshold_keys
    ):
        fail("ratchet allow_threshold_keys must be a string list")
    if not allow_threshold_keys:
        fail("ratchet allow_threshold_keys must not be empty")
    unknown_keys = sorted(set(allow_threshold_keys).difference(RATCHET_THRESHOLD_KEYS))
    if unknown_keys:
        fail(
            "ratchet allow_threshold_keys contains unsupported keys: "
            + ",".join(unknown_keys)
        )

    missing_keys = sorted(set(ratchet_violations).difference(allow_threshold_keys))
    if missing_keys:
        return (
            False,
            "ratchet exception does not allow thresholds: " + ",".join(missing_keys),
            mitigation_issue,
        )

    return True, reason.strip(), mitigation_issue


def extract_section(payload: dict, key: str) -> dict:
    section = payload.get(key)
    if not isinstance(section, dict):
        fail(f"{key} must be an object")
    return section


def extract_float(section: dict, key: str) -> float:
    value = section.get(key)
    if not isinstance(value, (int, float)):
        fail(f"{key} must be numeric")
    return float(value)


def extract_string(payload: dict, key: str) -> str:
    value = payload.get(key)
    if not isinstance(value, str):
        fail(f"{key} must be a string")
    return value


def extract_bool(payload: dict, key: str) -> bool:
    value = payload.get(key)
    if not isinstance(value, bool):
        fail(f"{key} must be boolean")
    return value


def emit_check_markers(
    *,
    status: str,
    waived: bool,
    violations: list[str],
    review_required: bool,
    soft_overrun_status: str,
    reason_codes: list[str],
    local_heavy_sensitive: bool,
    local_heavy_sensitive_drift_detected: bool,
    threshold_ratchet_status: str,
    threshold_ratchet_violations: list[str],
    failure_reason: str = "",
    waiver_reason: str = "",
    ratchet_exception_reason: str = "",
    ratchet_mitigation_issue: str = "",
) -> None:
    print(f"status={status}")
    print(f"waived={bool_marker(waived)}")
    print(f"violations={'none' if not violations else ','.join(violations)}")
    print(f"review_required={bool_marker(review_required)}")
    print(f"soft_overrun_status={soft_overrun_status}")
    print(f"reason_codes={'none' if not reason_codes else ','.join(reason_codes)}")
    print(f"local_heavy_sensitive={bool_marker(local_heavy_sensitive)}")
    print(
        "local_heavy_sensitive_drift_detected="
        f"{bool_marker(local_heavy_sensitive_drift_detected)}"
    )
    print(f"threshold_ratchet_status={threshold_ratchet_status}")
    print(
        "threshold_ratchet_violations="
        f"{'none' if not threshold_ratchet_violations else ','.join(threshold_ratchet_violations)}"
    )
    if ratchet_mitigation_issue:
        print(f"threshold_ratchet_mitigation_issue={ratchet_mitigation_issue}")
    if waiver_reason:
        print(f"waiver_reason={waiver_reason}")
    if ratchet_exception_reason:
        print(f"ratchet_exception_reason={ratchet_exception_reason}")
    if failure_reason:
        print(f"failure_reason={failure_reason}")


def command_check(args: argparse.Namespace) -> int:
    report_path = Path(args.report_json)
    threshold_path = Path(args.threshold_file)
    waiver_path = Path(args.waiver_file)
    ratchet_baseline_path = Path(args.ratchet_baseline_file)
    ratchet_exception_path = Path(args.ratchet_exception_file)

    report = load_json(report_path)
    if report.get("schema_version") != SCHEMA_VERSION:
        fail("unexpected schema_version in fast-gate delta report")
    if report.get("lane") != "fast-gate":
        fail("delta report lane must be fast-gate")

    _test_scope = extract_string(report, "test_scope")
    local_heavy_sensitive = extract_bool(report, "local_heavy_sensitive")
    local_heavy_scope_class = extract_string(report, "local_heavy_scope_class")
    local_heavy_sensitive_drift_detected = extract_bool(
        report, "local_heavy_sensitive_drift_detected"
    )

    if local_heavy_scope_class not in {"none", "contract", "local-only", "other"}:
        fail("local_heavy_scope_class must be one of: none, contract, local-only, other")
    if local_heavy_sensitive_drift_detected and not local_heavy_sensitive:
        fail(
            "local_heavy_sensitive_drift_detected requires local_heavy_sensitive=true"
        )

    variance = extract_section(report, "variance")
    elapsed_delta = extract_float(variance, "elapsed_seconds_delta")
    elapsed_delta_pct = extract_float(variance, "elapsed_seconds_delta_pct")
    runner_delta = extract_float(variance, "runner_minutes_delta")
    runner_delta_pct = extract_float(variance, "runner_minutes_delta_pct")

    config = load_delta_config(threshold_path)
    ratchet_baseline = load_delta_config(ratchet_baseline_path)
    ratchet_violations: list[str] = []
    if config.max_elapsed_delta_pct > ratchet_baseline.max_elapsed_delta_pct:
        ratchet_violations.append("FAST_GATE_DELTA_MAX_ELAPSED_DELTA_PCT")
    if config.max_runner_delta_pct > ratchet_baseline.max_runner_delta_pct:
        ratchet_violations.append("FAST_GATE_DELTA_MAX_RUNNER_MINUTES_DELTA_PCT")

    ratchet_review_reason_codes: list[str] = []
    ratchet_mitigation_issue = ""
    ratchet_exception_reason = ""
    threshold_ratchet_status = "within"
    if ratchet_violations:
        threshold_ratchet_status = "regressed"
        ratchet_exception_applied, ratchet_exception_reason, ratchet_mitigation_issue = (
            parse_ratchet_exception(
                ratchet_exception_path=ratchet_exception_path,
                ratchet_violations=ratchet_violations,
            )
        )
        if not ratchet_exception_applied:
            emit_check_markers(
                status="fail",
                waived=False,
                violations=[],
                review_required=False,
                soft_overrun_status="within",
                reason_codes=["fast_gate_delta_threshold_ratchet_regression_unwaived"],
                local_heavy_sensitive=local_heavy_sensitive,
                local_heavy_sensitive_drift_detected=local_heavy_sensitive_drift_detected,
                threshold_ratchet_status=threshold_ratchet_status,
                threshold_ratchet_violations=ratchet_violations,
                ratchet_mitigation_issue=ratchet_mitigation_issue,
                failure_reason=ratchet_exception_reason,
            )
            return 1

        threshold_ratchet_status = "exception-applied"
        ratchet_review_reason_codes.append("fast_gate_delta_threshold_ratchet_exception_applied")
    violations: list[str] = []
    if elapsed_delta > 0 and elapsed_delta_pct > config.max_elapsed_delta_pct:
        violations.append("elapsed_seconds_delta_pct")
    if runner_delta > 0 and runner_delta_pct > config.max_runner_delta_pct:
        violations.append("runner_minutes_delta_pct")

    review_reason_codes: list[str] = ratchet_review_reason_codes.copy()
    if local_heavy_sensitive_drift_detected:
        review_reason_codes.append("local_heavy_sensitive_drift_detected")

    if not violations:
        emit_check_markers(
            status="pass",
            waived=False,
            violations=[],
            review_required=bool(review_reason_codes),
            soft_overrun_status="exceeded" if review_reason_codes else "within",
            reason_codes=review_reason_codes,
            local_heavy_sensitive=local_heavy_sensitive,
            local_heavy_sensitive_drift_detected=local_heavy_sensitive_drift_detected,
            threshold_ratchet_status=threshold_ratchet_status,
            threshold_ratchet_violations=ratchet_violations,
            ratchet_mitigation_issue=ratchet_mitigation_issue,
            ratchet_exception_reason=ratchet_exception_reason,
        )
        return 0

    waiver_applied, waiver_reason = parse_waiver(waiver_path, violations)
    if waiver_applied:
        emit_check_markers(
            status="pass",
            waived=True,
            violations=violations,
            review_required=True,
            soft_overrun_status="exceeded",
            reason_codes=["delta_threshold_waiver_applied"] + review_reason_codes,
            local_heavy_sensitive=local_heavy_sensitive,
            local_heavy_sensitive_drift_detected=local_heavy_sensitive_drift_detected,
            threshold_ratchet_status=threshold_ratchet_status,
            threshold_ratchet_violations=ratchet_violations,
            waiver_reason=waiver_reason,
            ratchet_mitigation_issue=ratchet_mitigation_issue,
            ratchet_exception_reason=ratchet_exception_reason,
        )
        return 0

    emit_check_markers(
        status="fail",
        waived=False,
        violations=violations,
        review_required=False,
        soft_overrun_status="within",
        reason_codes=["delta_threshold_violation_unwaived"],
        local_heavy_sensitive=local_heavy_sensitive,
        local_heavy_sensitive_drift_detected=local_heavy_sensitive_drift_detected,
        threshold_ratchet_status=threshold_ratchet_status,
        threshold_ratchet_violations=ratchet_violations,
        ratchet_mitigation_issue=ratchet_mitigation_issue,
        ratchet_exception_reason=ratchet_exception_reason,
        failure_reason=waiver_reason,
    )
    return 1


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Fast-gate budget delta tooling.")
    subparsers = parser.add_subparsers(dest="command", required=True)

    generate = subparsers.add_parser("generate", help="Generate a fast-gate delta report.")
    generate.add_argument("--current-json", required=True)
    generate.add_argument("--output-json", required=True)
    generate.add_argument("--baseline-file", default=".ci/fast-gate-budget-delta.env")
    generate.add_argument("--lane", default="fast-gate")
    generate.set_defaults(handler=command_generate)

    check = subparsers.add_parser("check", help="Validate a fast-gate delta report against thresholds.")
    check.add_argument("--report-json", required=True)
    check.add_argument("--threshold-file", default=".ci/fast-gate-budget-delta.env")
    check.add_argument("--waiver-file", default=".ci/fast-gate-budget-delta-waiver.json")
    check.add_argument(
        "--ratchet-baseline-file",
        default=".ci/fast-gate-budget-delta-ratchet.env",
    )
    check.add_argument(
        "--ratchet-exception-file",
        default=".ci/fast-gate-budget-delta-ratchet-exception.json",
    )
    check.set_defaults(handler=command_check)

    return parser


def main(argv: list[str]) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    return args.handler(args)


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except SystemExit as exc:
        if isinstance(exc.code, str):
            print(exc.code, file=sys.stderr)
            raise SystemExit(1)
        raise
