#!/usr/bin/env python3
"""Post-cutover SLO evidence generator and policy checker."""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
from pathlib import Path
import sys
from typing import Any, Mapping

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import (  # noqa: E402
    ContractError,
    fail,
    load_json,
    require_int,
    require_keys,
    require_non_negative_int,
    require_object,
    write_json,
)

SCHEMA_VERSION = "kamn.launch-slo.evidence.v1"
GO_DECISION = "GO"
NO_GO_DECISION = "NO-GO"

REASON_TO_ALERT_KEY = {
    "p95-latency-threshold-exceeded": "slo.latency.p95.threshold_exceeded",
    "error-rate-threshold-exceeded": "slo.error_rate.threshold_exceeded",
    "delivery-success-threshold-breached": "slo.delivery_success.threshold_breached",
    "stale-snapshot-evidence": "slo.snapshot_age.stale",
    "incomplete-slo-evidence": "slo.evidence.incomplete",
    "ci-fast-gate-failed": "slo.ci_fast_gate.failed",
}

REASON_TO_SEVERITY = {
    "p95-latency-threshold-exceeded": "CRITICAL",
    "error-rate-threshold-exceeded": "CRITICAL",
    "delivery-success-threshold-breached": "CRITICAL",
    "stale-snapshot-evidence": "CRITICAL",
    "incomplete-slo-evidence": "WARNING",
    "ci-fast-gate-failed": "WARNING",
}


def _parse_bool(field_name: str, raw_value: str) -> bool:
    if raw_value == "true":
        return True
    if raw_value == "false":
        return False
    fail(f"{field_name} must be true or false")


def _parse_ci_fast_gate(raw_value: str, *, error_message: str) -> str:
    if raw_value in {"PASS", "FAIL"}:
        return raw_value
    fail(error_message)


def _compute_decision_reasons(
    *,
    p95_latency_ms: int,
    max_p95_latency_ms: int,
    error_rate_bps: int,
    max_error_rate_bps: int,
    delivery_success_bps: int,
    min_delivery_success_bps: int,
    snapshot_age_seconds: int,
    max_snapshot_age_seconds: int,
    evidence_complete: bool,
    ci_fast_gate: str,
) -> list[str]:
    reasons: list[str] = []
    if p95_latency_ms > max_p95_latency_ms:
        reasons.append("p95-latency-threshold-exceeded")
    if error_rate_bps > max_error_rate_bps:
        reasons.append("error-rate-threshold-exceeded")
    if delivery_success_bps < min_delivery_success_bps:
        reasons.append("delivery-success-threshold-breached")
    if snapshot_age_seconds > max_snapshot_age_seconds:
        reasons.append("stale-snapshot-evidence")
    if not evidence_complete:
        reasons.append("incomplete-slo-evidence")
    if ci_fast_gate != "PASS":
        reasons.append("ci-fast-gate-failed")
    return reasons


def _build_alert_summary(decision_reasons: list[str]) -> Mapping[str, Any]:
    alert_keys = [REASON_TO_ALERT_KEY[reason] for reason in decision_reasons]
    critical_alerts = sum(
        1 for reason in decision_reasons if REASON_TO_SEVERITY[reason] == "CRITICAL"
    )
    warning_alerts = sum(
        1 for reason in decision_reasons if REASON_TO_SEVERITY[reason] == "WARNING"
    )

    highest_severity = "NONE"
    if critical_alerts > 0:
        highest_severity = "CRITICAL"
    elif warning_alerts > 0:
        highest_severity = "WARNING"

    return {
        "total_alerts": len(alert_keys),
        "critical_alerts": critical_alerts,
        "warning_alerts": warning_alerts,
        "has_alerts": len(alert_keys) > 0,
        "highest_severity": highest_severity,
        "alert_keys": alert_keys,
    }


