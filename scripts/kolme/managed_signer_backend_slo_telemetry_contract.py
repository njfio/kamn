#!/usr/bin/env python3
"""Managed-signer backend SLO telemetry bundle generator."""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
from pathlib import Path
import sys
from typing import Any

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import fail, write_json  # noqa: E402

SCHEMA_VERSION = "kamn.kolme.managed-signer-backend-slo-telemetry.v1"
GO_DECISION = "GO"
NO_GO_DECISION = "NO-GO"
MAX_BPS = 10_000
REQUIRED_SIGNER_KEY_SOURCE = "managed-external"


def _parse_positive_int(field_name: str, raw_value: str) -> int:
    try:
        parsed = int(raw_value)
    except (TypeError, ValueError):
        fail(f"{field_name} must be an integer")
    if parsed <= 0:
        fail(f"{field_name} must be > 0")
    return parsed


def _parse_non_negative_int(field_name: str, raw_value: str) -> int:
    try:
        parsed = int(raw_value)
    except (TypeError, ValueError):
        fail(f"{field_name} must be an integer")
    if parsed < 0:
        fail(f"{field_name} must be >= 0")
    return parsed


def _parse_bps(field_name: str, raw_value: str) -> int:
    parsed = _parse_non_negative_int(field_name, raw_value)
    if parsed > MAX_BPS:
        fail(f"{field_name} must be <= {MAX_BPS}")
    return parsed


def _parse_ci_fast_gate(raw_value: str) -> str:
    if raw_value in {"PASS", "FAIL"}:
        return raw_value
    fail("ci-fast-gate must be PASS or FAIL")


def _parse_utc_timestamp(field_name: str, raw_value: str) -> datetime:
    if not isinstance(raw_value, str) or not raw_value.strip():
        fail(f"{field_name} must be a non-empty UTC timestamp")
    normalized = raw_value.replace("Z", "+00:00")
    try:
        parsed = datetime.fromisoformat(normalized)
    except ValueError:
        fail(f"{field_name} must be RFC3339 UTC timestamp (example: 2026-02-13T00:00:00Z)")
    if parsed.tzinfo is None:
        fail(f"{field_name} must include timezone offset")
    return parsed.astimezone(timezone.utc)


def _parse_non_empty(field_name: str, raw_value: str) -> str:
    if not isinstance(raw_value, str) or not raw_value.strip():
        fail(f"{field_name} must be non-empty")
    return raw_value.strip()


def _compute_rate_bps(events: int, sample_count: int) -> int:
    return (events * MAX_BPS) // sample_count


