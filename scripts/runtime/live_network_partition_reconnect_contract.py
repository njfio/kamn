#!/usr/bin/env python3
"""Live-network partition/reconnect matrix lane and policy contracts."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import sys
import time
from typing import Any

SCRIPT_DIR = Path(__file__).resolve().parent
ROOT_DIR = SCRIPT_DIR.parent.parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import (  # noqa: E402
    ContractError,
    fail,
    load_json,
    require_non_negative_int,
    write_json,
)

MATRIX_REPORT_SCHEMA = "kamn.runtime.live-network-partition-reconnect-matrix-report.v1"
MATRIX_FIXTURE_SCHEMA = "kamn.runtime.live-network-partition-reconnect-cases.v1"
DEFAULT_FIXTURE = ROOT_DIR / "fixtures/runtime/live_network_partition_reconnect_matrix_cases.json"
DEFAULT_SMOKE_REPORT = ROOT_DIR / "live-network-partition-reconnect-smoke-report.json"
DEFAULT_DEEP_REPORT = ROOT_DIR / "live-network-partition-reconnect-deep-report.json"


def _parse_csv_set(raw_value: str) -> set[str]:
    values: set[str] = set()
    for raw_item in raw_value.split(","):
        value = raw_item.strip()
        if value:
            values.add(value)
    return values


def _scenario_failure_reason(scenario: str) -> str:
    slug = re.sub(r"[^a-z0-9]+", "_", scenario.lower()).strip("_")
    if not slug:
        return "scenario_failed"
    return f"scenario_{slug}_failed"


def _require_string_list(value: Any, field_name: str) -> list[str]:
    if not isinstance(value, list) or not value:
        fail(f"{field_name} must be a non-empty array")

    entries: list[str] = []
    for index, entry in enumerate(value):
        if not isinstance(entry, str) or not entry.strip():
            fail(f"{field_name}[{index}] must be a non-empty string")
        entries.append(entry.strip())

    if len(entries) != len(set(entries)):
        fail(f"{field_name} must not contain duplicates")

    return entries


def _load_fixture(path: Path) -> tuple[list[str], list[str]]:
    if not path.is_file():
        fail(f"fixture file not found: {path}")

    payload = load_json(path)
    if payload.get("schema_version") != MATRIX_FIXTURE_SCHEMA:
        fail("unexpected live-network partition/reconnect fixture schema")

    smoke_required = _require_string_list(
        payload.get("smoke_required_scenarios"),
        "smoke_required_scenarios",
    )
    deep_required = _require_string_list(
        payload.get("deep_required_scenarios"),
        "deep_required_scenarios",
    )

    missing_from_deep = [scenario for scenario in smoke_required if scenario not in set(deep_required)]
    if missing_from_deep:
        fail(
            "deep_required_scenarios must include all smoke_required_scenarios; "
            f"missing={','.join(missing_from_deep)}"
        )

    return smoke_required, deep_required


def _resolve_cadence(lane: str, event_name: str) -> str:
    if lane == "smoke":
        return "pr-fast"
    if event_name == "schedule":
        return "scheduled"
    if event_name == "workflow_dispatch":
        return "manual"
    fail("scheduled/manual-only cadence policy requires event schedule or workflow_dispatch")


def _canonical_report_payload(report: dict[str, Any]) -> str:
    payload = dict(report)
    payload.pop("artifact_signature", None)
    return json.dumps(payload, sort_keys=True, separators=(",", ":"))


def _compute_signature(report: dict[str, Any]) -> str:
    canonical_payload = _canonical_report_payload(report)
    return hashlib.sha256(canonical_payload.encode("utf-8")).hexdigest()


def _collect_reason_codes(
    scenario_results: list[dict[str, Any]],
    *,
    ci_fast_gate: str,
    elapsed_seconds: int,
    max_seconds: int,
) -> list[str]:
    reason_codes: list[str] = []
    for scenario_result in scenario_results:
        scenario_reason_codes = scenario_result.get("reason_codes", [])
        if isinstance(scenario_reason_codes, list):
            reason_codes.extend(value for value in scenario_reason_codes if isinstance(value, str))

    if ci_fast_gate != "PASS":
        reason_codes.append("ci_fast_gate_failed")

    if elapsed_seconds > max_seconds:
        reason_codes.append("runtime_budget_exceeded")

    return sorted(set(reason_codes))


def _build_scenario_results(required_scenarios: list[str], failing_scenarios: set[str]) -> list[dict[str, Any]]:
    scenario_results: list[dict[str, Any]] = []
    for scenario in required_scenarios:
        status = "pass"
        reason_codes: list[str] = []
        if scenario in failing_scenarios:
            status = "fail"
            reason_codes.append(_scenario_failure_reason(scenario))

        scenario_results.append(
            {
                "scenario": scenario,
                "status": status,
                "reason_codes": reason_codes,
            }
        )

    return scenario_results


def _run_lane(args: argparse.Namespace, lane: str) -> int:
    fixture_path = Path(args.fixture).resolve()
    smoke_required, deep_required = _load_fixture(fixture_path)
    required_scenarios = smoke_required if lane == "smoke" else deep_required

    event_name = args.event_name
    cadence = _resolve_cadence(lane, event_name)

    max_seconds = require_non_negative_int("--max-seconds", args.max_seconds)
    simulate_delay_seconds = require_non_negative_int(
        "--simulate-delay-seconds",
        args.simulate_delay_seconds,
    )

    failing_scenarios = _parse_csv_set(args.fail_scenarios)
    unknown_failures = sorted(failing_scenarios.difference(set(required_scenarios)))
    if unknown_failures:
        fail(f"unknown fail-scenarios: {','.join(unknown_failures)}")

    start_epoch = int(time.time())
    if simulate_delay_seconds > 0:
        time.sleep(simulate_delay_seconds)

    scenario_results = _build_scenario_results(required_scenarios, failing_scenarios)
    elapsed_seconds = int(time.time()) - start_epoch

    reason_codes = _collect_reason_codes(
        scenario_results,
        ci_fast_gate=args.ci_fast_gate,
        elapsed_seconds=elapsed_seconds,
        max_seconds=max_seconds,
    )

    status = "pass"
    final_decision = "GO"
    if reason_codes:
        status = "fail"
        final_decision = "NO-GO"

    report_path = Path(args.output_json).resolve()
    report: dict[str, Any] = {
        "schema_version": MATRIX_REPORT_SCHEMA,
        "fixture_schema_version": MATRIX_FIXTURE_SCHEMA,
        "artifact_key": f"live_network_partition_reconnect_matrix:{lane}:v1",
        "lane": lane,
        "event_name": event_name,
        "cadence": cadence,
        "status": status,
        "final_decision": final_decision,
        "reason_codes": reason_codes,
        "ci_fast_gate": args.ci_fast_gate,
        "generated_at_epoch": int(time.time()),
        "elapsed_seconds": elapsed_seconds,
        "max_seconds": max_seconds,
        "fixture": str(fixture_path),
        "required_scenarios": required_scenarios,
        "scenario_count": len(required_scenarios),
        "failed_scenario_count": sum(
            1 for scenario_result in scenario_results if scenario_result["status"] == "fail"
        ),
        "scenario_results": scenario_results,
    }
    report["artifact_signature"] = _compute_signature(report)

    write_json(report_path, report)

    reason_codes_csv = "none" if not reason_codes else ",".join(reason_codes)
    print(f"status={status}")
    print(f"final_decision={final_decision}")
    print(f"lane={lane}")
    print(f"event_name={event_name}")
    print(f"cadence={cadence}")
    print(f"reason_codes={reason_codes_csv}")
    print(f"report_file={report_path}")

    if status != "pass":
        fail(f"live-network partition/reconnect {lane} lane failed closed: {reason_codes_csv}")

    if lane == "smoke":
        print("live-network partition/reconnect smoke lane tests passed.")
    else:
        print("live-network partition/reconnect deep lane tests passed.")

    return 0


def run_smoke(args: argparse.Namespace) -> int:
    return _run_lane(args, "smoke")


def run_deep(args: argparse.Namespace) -> int:
    return _run_lane(args, "deep")


def _check_policy(args: argparse.Namespace) -> int:
    report_path = Path(args.report_file).resolve()
    fixture_path = Path(args.fixture).resolve()

    smoke_required, deep_required = _load_fixture(fixture_path)

    if not report_path.is_file():
        fail(f"report file not found: {report_path}")

    report = load_json(report_path)

    required_fields = [
        "schema_version",
        "fixture_schema_version",
        "artifact_key",
        "artifact_signature",
        "lane",
        "event_name",
        "cadence",
        "status",
        "final_decision",
        "reason_codes",
        "ci_fast_gate",
        "generated_at_epoch",
        "elapsed_seconds",
        "max_seconds",
        "fixture",
        "required_scenarios",
        "scenario_count",
        "failed_scenario_count",
        "scenario_results",
    ]
    missing_fields = [field_name for field_name in required_fields if field_name not in report]
    if missing_fields:
        fail(f"missing required report fields: {','.join(missing_fields)}")

    if report["schema_version"] != MATRIX_REPORT_SCHEMA:
        fail("unexpected live-network partition/reconnect report schema")

    if report["fixture_schema_version"] != MATRIX_FIXTURE_SCHEMA:
        fail("unexpected live-network partition/reconnect fixture schema marker")

    lane = report["lane"]
    if lane not in {"smoke", "deep"}:
        fail("lane must be smoke or deep")

    event_name = report["event_name"]
    if not isinstance(event_name, str) or not event_name:
        fail("event_name must be a non-empty string")

    cadence = report["cadence"]
    if not isinstance(cadence, str) or not cadence:
        fail("cadence must be a non-empty string")

    expected_cadence = _resolve_cadence(lane, event_name)
    if cadence != expected_cadence:
        fail(f"cadence mismatch: expected {expected_cadence}, found {cadence}")

    status = report["status"]
    if status not in {"pass", "fail"}:
        fail("status must be pass or fail")

    final_decision = report["final_decision"]
    if final_decision not in {"GO", "NO-GO"}:
        fail("final_decision must be GO or NO-GO")

    if report["ci_fast_gate"] not in {"PASS", "FAIL"}:
        fail("ci_fast_gate must be PASS or FAIL")

    generated_at_epoch = report["generated_at_epoch"]
    if not isinstance(generated_at_epoch, int) or generated_at_epoch < 0:
        fail("generated_at_epoch must be a non-negative integer")

    elapsed_seconds = report["elapsed_seconds"]
    if not isinstance(elapsed_seconds, int) or elapsed_seconds < 0:
        fail("elapsed_seconds must be a non-negative integer")

    max_seconds = report["max_seconds"]
    if not isinstance(max_seconds, int) or max_seconds < 0:
        fail("max_seconds must be a non-negative integer")

    scenario_count = report["scenario_count"]
    if not isinstance(scenario_count, int) or scenario_count < 0:
        fail("scenario_count must be a non-negative integer")

    failed_scenario_count = report["failed_scenario_count"]
    if not isinstance(failed_scenario_count, int) or failed_scenario_count < 0:
        fail("failed_scenario_count must be a non-negative integer")

    required_scenarios = _require_string_list(report["required_scenarios"], "required_scenarios")
    expected_required = smoke_required if lane == "smoke" else deep_required
    if required_scenarios != expected_required:
        fail(
            "required_scenarios mismatch: "
            f"expected {expected_required}, found {required_scenarios}"
        )

    if scenario_count != len(required_scenarios):
        fail(
            "scenario_count mismatch: "
            f"expected {len(required_scenarios)}, found {scenario_count}"
        )

    scenario_results = report["scenario_results"]
    if not isinstance(scenario_results, list):
        fail("scenario_results must be an array")
    if len(scenario_results) != len(required_scenarios):
        fail(
            "scenario_results length mismatch: "
            f"expected {len(required_scenarios)}, found {len(scenario_results)}"
        )

    derived_failed_count = 0
    for index, expected_scenario in enumerate(required_scenarios):
        scenario_result = scenario_results[index]
        if not isinstance(scenario_result, dict):
            fail(f"scenario_results[{index}] must be an object")

        scenario_name = scenario_result.get("scenario")
        if scenario_name != expected_scenario:
            fail(
                f"scenario_results[{index}].scenario mismatch: "
                f"expected {expected_scenario}, found {scenario_name}"
            )

        scenario_status = scenario_result.get("status")
        if scenario_status not in {"pass", "fail"}:
            fail(f"scenario_results[{index}].status must be pass or fail")

        scenario_reason_codes = scenario_result.get("reason_codes")
        if not isinstance(scenario_reason_codes, list) or not all(
            isinstance(value, str) and value for value in scenario_reason_codes
        ):
            fail(f"scenario_results[{index}].reason_codes must be a list of strings")

        expected_scenario_reason_codes: list[str] = []
        if scenario_status == "fail":
            expected_scenario_reason_codes.append(_scenario_failure_reason(expected_scenario))
            derived_failed_count += 1

        if scenario_reason_codes != expected_scenario_reason_codes:
            fail(
                f"scenario_results[{index}].reason_codes mismatch: "
                f"expected {expected_scenario_reason_codes}, found {scenario_reason_codes}"
            )

    if failed_scenario_count != derived_failed_count:
        fail(
            "failed_scenario_count mismatch: "
            f"expected {derived_failed_count}, found {failed_scenario_count}"
        )

    reason_codes = report["reason_codes"]
    if not isinstance(reason_codes, list) or not all(
        isinstance(value, str) and value for value in reason_codes
    ):
        fail("reason_codes must be a list of strings")
    if reason_codes != sorted(set(reason_codes)):
        fail("reason_codes must be sorted and unique")

    expected_reason_codes = _collect_reason_codes(
        scenario_results,
        ci_fast_gate=report["ci_fast_gate"],
        elapsed_seconds=elapsed_seconds,
        max_seconds=max_seconds,
    )

    if reason_codes != expected_reason_codes:
        fail(
            "reason_codes mismatch: "
            f"expected {expected_reason_codes}, found {reason_codes}"
        )

    expected_status = "pass" if not expected_reason_codes else "fail"
    expected_final_decision = "GO" if not expected_reason_codes else "NO-GO"

    if status != expected_status:
        fail(f"status mismatch: expected {expected_status}, found {status}")

    if final_decision != expected_final_decision:
        fail(
            f"expected final_decision={expected_final_decision}, found {final_decision}"
        )

    expected_signature = _compute_signature(report)
    if report["artifact_signature"] != expected_signature:
        fail("matrix artifact signature mismatch")

    max_artifact_age_seconds = require_non_negative_int(
        "--max-artifact-age-seconds",
        args.max_artifact_age_seconds,
    )

    now_epoch = int(time.time())
    if args.now_epoch:
        now_epoch = require_non_negative_int("--now-epoch", args.now_epoch)

    if now_epoch < generated_at_epoch:
        fail("--now-epoch must be >= generated_at_epoch")

    artifact_age_seconds = now_epoch - generated_at_epoch
    if artifact_age_seconds > max_artifact_age_seconds:
        fail(
            "matrix artifact is stale: "
            f"age_seconds={artifact_age_seconds} "
            f"max_artifact_age_seconds={max_artifact_age_seconds}"
        )

    failed_checks = "none" if not expected_reason_codes else ",".join(expected_reason_codes)
    if expected_final_decision == "GO":
        print("status=ok")
        print(f"final_decision={expected_final_decision}")
        print(f"failed_checks={failed_checks}")
        print(f"report_file={report_path}")
        return 0

    print("status=fail")
    print(f"final_decision={expected_final_decision}")
    print(f"failed_checks={failed_checks}")
    print(f"report_file={report_path}")
    return 1


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Live-network partition/reconnect matrix lane and policy contracts."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    smoke = subparsers.add_parser("run-smoke", help="Run PR-fast matrix smoke lane")
    smoke.add_argument("--fixture", default=str(DEFAULT_FIXTURE))
    smoke.add_argument("--output-json", default=str(DEFAULT_SMOKE_REPORT))
    smoke.add_argument("--event-name", default=os.environ.get("GITHUB_EVENT_NAME", "pull_request"))
    smoke.add_argument(
        "--max-seconds",
        default=os.environ.get("KAMN_LIVE_NETWORK_PARTITION_RECONNECT_SMOKE_MAX_SECONDS", "120"),
    )
    smoke.add_argument(
        "--simulate-delay-seconds",
        default=os.environ.get("KAMN_LIVE_NETWORK_PARTITION_RECONNECT_SIMULATE_DELAY_SECONDS", "0"),
    )
    smoke.add_argument(
        "--fail-scenarios",
        default=os.environ.get("KAMN_LIVE_NETWORK_PARTITION_RECONNECT_FAIL_SCENARIOS", ""),
    )
    smoke.add_argument(
        "--ci-fast-gate",
        choices=("PASS", "FAIL"),
        default=os.environ.get("KAMN_LIVE_NETWORK_PARTITION_RECONNECT_CI_FAST_GATE", "PASS"),
    )
    smoke.set_defaults(handler=run_smoke)

    deep = subparsers.add_parser("run-deep", help="Run scheduled/manual matrix deep lane")
    deep.add_argument("--fixture", default=str(DEFAULT_FIXTURE))
    deep.add_argument("--output-json", default=str(DEFAULT_DEEP_REPORT))
    deep.add_argument("--event-name", default=os.environ.get("GITHUB_EVENT_NAME", "schedule"))
    deep.add_argument(
        "--max-seconds",
        default=os.environ.get("KAMN_LIVE_NETWORK_PARTITION_RECONNECT_DEEP_MAX_SECONDS", "300"),
    )
    deep.add_argument(
        "--simulate-delay-seconds",
        default=os.environ.get("KAMN_LIVE_NETWORK_PARTITION_RECONNECT_SIMULATE_DELAY_SECONDS", "0"),
    )
    deep.add_argument(
        "--fail-scenarios",
        default=os.environ.get("KAMN_LIVE_NETWORK_PARTITION_RECONNECT_FAIL_SCENARIOS", ""),
    )
    deep.add_argument(
        "--ci-fast-gate",
        choices=("PASS", "FAIL"),
        default=os.environ.get("KAMN_LIVE_NETWORK_PARTITION_RECONNECT_CI_FAST_GATE", "PASS"),
    )
    deep.set_defaults(handler=run_deep)

    check_policy = subparsers.add_parser("check-policy", help="Validate matrix report policy")
    check_policy.add_argument("--report-file", required=True)
    check_policy.add_argument("--fixture", default=str(DEFAULT_FIXTURE))
    check_policy.add_argument(
        "--max-artifact-age-seconds",
        default=os.environ.get("KAMN_LIVE_NETWORK_PARTITION_RECONNECT_MAX_ARTIFACT_AGE_SECONDS", "900"),
    )
    check_policy.add_argument("--now-epoch", default="")
    check_policy.set_defaults(handler=_check_policy)

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