def generate_bundle(args: argparse.Namespace) -> int:
    window_minutes = require_non_negative_int("window_minutes", str(args.window_minutes))
    p95_latency_ms = require_non_negative_int("p95_latency_ms", str(args.p95_latency_ms))
    max_p95_latency_ms = require_non_negative_int(
        "max_p95_latency_ms", str(args.max_p95_latency_ms)
    )
    error_rate_bps = require_non_negative_int("error_rate_bps", str(args.error_rate_bps))
    max_error_rate_bps = require_non_negative_int(
        "max_error_rate_bps", str(args.max_error_rate_bps)
    )
    delivery_success_bps = require_non_negative_int(
        "delivery_success_bps", str(args.delivery_success_bps)
    )
    min_delivery_success_bps = require_non_negative_int(
        "min_delivery_success_bps", str(args.min_delivery_success_bps)
    )
    snapshot_age_seconds = require_non_negative_int(
        "snapshot_age_seconds", str(args.snapshot_age_seconds)
    )
    max_snapshot_age_seconds = require_non_negative_int(
        "max_snapshot_age_seconds", str(args.max_snapshot_age_seconds)
    )
    evidence_complete = _parse_bool("evidence_complete", args.evidence_complete)
    ci_fast_gate = _parse_ci_fast_gate(
        args.ci_fast_gate,
        error_message="--ci-fast-gate must be PASS or FAIL",
    )

    decision_reasons = _compute_decision_reasons(
        p95_latency_ms=p95_latency_ms,
        max_p95_latency_ms=max_p95_latency_ms,
        error_rate_bps=error_rate_bps,
        max_error_rate_bps=max_error_rate_bps,
        delivery_success_bps=delivery_success_bps,
        min_delivery_success_bps=min_delivery_success_bps,
        snapshot_age_seconds=snapshot_age_seconds,
        max_snapshot_age_seconds=max_snapshot_age_seconds,
        evidence_complete=evidence_complete,
        ci_fast_gate=ci_fast_gate,
    )
    final_decision = GO_DECISION if not decision_reasons else NO_GO_DECISION
    reason_key = f"slo_alert_reason_codes:{final_decision}:v1"
    generated_at = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")

    payload = {
        "schema_version": SCHEMA_VERSION,
        "reason_key": reason_key,
        "generated_at": generated_at,
        "window_minutes": window_minutes,
        "metrics": {
            "p95_latency_ms": p95_latency_ms,
            "max_p95_latency_ms": max_p95_latency_ms,
            "error_rate_bps": error_rate_bps,
            "max_error_rate_bps": max_error_rate_bps,
            "delivery_success_bps": delivery_success_bps,
            "min_delivery_success_bps": min_delivery_success_bps,
            "snapshot_age_seconds": snapshot_age_seconds,
            "max_snapshot_age_seconds": max_snapshot_age_seconds,
            "evidence_complete": evidence_complete,
            "ci_fast_gate": ci_fast_gate,
        },
        "decision_reasons": decision_reasons,
        "alerts": _build_alert_summary(decision_reasons),
        "final_decision": final_decision,
    }

    output_path = Path(args.output_file)
    write_json(output_path, payload)

    print("status=generated")
    print(f"bundle_file={output_path}")
    print(f"final_decision={final_decision}")
    print(f"reason_key={reason_key}")
    print(f"snapshot_age_seconds={snapshot_age_seconds}")
    print(f"max_snapshot_age_seconds={max_snapshot_age_seconds}")
    return 0


def _require_metrics_int(metrics: Mapping[str, Any], field_name: str) -> int:
    return require_int(metrics, field_name)


def _require_decision_reasons(payload: Mapping[str, Any]) -> list[str]:
    decision_reasons = payload.get("decision_reasons")
    if not isinstance(decision_reasons, list) or any(
        not isinstance(item, str) for item in decision_reasons
    ):
        fail("decision_reasons must be an array of strings")
    return decision_reasons