def generate_bundle(args: argparse.Namespace) -> int:
    output_file = _parse_non_empty("output-file", args.output_file)
    backend_name = _parse_non_empty("backend-name", args.backend_name)
    signer_profile = _parse_non_empty("signer-profile", args.signer_profile)
    signer_key_source = _parse_non_empty("signer-key-source", args.signer_key_source)
    if signer_key_source != REQUIRED_SIGNER_KEY_SOURCE:
        fail(f"signer-key-source must be {REQUIRED_SIGNER_KEY_SOURCE}")

    window_start_utc = _parse_utc_timestamp("window-start-utc", args.window_start_utc)
    window_end_utc = _parse_utc_timestamp("window-end-utc", args.window_end_utc)
    if window_end_utc <= window_start_utc:
        fail("window-end-utc must be after window-start-utc")

    sample_count = _parse_positive_int("sample-count", args.sample_count)
    timeout_events = _parse_non_negative_int("timeout-events", args.timeout_events)
    unavailable_events = _parse_non_negative_int("unavailable-events", args.unavailable_events)
    error_events = _parse_non_negative_int("error-events", args.error_events)

    if timeout_events > sample_count:
        fail("timeout-events must be <= sample-count")
    if unavailable_events > sample_count:
        fail("unavailable-events must be <= sample-count")
    if error_events > sample_count:
        fail("error-events must be <= sample-count")

    max_timeout_rate_bps = _parse_bps("max-timeout-rate-bps", args.max_timeout_rate_bps)
    max_unavailable_rate_bps = _parse_bps(
        "max-unavailable-rate-bps", args.max_unavailable_rate_bps
    )
    max_error_rate_bps = _parse_bps("max-error-rate-bps", args.max_error_rate_bps)
    ci_fast_gate = _parse_ci_fast_gate(args.ci_fast_gate)

    timeout_rate_bps = _compute_rate_bps(timeout_events, sample_count)
    unavailable_rate_bps = _compute_rate_bps(unavailable_events, sample_count)
    error_rate_bps = _compute_rate_bps(error_events, sample_count)

    threshold_breaches: list[str] = []
    if timeout_rate_bps > max_timeout_rate_bps:
        threshold_breaches.append("managed_signer_backend_timeout_rate_threshold_exceeded")
    if unavailable_rate_bps > max_unavailable_rate_bps:
        threshold_breaches.append("managed_signer_backend_unavailable_rate_threshold_exceeded")
    if error_rate_bps > max_error_rate_bps:
        threshold_breaches.append("managed_signer_backend_error_rate_threshold_exceeded")
    if ci_fast_gate != "PASS":
        threshold_breaches.append("managed_signer_backend_ci_fast_gate_failed")

    final_decision = GO_DECISION if not threshold_breaches else NO_GO_DECISION

    payload: dict[str, Any] = {
        "schema_version": SCHEMA_VERSION,
        "generated_at_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "window_start_utc": window_start_utc.strftime("%Y-%m-%dT%H:%M:%SZ"),
        "window_end_utc": window_end_utc.strftime("%Y-%m-%dT%H:%M:%SZ"),
        "backend_name": backend_name,
        "signer_profile": signer_profile,
        "signer_key_source": signer_key_source,
        "sample_count": sample_count,
        "timeout_events": timeout_events,
        "unavailable_events": unavailable_events,
        "error_events": error_events,
        "timeout_rate_bps": timeout_rate_bps,
        "unavailable_rate_bps": unavailable_rate_bps,
        "error_rate_bps": error_rate_bps,
        "max_timeout_rate_bps": max_timeout_rate_bps,
        "max_unavailable_rate_bps": max_unavailable_rate_bps,
        "max_error_rate_bps": max_error_rate_bps,
        "threshold_breaches": threshold_breaches,
        "final_decision": final_decision,
        "ci_fast_gate": ci_fast_gate,
        "contracts": {
            "required_signer_key_source": REQUIRED_SIGNER_KEY_SOURCE,
            "threshold_source": "operator-slo-policy",
            "timeout_rate_bps_threshold_field": "max_timeout_rate_bps",
            "unavailable_rate_bps_threshold_field": "max_unavailable_rate_bps",
            "error_rate_bps_threshold_field": "max_error_rate_bps",
            "timeout_rate_breach_reason_code": "managed_signer_backend_timeout_rate_threshold_exceeded",
            "unavailable_rate_breach_reason_code": "managed_signer_backend_unavailable_rate_threshold_exceeded",
            "error_rate_breach_reason_code": "managed_signer_backend_error_rate_threshold_exceeded",
            "ci_fast_gate_required": True,
        },
    }

    output_path = Path(output_file)
    write_json(output_path, payload)
    threshold_breaches_csv = ",".join(threshold_breaches) if threshold_breaches else "none"
    print("status=generated")
    print(f"bundle_file={output_path}")
    print(f"final_decision={final_decision}")
    print(f"threshold_breaches={threshold_breaches_csv}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Managed-signer backend SLO telemetry bundle generator."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    generate = subparsers.add_parser("generate", help="Generate managed-signer backend SLO telemetry bundle.")
    generate.add_argument("--output-file", required=True)
    generate.add_argument("--window-start-utc", required=True)
    generate.add_argument("--window-end-utc", required=True)
    generate.add_argument("--backend-name", required=True)
    generate.add_argument("--signer-profile", required=True)
    generate.add_argument("--signer-key-source", required=True)
    generate.add_argument("--sample-count", required=True)
    generate.add_argument("--timeout-events", required=True)
    generate.add_argument("--unavailable-events", required=True)
    generate.add_argument("--error-events", required=True)
    generate.add_argument("--max-timeout-rate-bps", required=True)
    generate.add_argument("--max-unavailable-rate-bps", required=True)
    generate.add_argument("--max-error-rate-bps", required=True)
    generate.add_argument("--ci-fast-gate", default="PASS")
    generate.set_defaults(handler=generate_bundle)

    return parser


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()
    return args.handler(args)


if __name__ == "__main__":
    raise SystemExit(main())
