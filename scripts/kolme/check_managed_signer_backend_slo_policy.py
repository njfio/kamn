#!/usr/bin/env python3
"""Fail-closed policy checker for managed-signer backend SLO telemetry bundles."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

POLICY_SCHEMA_VERSION = "kamn.kolme.managed-signer-backend-slo-policy-report.v1"
TELEMETRY_SCHEMA_VERSION = "kamn.kolme.managed-signer-backend-slo-telemetry.v1"
GO_DECISION = "GO"
NO_GO_DECISION = "NO-GO"
GO_REASON_CODE = "managed_signer_backend_slo_within_threshold"
GO_REMEDIATION_MARKER = "managed_signer_backend_no_action_required"
MAX_BPS = 10_000

TIMEOUT_BREACH_REASON = "managed_signer_backend_timeout_rate_threshold_exceeded"
UNAVAILABLE_BREACH_REASON = "managed_signer_backend_unavailable_rate_threshold_exceeded"
ERROR_BREACH_REASON = "managed_signer_backend_error_rate_threshold_exceeded"
CI_FAST_GATE_BREACH_REASON = "managed_signer_backend_ci_fast_gate_failed"

BREACH_TO_REMEDIATION = {
    TIMEOUT_BREACH_REASON: "managed_signer_backend_reduce_timeout_burst",
    UNAVAILABLE_BREACH_REASON: "managed_signer_backend_failover_endpoint",
    ERROR_BREACH_REASON: "managed_signer_backend_enable_circuit_breaker",
    CI_FAST_GATE_BREACH_REASON: "managed_signer_backend_replay_ci_fast_gate",
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Evaluate managed-signer backend SLO telemetry policy checks."
    )
    parser.add_argument("--telemetry-bundle", required=True)
    parser.add_argument("--expected-final-decision", choices=[GO_DECISION, NO_GO_DECISION], default="")
    parser.add_argument("--require-reason-code", action="append", default=[])
    parser.add_argument("--output-json", default="")
    return parser.parse_args()


def _append_unique(sequence: list[str], value: str) -> None:
    if value not in sequence:
        sequence.append(value)


def _read_bundle(bundle_path: Path) -> tuple[dict[str, Any], list[str]]:
    reason_codes: list[str] = []
    if not bundle_path.is_file():
        reason_codes.append("telemetry_bundle_missing")
        return {}, reason_codes
    try:
        payload = json.loads(bundle_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        reason_codes.append("telemetry_bundle_invalid_json")
        return {}, reason_codes
    if not isinstance(payload, dict):
        reason_codes.append("telemetry_bundle_invalid_type")
        return {}, reason_codes
    return payload, reason_codes


def _read_non_empty_string(payload: dict[str, Any], key: str, reason_codes: list[str]) -> str:
    value = payload.get(key)
    if not isinstance(value, str) or not value.strip():
        reason_codes.append(f"{key}_missing")
        return ""
    return value.strip()


def _read_non_negative_int(
    payload: dict[str, Any],
    key: str,
    reason_codes: list[str],
    *,
    max_value: int | None = None,
) -> int:
    value = payload.get(key)
    if not isinstance(value, int):
        reason_codes.append(f"{key}_invalid")
        return 0
    if value < 0:
        reason_codes.append(f"{key}_invalid")
        return 0
    if max_value is not None and value > max_value:
        reason_codes.append(f"{key}_invalid")
        return 0
    return value


def _ordered_unique(values: list[str]) -> list[str]:
    ordered: list[str] = []
    for value in values:
        _append_unique(ordered, value)
    return ordered


def evaluate(bundle: dict[str, Any], args: argparse.Namespace, base_reason_codes: list[str]) -> tuple[str, list[str], list[str]]:
    reason_codes: list[str] = list(base_reason_codes)

    telemetry_schema_version = bundle.get("schema_version")
    if telemetry_schema_version != TELEMETRY_SCHEMA_VERSION:
        reason_codes.append("telemetry_schema_version_mismatch")

    backend_name = _read_non_empty_string(bundle, "backend_name", reason_codes)
    signer_profile = _read_non_empty_string(bundle, "signer_profile", reason_codes)
    signer_key_source = _read_non_empty_string(bundle, "signer_key_source", reason_codes)
    if signer_key_source and signer_key_source != "managed-external":
        reason_codes.append("signer_key_source_invalid")

    ci_fast_gate = _read_non_empty_string(bundle, "ci_fast_gate", reason_codes)
    if ci_fast_gate and ci_fast_gate not in {"PASS", "FAIL"}:
        reason_codes.append("ci_fast_gate_invalid")

    sample_count = _read_non_negative_int(bundle, "sample_count", reason_codes)
    if sample_count <= 0:
        reason_codes.append("sample_count_invalid")

    timeout_events = _read_non_negative_int(bundle, "timeout_events", reason_codes)
    unavailable_events = _read_non_negative_int(bundle, "unavailable_events", reason_codes)
    error_events = _read_non_negative_int(bundle, "error_events", reason_codes)

    if sample_count > 0:
        if timeout_events > sample_count:
            reason_codes.append("timeout_events_exceeds_sample_count")
        if unavailable_events > sample_count:
            reason_codes.append("unavailable_events_exceeds_sample_count")
        if error_events > sample_count:
            reason_codes.append("error_events_exceeds_sample_count")

    timeout_rate_bps = _read_non_negative_int(bundle, "timeout_rate_bps", reason_codes, max_value=MAX_BPS)
    unavailable_rate_bps = _read_non_negative_int(bundle, "unavailable_rate_bps", reason_codes, max_value=MAX_BPS)
    error_rate_bps = _read_non_negative_int(bundle, "error_rate_bps", reason_codes, max_value=MAX_BPS)

    max_timeout_rate_bps = _read_non_negative_int(
        bundle, "max_timeout_rate_bps", reason_codes, max_value=MAX_BPS
    )
    max_unavailable_rate_bps = _read_non_negative_int(
        bundle, "max_unavailable_rate_bps", reason_codes, max_value=MAX_BPS
    )
    max_error_rate_bps = _read_non_negative_int(
        bundle, "max_error_rate_bps", reason_codes, max_value=MAX_BPS
    )

    if sample_count > 0:
        expected_timeout_rate_bps = (timeout_events * MAX_BPS) // sample_count
        expected_unavailable_rate_bps = (unavailable_events * MAX_BPS) // sample_count
        expected_error_rate_bps = (error_events * MAX_BPS) // sample_count
        if timeout_rate_bps != expected_timeout_rate_bps:
            reason_codes.append("timeout_rate_bps_mismatch")
        if unavailable_rate_bps != expected_unavailable_rate_bps:
            reason_codes.append("unavailable_rate_bps_mismatch")
        if error_rate_bps != expected_error_rate_bps:
            reason_codes.append("error_rate_bps_mismatch")

    threshold_reasons: list[str] = []
    if timeout_rate_bps > max_timeout_rate_bps:
        threshold_reasons.append(TIMEOUT_BREACH_REASON)
    if unavailable_rate_bps > max_unavailable_rate_bps:
        threshold_reasons.append(UNAVAILABLE_BREACH_REASON)
    if error_rate_bps > max_error_rate_bps:
        threshold_reasons.append(ERROR_BREACH_REASON)
    if ci_fast_gate == "FAIL":
        threshold_reasons.append(CI_FAST_GATE_BREACH_REASON)

    reason_codes.extend(threshold_reasons)

    threshold_breaches = bundle.get("threshold_breaches")
    if not isinstance(threshold_breaches, list) or not all(
        isinstance(reason, str) and reason.strip() for reason in threshold_breaches
    ):
        reason_codes.append("threshold_breaches_invalid")
    else:
        telemetry_threshold_reasons = set(threshold_breaches)
        policy_threshold_reasons = set(threshold_reasons)
        if telemetry_threshold_reasons != policy_threshold_reasons:
            reason_codes.append("threshold_breaches_mismatch")

    bundle_final_decision = bundle.get("final_decision")
    if bundle_final_decision not in {GO_DECISION, NO_GO_DECISION}:
        reason_codes.append("telemetry_final_decision_invalid")

    reason_codes = _ordered_unique(reason_codes)
    derived_final_decision = GO_DECISION if not reason_codes else NO_GO_DECISION

    if bundle_final_decision in {GO_DECISION, NO_GO_DECISION} and bundle_final_decision != derived_final_decision:
        reason_codes.append("telemetry_final_decision_mismatch")
        derived_final_decision = NO_GO_DECISION

    if args.expected_final_decision and derived_final_decision != args.expected_final_decision:
        reason_codes.append("observed_final_decision_mismatch")
        derived_final_decision = NO_GO_DECISION

    if derived_final_decision == GO_DECISION:
        output_reason_codes = [GO_REASON_CODE]
    else:
        output_reason_codes = _ordered_unique(reason_codes)

    for required_reason_code in args.require_reason_code:
        if required_reason_code not in output_reason_codes:
            output_reason_codes.append(f"required_reason_code_missing:{required_reason_code}")
            derived_final_decision = NO_GO_DECISION

    if derived_final_decision == GO_DECISION:
        remediation_markers = [GO_REMEDIATION_MARKER]
    else:
        remediation_markers: list[str] = []
        for reason_code in output_reason_codes:
            marker = BREACH_TO_REMEDIATION.get(reason_code)
            if marker:
                _append_unique(remediation_markers, marker)
        if not remediation_markers:
            remediation_markers = ["managed_signer_backend_rebuild_telemetry_bundle"]

    # Keep these fields validated to prevent accidental regressions where metadata is dropped.
    if not backend_name:
        _append_unique(output_reason_codes, "backend_name_missing")
    if not signer_profile:
        _append_unique(output_reason_codes, "signer_profile_missing")
    if not signer_key_source:
        _append_unique(output_reason_codes, "signer_key_source_missing")
    if derived_final_decision == GO_DECISION and output_reason_codes != [GO_REASON_CODE]:
        derived_final_decision = NO_GO_DECISION

    return derived_final_decision, output_reason_codes, remediation_markers


def main() -> int:
    args = parse_args()
    bundle_path = Path(args.telemetry_bundle).resolve()
    bundle, base_reason_codes = _read_bundle(bundle_path)
    final_decision, reason_codes, remediation_markers = evaluate(bundle, args, base_reason_codes)

    report = {
        "schema_version": POLICY_SCHEMA_VERSION,
        "telemetry_bundle": str(bundle_path),
        "telemetry_schema_version": bundle.get("schema_version"),
        "backend_name": bundle.get("backend_name"),
        "signer_profile": bundle.get("signer_profile"),
        "signer_key_source": bundle.get("signer_key_source"),
        "sample_count": bundle.get("sample_count"),
        "timeout_rate_bps": bundle.get("timeout_rate_bps"),
        "unavailable_rate_bps": bundle.get("unavailable_rate_bps"),
        "error_rate_bps": bundle.get("error_rate_bps"),
        "max_timeout_rate_bps": bundle.get("max_timeout_rate_bps"),
        "max_unavailable_rate_bps": bundle.get("max_unavailable_rate_bps"),
        "max_error_rate_bps": bundle.get("max_error_rate_bps"),
        "ci_fast_gate": bundle.get("ci_fast_gate"),
        "required_reason_codes": args.require_reason_code,
        "reason_codes": reason_codes,
        "remediation_markers": remediation_markers,
        "final_decision": final_decision,
        "contracts": {
            "telemetry_schema_version": TELEMETRY_SCHEMA_VERSION,
            "go_reason_code": GO_REASON_CODE,
            "go_remediation_marker": GO_REMEDIATION_MARKER,
            "timeout_rate_breach_reason_code": TIMEOUT_BREACH_REASON,
            "unavailable_rate_breach_reason_code": UNAVAILABLE_BREACH_REASON,
            "error_rate_breach_reason_code": ERROR_BREACH_REASON,
            "ci_fast_gate_breach_reason_code": CI_FAST_GATE_BREACH_REASON,
        },
    }

    if args.output_json:
        output_file = Path(args.output_json).resolve()
        output_file.parent.mkdir(parents=True, exist_ok=True)
        output_file.write_text(json.dumps(report, sort_keys=True, indent=2) + "\n", encoding="utf-8")

    status = "ok" if final_decision == GO_DECISION else "fail"
    reason_codes_csv = ",".join(reason_codes) if reason_codes else "none"
    remediation_csv = ",".join(remediation_markers) if remediation_markers else "none"
    print(f"status={status}")
    print(f"final_decision={final_decision}")
    print(f"reason_codes={reason_codes_csv}")
    print(f"remediation_markers={remediation_csv}")
    if args.output_json:
        print(f"report_file={Path(args.output_json).resolve()}")

    return 0 if final_decision == GO_DECISION else 1


if __name__ == "__main__":
    raise SystemExit(main())