def check_bundle(args: argparse.Namespace) -> int:
    bundle_path = Path(args.bundle_file)
    if not bundle_path.is_file():
        fail(f"bundle file not found: {bundle_path}")

    payload = load_json(bundle_path)
    require_keys(
        payload,
        (
            "schema_version",
            "reason_key",
            "generated_at",
            "window_minutes",
            "metrics",
            "decision_reasons",
            "alerts",
            "final_decision",
        ),
    )

    schema_version = payload.get("schema_version")
    if schema_version != SCHEMA_VERSION:
        fail("unexpected post-cutover SLO evidence schema_version")

    window_minutes = payload.get("window_minutes")
    if not isinstance(window_minutes, int) or window_minutes < 1:
        fail("window_minutes must be an integer >= 1")

    metrics = require_object(payload, "metrics")
    required_metric_fields = (
        "p95_latency_ms",
        "max_p95_latency_ms",
        "error_rate_bps",
        "max_error_rate_bps",
        "delivery_success_bps",
        "min_delivery_success_bps",
        "snapshot_age_seconds",
        "max_snapshot_age_seconds",
        "evidence_complete",
        "ci_fast_gate",
    )
    for metric_field in required_metric_fields:
        if metric_field not in metrics:
            fail(f"missing metrics field: {metric_field}")

    p95_latency_ms = _require_metrics_int(metrics, "p95_latency_ms")
    max_p95_latency_ms = _require_metrics_int(metrics, "max_p95_latency_ms")
    error_rate_bps = _require_metrics_int(metrics, "error_rate_bps")
    max_error_rate_bps = _require_metrics_int(metrics, "max_error_rate_bps")
    delivery_success_bps = _require_metrics_int(metrics, "delivery_success_bps")
    min_delivery_success_bps = _require_metrics_int(metrics, "min_delivery_success_bps")
    snapshot_age_seconds = _require_metrics_int(metrics, "snapshot_age_seconds")
    max_snapshot_age_seconds = _require_metrics_int(
        metrics, "max_snapshot_age_seconds"
    )

    if max_p95_latency_ms < 1:
        fail("metrics.max_p95_latency_ms must be >= 1")
    if max_error_rate_bps < 1:
        fail("metrics.max_error_rate_bps must be >= 1")
    if min_delivery_success_bps < 1:
        fail("metrics.min_delivery_success_bps must be >= 1")
    if max_snapshot_age_seconds < 1:
        fail("metrics.max_snapshot_age_seconds must be >= 1")

    evidence_complete = metrics.get("evidence_complete")
    if not isinstance(evidence_complete, bool):
        fail("metrics.evidence_complete must be a boolean")

    ci_fast_gate = metrics.get("ci_fast_gate")
    if ci_fast_gate not in {"PASS", "FAIL"}:
        fail("metrics.ci_fast_gate must be PASS or FAIL")

    expected_decision_reasons = _compute_decision_reasons(
        p95_latency_ms=p95_latency_ms,
        max_p95_latency_ms=max_p95_latency_ms,
        error_rate_bps=error_rate_bps,
        max_error_rate_bps=max_error_rate_bps,
        delivery_success_bps=delivery_success_bps,
        min_delivery_success_bps=min_delivery_success_bps,
        snapshot_age_seconds=snapshot_age_seconds,
        max_snapshot_age_seconds=max_snapshot_age_seconds,
        evidence_complete=evidence_complete,
        ci_fast_gate=ci_fast_gate,
    )

    actual_decision_reasons = _require_decision_reasons(payload)
    if actual_decision_reasons != expected_decision_reasons:
        fail(
            "decision_reasons mismatch: "
            f"expected {expected_decision_reasons}, found {actual_decision_reasons}"
        )

    expected_decision = GO_DECISION if not expected_decision_reasons else NO_GO_DECISION
    actual_decision = payload.get("final_decision")
    if actual_decision not in {GO_DECISION, NO_GO_DECISION}:
        fail("final_decision must be GO or NO-GO")
    if actual_decision != expected_decision:
        reasons = ", ".join(expected_decision_reasons) or "all SLO gates satisfied"
        fail(
            "policy decision mismatch: "
            f"expected final_decision={expected_decision}, found {actual_decision}; "
            f"reasons={reasons}"
        )

    expected_reason_key = f"slo_alert_reason_codes:{expected_decision}:v1"
    actual_reason_key = payload.get("reason_key")
    if actual_reason_key != expected_reason_key:
        fail(
            "reason_key mismatch: "
            f"expected {expected_reason_key}, found {actual_reason_key}"
        )

    alerts = payload.get("alerts")
    if not isinstance(alerts, dict):
        fail("bundle field 'alerts' must be an object")
    for alert_field in (
        "total_alerts",
        "critical_alerts",
        "warning_alerts",
        "has_alerts",
        "highest_severity",
        "alert_keys",
    ):
        if alert_field not in alerts:
            fail(f"missing alerts field: {alert_field}")

    for alert_int_field in ("total_alerts", "critical_alerts", "warning_alerts"):
        if not isinstance(alerts[alert_int_field], int):
            fail(f"alerts.{alert_int_field} must be an integer")

    if not isinstance(alerts["has_alerts"], bool):
        fail("alerts.has_alerts must be a boolean")
    if alerts["highest_severity"] not in {"NONE", "WARNING", "CRITICAL"}:
        fail("alerts.highest_severity must be NONE, WARNING, or CRITICAL")

    alert_keys = alerts.get("alert_keys")
    if not isinstance(alert_keys, list) or any(
        not isinstance(item, str) for item in alert_keys
    ):
        fail("alerts.alert_keys must be an array of strings")

    expected_alert_keys = [
        REASON_TO_ALERT_KEY[reason] for reason in expected_decision_reasons
    ]
    if alert_keys != expected_alert_keys:
        fail(
            "alerts.alert_keys mismatch: "
            f"expected {expected_alert_keys}, found {alert_keys}"
        )

    expected_critical = sum(
        1
        for reason in expected_decision_reasons
        if REASON_TO_SEVERITY[reason] == "CRITICAL"
    )
    expected_warning = sum(
        1
        for reason in expected_decision_reasons
        if REASON_TO_SEVERITY[reason] == "WARNING"
    )
    expected_total = len(expected_alert_keys)
    expected_has_alerts = expected_total > 0
    expected_highest_severity = "NONE"
    if expected_critical > 0:
        expected_highest_severity = "CRITICAL"
    elif expected_warning > 0:
        expected_highest_severity = "WARNING"

    if alerts["critical_alerts"] != expected_critical:
        fail(
            "alerts.critical_alerts mismatch: "
            f"expected {expected_critical}, found {alerts['critical_alerts']}"
        )
    if alerts["warning_alerts"] != expected_warning:
        fail(
            "alerts.warning_alerts mismatch: "
            f"expected {expected_warning}, found {alerts['warning_alerts']}"
        )
    if alerts["total_alerts"] != expected_total:
        fail(
            "alerts.total_alerts mismatch: "
            f"expected {expected_total}, found {alerts['total_alerts']}"
        )
    if alerts["has_alerts"] != expected_has_alerts:
        fail(
            "alerts.has_alerts mismatch: "
            f"expected {expected_has_alerts}, found {alerts['has_alerts']}"
        )
    if alerts["highest_severity"] != expected_highest_severity:
        fail(
            "alerts.highest_severity mismatch: "
            f"expected {expected_highest_severity}, found {alerts['highest_severity']}"
        )

    print("status=ok")
    print(f"bundle_file={bundle_path}")
    print(f"final_decision={actual_decision}")
    print(f"reason_key={actual_reason_key}")
    print(f"snapshot_age_seconds={snapshot_age_seconds}")
    print(f"max_snapshot_age_seconds={max_snapshot_age_seconds}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description=(
            "Post-cutover SLO evidence contract utilities "
            "(generate/check)."
        )
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    generate = subparsers.add_parser("generate")
    generate.add_argument("--output-file", required=True)
    generate.add_argument("--window-minutes", required=True)
    generate.add_argument("--p95-latency-ms", required=True)
    generate.add_argument("--max-p95-latency-ms", required=True)
    generate.add_argument("--error-rate-bps", required=True)
    generate.add_argument("--max-error-rate-bps", required=True)
    generate.add_argument("--delivery-success-bps", required=True)
    generate.add_argument("--min-delivery-success-bps", required=True)
    generate.add_argument("--snapshot-age-seconds", required=True)
    generate.add_argument("--max-snapshot-age-seconds", required=True)
    generate.add_argument("--evidence-complete", required=True)
    generate.add_argument("--ci-fast-gate", required=True)
    generate.set_defaults(handler=generate_bundle)

    check = subparsers.add_parser("check")
    check.add_argument("--bundle-file", required=True)
    check.set_defaults(handler=check_bundle)

    return parser


def main(argv: list[str]) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    return args.handler(args)


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except ContractError as error:
        print(error, file=sys.stderr)
        raise SystemExit(1)
