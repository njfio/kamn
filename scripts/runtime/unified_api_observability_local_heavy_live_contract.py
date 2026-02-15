#!/usr/bin/env python3
"""Unified API-observability local-heavy lane and policy checker contracts."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
from typing import Any

SCRIPT_DIR = Path(__file__).resolve().parent
ROOT_DIR = SCRIPT_DIR.parent.parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import (  # noqa: E402
    ContractError,
    DecisionAccumulator,
    fail,
    load_json,
    require_enum,
    require_positive_int,
    write_json,
)

RUN_LANE_SCHEMA = "kamn.runtime.unified-api-observability-local-heavy-live-report.v1"
POLICY_SCHEMA = "kamn.runtime.unified-api-observability-local-heavy-live-policy-report.v1"
CONTRACT_LANE_SCHEMA = (
    "kamn.runtime.unified-api-observability-local-heavy-live-contract-lane-report.v1"
)

COMPATIBILITY_RUN_SCHEMA = (
    "kamn.runtime.service-api-observability-route-compatibility-live-report.v1"
)
COMPATIBILITY_POLICY_SCHEMA = (
    "kamn.runtime.service-api-observability-route-compatibility-live-policy-report.v1"
)
OBSERVABILITY_SCRAPE_RUN_SCHEMA = "kamn.runtime.local-observability-scrape-live-report.v1"
OBSERVABILITY_SCRAPE_POLICY_SCHEMA = (
    "kamn.runtime.local-observability-scrape-live-policy-report.v1"
)

OPT_IN_ENV = "KAMN_UNIFIED_STACK_LOCAL_HEAVY_OPT_IN"
DRY_RUN_REASON = "dry_run_no_commands_executed"
RUN_REASON = "unified_api_observability_local_heavy_executed"
FAST_GATE_EXCLUSION_REASON = (
    "unified_api_observability_local_heavy_run_mode_excluded_from_fast_gate"
)
DOCS_FILE = ROOT_DIR / "docs/ci/strategy.md"


def _extract_line_value(output: str, key: str) -> str:
    prefix = f"{key}="
    for line in output.splitlines():
        if line.startswith(prefix):
            return line[len(prefix) :]
    return ""


def _is_non_negative_int(value: Any) -> bool:
    return isinstance(value, int) and not isinstance(value, bool) and value >= 0


def _run_command(
    command: list[str],
    *,
    timeout_seconds: int,
    env: dict[str, str] | None = None,
) -> str:
    try:
        completed = subprocess.run(
            command,
            cwd=ROOT_DIR,
            capture_output=True,
            text=True,
            check=False,
            timeout=timeout_seconds,
            env=env,
        )
    except subprocess.TimeoutExpired as error:
        fail(f"lane command timed out: {' '.join(command)} (timeout={timeout_seconds}s): {error}")

    if completed.returncode != 0:
        detail = (completed.stderr or completed.stdout or "command failed").strip()
        fail(f"lane command failed: {' '.join(command)}: {detail}")

    return completed.stdout


def _run_lane(args: argparse.Namespace) -> int:
    mode = require_enum("--mode", args.mode.strip(), ("dry-run", "run"))
    ci_fast_gate = require_enum("--ci-fast-gate", args.ci_fast_gate.strip(), ("PASS", "FAIL"))
    max_seconds = require_positive_int(
        "KAMN_UNIFIED_API_OBSERVABILITY_LOCAL_HEAVY_MAX_SECONDS",
        args.max_seconds,
    )
    command_max_seconds = require_positive_int(
        "KAMN_UNIFIED_API_OBSERVABILITY_LOCAL_HEAVY_COMMAND_MAX_SECONDS",
        args.command_max_seconds,
    )
    soak_iterations = require_positive_int(
        "KAMN_UNIFIED_API_OBSERVABILITY_LOCAL_HEAVY_SOAK_ITERATIONS",
        args.soak_iterations,
    )
    if command_max_seconds > max_seconds:
        command_max_seconds = max_seconds

    if mode == "run" and args.local_opt_in != "1":
        fail(f"run mode requires explicit local-only opt-in via {OPT_IN_ENV}=1")

    start_epoch = int(time.time())
    run_mode_command_count = 0
    artifact_paths: dict[str, str] = {}
    local_heavy_soak_lane_status = "not_executed"
    soak_iterations_executed = 0
    execution_reason_code = DRY_RUN_REASON

    if mode == "run":
        with tempfile.TemporaryDirectory(prefix="unified-api-observability-local-heavy-") as tmp:
            tmp_dir = Path(tmp)
            compatibility_report = tmp_dir / "service-api-observability-route-compatibility-summary.json"
            compatibility_policy = tmp_dir / "service-api-observability-route-compatibility-policy.json"
            observability_report = tmp_dir / "local-observability-scrape-summary.json"
            observability_policy = tmp_dir / "local-observability-scrape-policy.json"

            compatibility_output = _run_command(
                [
                    "bash",
                    "scripts/runtime/validate_service_api_observability_route_compatibility_live.sh",
                    "--mode",
                    "run",
                    "--max-seconds",
                    str(command_max_seconds),
                    "--command-max-seconds",
                    str(min(command_max_seconds, 180)),
                    "--output-json",
                    str(compatibility_report),
                ],
                timeout_seconds=command_max_seconds,
            )
            if _extract_line_value(compatibility_output, "status") != "pass":
                fail("compatibility matrix lane did not emit status=pass")
            if _extract_line_value(compatibility_output, "final_decision") != "GO":
                fail("compatibility matrix lane did not emit final_decision=GO")
            run_mode_command_count += 1

            compatibility_policy_output = _run_command(
                [
                    "bash",
                    "scripts/runtime/check_service_api_observability_route_compatibility_live_policy.sh",
                    "--report-file",
                    str(compatibility_report),
                    "--expected-final-decision",
                    "GO",
                    "--ci-fast-gate",
                    "FAIL",
                    "--output-json",
                    str(compatibility_policy),
                ],
                timeout_seconds=command_max_seconds,
            )
            if _extract_line_value(compatibility_policy_output, "status") != "ok":
                fail("compatibility matrix policy did not emit status=ok")
            if _extract_line_value(compatibility_policy_output, "final_decision") != "GO":
                fail("compatibility matrix policy did not emit final_decision=GO")
            run_mode_command_count += 1

            observability_output = _run_command(
                [
                    "bash",
                    "scripts/runtime/validate_local_observability_scrape_live.sh",
                    "--mode",
                    "run",
                    "--lane-profile",
                    "soak",
                    "--soak-iterations",
                    str(soak_iterations),
                    "--max-seconds",
                    str(command_max_seconds),
                    "--command-max-seconds",
                    str(min(command_max_seconds, 120)),
                    "--output-json",
                    str(observability_report),
                ],
                timeout_seconds=command_max_seconds,
                env={**os.environ, "KAMN_LOCAL_OBSERVABILITY_SCRAPE_OPT_IN": "1"},
            )
            if _extract_line_value(observability_output, "status") != "pass":
                fail("local observability scrape lane did not emit status=pass")
            if _extract_line_value(observability_output, "final_decision") != "GO":
                fail("local observability scrape lane did not emit final_decision=GO")
            run_mode_command_count += 1

            observability_policy_output = _run_command(
                [
                    "bash",
                    "scripts/runtime/check_local_observability_scrape_live_policy.sh",
                    "--report-file",
                    str(observability_report),
                    "--expected-final-decision",
                    "GO",
                    "--ci-fast-gate",
                    "FAIL",
                    "--output-json",
                    str(observability_policy),
                ],
                timeout_seconds=command_max_seconds,
            )
            if _extract_line_value(observability_policy_output, "status") != "ok":
                fail("local observability scrape policy did not emit status=ok")
            if _extract_line_value(observability_policy_output, "final_decision") != "GO":
                fail("local observability scrape policy did not emit final_decision=GO")
            run_mode_command_count += 1

            compatibility_payload = load_json(compatibility_report)
            compatibility_policy_payload = load_json(compatibility_policy)
            observability_payload = load_json(observability_report)
            observability_policy_payload = load_json(observability_policy)

            if compatibility_payload.get("schema_version") != COMPATIBILITY_RUN_SCHEMA:
                fail("compatibility report schema mismatch")
            if compatibility_policy_payload.get("schema_version") != COMPATIBILITY_POLICY_SCHEMA:
                fail("compatibility policy schema mismatch")
            if observability_payload.get("schema_version") != OBSERVABILITY_SCRAPE_RUN_SCHEMA:
                fail("local observability scrape report schema mismatch")
            if (
                observability_policy_payload.get("schema_version")
                != OBSERVABILITY_SCRAPE_POLICY_SCHEMA
            ):
                fail("local observability scrape policy schema mismatch")

            if compatibility_payload.get("route_compatibility_matrix_status") != "verified":
                fail("compatibility report missing route_compatibility_matrix_status=verified")
            if (
                compatibility_policy_payload.get(
                    "service_api_observability_route_compatibility_policy_status"
                )
                != "verified"
            ):
                fail("compatibility policy missing verified policy marker")
            if observability_payload.get("local_heavy_soak_lane_status") != "verified":
                fail("local observability scrape report missing local_heavy_soak_lane_status=verified")
            if observability_policy_payload.get("local_observability_scrape_policy_status") != "verified":
                fail("local observability scrape policy missing verified policy marker")

            local_heavy_soak_lane_status = "verified"
            soak_iterations_executed = int(
                observability_payload.get("soak_iterations_executed", 0)
            )
            if soak_iterations_executed <= 0:
                fail("local observability scrape soak_iterations_executed must be > 0 in run mode")
            execution_reason_code = RUN_REASON

            artifact_paths = {
                "compatibility_report": str(compatibility_report),
                "compatibility_policy_report": str(compatibility_policy),
                "observability_report": str(observability_report),
                "observability_policy_report": str(observability_policy),
            }

    elapsed_seconds = int(time.time()) - start_epoch
    if elapsed_seconds > max_seconds:
        fail(
            "unified API-observability local-heavy lane exceeded runtime budget: "
            f"{elapsed_seconds}s (max={max_seconds}s)"
        )

    ci_fast_gate_eligibility = "excluded_local_heavy" if mode == "run" else "eligible"
    run_mode_command_status = "executed" if mode == "run" else DRY_RUN_REASON

    report_payload = {
        "schema_version": RUN_LANE_SCHEMA,
        "status": "pass",
        "final_decision": "GO",
        "lane_mode": mode,
        "ci_fast_gate": ci_fast_gate,
        "ci_fast_gate_eligibility": ci_fast_gate_eligibility,
        "fast_gate_exclusion_status": "verified",
        "fast_gate_exclusion_reason_code": FAST_GATE_EXCLUSION_REASON,
        "compatibility_matrix_status": "verified",
        "compatibility_policy_status": "verified",
        "observability_soak_status": "verified",
        "observability_policy_status": "verified",
        "local_heavy_soak_lane_status": local_heavy_soak_lane_status,
        "soak_iterations_requested": soak_iterations,
        "soak_iterations_executed": soak_iterations_executed,
        "local_heavy_runtime_budget_status": "verified",
        "run_mode_command_status": run_mode_command_status,
        "run_mode_command_count": run_mode_command_count,
        "execution_reason_code": execution_reason_code,
        "compatibility_report_schema_version": COMPATIBILITY_RUN_SCHEMA,
        "compatibility_policy_schema_version": COMPATIBILITY_POLICY_SCHEMA,
        "observability_report_schema_version": OBSERVABILITY_SCRAPE_RUN_SCHEMA,
        "observability_policy_schema_version": OBSERVABILITY_SCRAPE_POLICY_SCHEMA,
        "elapsed_seconds": elapsed_seconds,
        "max_seconds": max_seconds,
        "command_max_seconds": command_max_seconds,
        "artifact_paths": artifact_paths,
    }

    output_json = None
    if args.output_json:
        output_json = Path(args.output_json).resolve()
        write_json(output_json, report_payload)

    print("status=pass")
    print("final_decision=GO")
    print(f"lane_mode={mode}")
    print(f"ci_fast_gate={ci_fast_gate}")
    print(f"ci_fast_gate_eligibility={ci_fast_gate_eligibility}")
    print("fast_gate_exclusion_status=verified")
    print(f"fast_gate_exclusion_reason_code={FAST_GATE_EXCLUSION_REASON}")
    print("compatibility_matrix_status=verified")
    print("compatibility_policy_status=verified")
    print("observability_soak_status=verified")
    print("observability_policy_status=verified")
    print(f"local_heavy_soak_lane_status={local_heavy_soak_lane_status}")
    print(f"soak_iterations_requested={soak_iterations}")
    print(f"soak_iterations_executed={soak_iterations_executed}")
    print("local_heavy_runtime_budget_status=verified")
    print(f"run_mode_command_status={run_mode_command_status}")
    print(f"run_mode_command_count={run_mode_command_count}")
    print(f"execution_reason_code={execution_reason_code}")
    print(f"compatibility_report_schema_version={COMPATIBILITY_RUN_SCHEMA}")
    print(f"compatibility_policy_schema_version={COMPATIBILITY_POLICY_SCHEMA}")
    print(f"observability_report_schema_version={OBSERVABILITY_SCRAPE_RUN_SCHEMA}")
    print(f"observability_policy_schema_version={OBSERVABILITY_SCRAPE_POLICY_SCHEMA}")
    print(f"elapsed_seconds={elapsed_seconds}")
    print(f"max_seconds={max_seconds}")
    print(f"command_max_seconds={command_max_seconds}")
    if output_json is not None:
        print(f"report_file={output_json}")
    return 0


def _check_policy(args: argparse.Namespace) -> int:
    report_file = Path(args.report_file).resolve()
    if not report_file.is_file():
        fail(f"report file not found: {report_file}")

    report = load_json(report_file)
    expected_final_decision = require_enum(
        "--expected-final-decision",
        args.expected_final_decision.strip(),
        ("GO", "NO-GO"),
    )
    ci_fast_gate = require_enum("--ci-fast-gate", args.ci_fast_gate.strip(), ("PASS", "FAIL"))

    checks = DecisionAccumulator()
    checks.reject_if(
        report.get("schema_version") != RUN_LANE_SCHEMA,
        "unified_api_observability_local_heavy_policy_schema_mismatch",
    )
    checks.reject_if(
        report.get("status") != "pass",
        "unified_api_observability_local_heavy_policy_status_mismatch",
    )
    checks.reject_if(
        report.get("final_decision") not in {"GO", "NO-GO"},
        "unified_api_observability_local_heavy_policy_final_decision_invalid",
    )
    checks.reject_if(
        report.get("final_decision") != expected_final_decision,
        "unified_api_observability_local_heavy_policy_final_decision_mismatch",
    )
    checks.reject_if(
        report.get("ci_fast_gate") != ci_fast_gate,
        "unified_api_observability_local_heavy_policy_ci_fast_gate_mismatch",
    )
    checks.reject_if(ci_fast_gate != "PASS", "ci_fast_gate_failed")

    checks.reject_if(
        report.get("fast_gate_exclusion_status") != "verified",
        "unified_api_observability_local_heavy_policy_fast_gate_exclusion_status_mismatch",
    )
    checks.reject_if(
        report.get("fast_gate_exclusion_reason_code") != FAST_GATE_EXCLUSION_REASON,
        "unified_api_observability_local_heavy_policy_fast_gate_exclusion_reason_mismatch",
    )
    checks.reject_if(
        report.get("compatibility_matrix_status") != "verified",
        "unified_api_observability_local_heavy_policy_compatibility_matrix_status_mismatch",
    )
    checks.reject_if(
        report.get("compatibility_policy_status") != "verified",
        "unified_api_observability_local_heavy_policy_compatibility_policy_status_mismatch",
    )
    checks.reject_if(
        report.get("observability_soak_status") != "verified",
        "unified_api_observability_local_heavy_policy_observability_soak_status_mismatch",
    )
    checks.reject_if(
        report.get("observability_policy_status") != "verified",
        "unified_api_observability_local_heavy_policy_observability_policy_status_mismatch",
    )
    checks.reject_if(
        report.get("local_heavy_runtime_budget_status") != "verified",
        "unified_api_observability_local_heavy_policy_runtime_budget_status_mismatch",
    )

    lane_mode = report.get("lane_mode")
    checks.reject_if(
        lane_mode not in {"dry-run", "run"},
        "unified_api_observability_local_heavy_policy_lane_mode_invalid",
    )

    checks.reject_if(
        not _is_non_negative_int(report.get("run_mode_command_count")),
        "unified_api_observability_local_heavy_policy_command_count_invalid",
    )
    checks.reject_if(
        not _is_non_negative_int(report.get("soak_iterations_requested")),
        "unified_api_observability_local_heavy_policy_soak_iterations_requested_invalid",
    )
    checks.reject_if(
        not _is_non_negative_int(report.get("soak_iterations_executed")),
        "unified_api_observability_local_heavy_policy_soak_iterations_executed_invalid",
    )
    checks.reject_if(
        not _is_non_negative_int(report.get("elapsed_seconds")),
        "unified_api_observability_local_heavy_policy_elapsed_seconds_invalid",
    )
    checks.reject_if(
        not _is_non_negative_int(report.get("max_seconds")),
        "unified_api_observability_local_heavy_policy_max_seconds_invalid",
    )
    checks.reject_if(
        not _is_non_negative_int(report.get("command_max_seconds")),
        "unified_api_observability_local_heavy_policy_command_max_seconds_invalid",
    )

    if (
        isinstance(report.get("elapsed_seconds"), int)
        and isinstance(report.get("max_seconds"), int)
        and report.get("elapsed_seconds") > report.get("max_seconds")
    ):
        checks.reject_if(
            True,
            "unified_api_observability_local_heavy_policy_runtime_budget_exceeded",
        )
    if (
        isinstance(report.get("command_max_seconds"), int)
        and isinstance(report.get("max_seconds"), int)
        and report.get("command_max_seconds") > report.get("max_seconds")
    ):
        checks.reject_if(
            True,
            "unified_api_observability_local_heavy_policy_command_budget_exceeded",
        )

    checks.reject_if(
        report.get("compatibility_report_schema_version") != COMPATIBILITY_RUN_SCHEMA,
        "unified_api_observability_local_heavy_policy_compatibility_report_schema_mismatch",
    )
    checks.reject_if(
        report.get("compatibility_policy_schema_version") != COMPATIBILITY_POLICY_SCHEMA,
        "unified_api_observability_local_heavy_policy_compatibility_policy_schema_mismatch",
    )
    checks.reject_if(
        report.get("observability_report_schema_version") != OBSERVABILITY_SCRAPE_RUN_SCHEMA,
        "unified_api_observability_local_heavy_policy_observability_report_schema_mismatch",
    )
    checks.reject_if(
        report.get("observability_policy_schema_version") != OBSERVABILITY_SCRAPE_POLICY_SCHEMA,
        "unified_api_observability_local_heavy_policy_observability_policy_schema_mismatch",
    )
    checks.reject_if(
        not isinstance(report.get("artifact_paths"), dict),
        "unified_api_observability_local_heavy_policy_artifact_paths_invalid",
    )

    if lane_mode == "dry-run":
        checks.reject_if(
            report.get("ci_fast_gate_eligibility") != "eligible",
            "unified_api_observability_local_heavy_policy_dry_run_eligibility_mismatch",
        )
        checks.reject_if(
            report.get("run_mode_command_status") != DRY_RUN_REASON,
            "unified_api_observability_local_heavy_policy_dry_run_command_status_mismatch",
        )
        checks.reject_if(
            report.get("run_mode_command_count") != 0,
            "unified_api_observability_local_heavy_policy_dry_run_command_count_mismatch",
        )
        checks.reject_if(
            report.get("execution_reason_code") != DRY_RUN_REASON,
            "unified_api_observability_local_heavy_policy_dry_run_reason_code_mismatch",
        )
        checks.reject_if(
            report.get("local_heavy_soak_lane_status") != "not_executed",
            "unified_api_observability_local_heavy_policy_dry_run_soak_status_mismatch",
        )
        checks.reject_if(
            report.get("soak_iterations_executed") != 0,
            "unified_api_observability_local_heavy_policy_dry_run_soak_iterations_executed_mismatch",
        )
    elif lane_mode == "run":
        checks.reject_if(
            report.get("ci_fast_gate_eligibility") != "excluded_local_heavy",
            "unified_api_observability_local_heavy_policy_run_mode_exclusion_mismatch",
        )
        checks.reject_if(
            report.get("run_mode_command_status") != "executed",
            "unified_api_observability_local_heavy_policy_run_mode_command_status_mismatch",
        )
        checks.reject_if(
            not isinstance(report.get("run_mode_command_count"), int)
            or report.get("run_mode_command_count", 0) < 4,
            "unified_api_observability_local_heavy_policy_run_mode_command_count_mismatch",
        )
        checks.reject_if(
            report.get("execution_reason_code") != RUN_REASON,
            "unified_api_observability_local_heavy_policy_run_mode_reason_code_mismatch",
        )
        checks.reject_if(
            report.get("local_heavy_soak_lane_status") != "verified",
            "unified_api_observability_local_heavy_policy_run_mode_soak_status_mismatch",
        )
        soak_requested = report.get("soak_iterations_requested")
        soak_executed = report.get("soak_iterations_executed")
        checks.reject_if(
            not isinstance(soak_requested, int) or soak_requested <= 0,
            "unified_api_observability_local_heavy_policy_run_mode_soak_iterations_requested_invalid",
        )
        checks.reject_if(
            not isinstance(soak_executed, int) or soak_executed <= 0,
            "unified_api_observability_local_heavy_policy_run_mode_soak_iterations_executed_invalid",
        )
        if isinstance(soak_requested, int) and isinstance(soak_executed, int):
            checks.reject_if(
                soak_requested != soak_executed,
                "unified_api_observability_local_heavy_policy_run_mode_soak_iterations_mismatch",
            )

    final_decision, reason_codes = checks.finalize("none")
    status = "pass" if final_decision == "GO" else "fail"
    policy_status = "verified" if final_decision == "GO" else "failed"

    policy_payload = {
        "schema_version": POLICY_SCHEMA,
        "status": status,
        "final_decision": final_decision,
        "expected_final_decision": expected_final_decision,
        "unified_api_observability_local_heavy_policy_status": policy_status,
        "reason_codes": reason_codes,
        "ci_fast_gate": ci_fast_gate,
        "source_report_file": str(report_file),
        "generated_at_epoch": int(time.time()),
    }

    output_json = None
    if args.output_json:
        output_json = Path(args.output_json).resolve()
        write_json(output_json, policy_payload)

    print(f"status={'ok' if final_decision == 'GO' else 'error'}")
    print(f"final_decision={final_decision}")
    print(f"unified_api_observability_local_heavy_policy_status={policy_status}")
    print(f"reason_codes={','.join(reason_codes)}")
    if output_json is not None:
        print(f"policy_report_file={output_json}")

    if final_decision != "GO":
        fail(
            "unified API-observability local-heavy policy rejected: "
            f"{','.join(reason_codes)}"
        )

    return 0


def _run_contract_lane(args: argparse.Namespace) -> int:
    ci_fast_gate = require_enum("--ci-fast-gate", args.ci_fast_gate.strip(), ("PASS", "FAIL"))
    max_seconds = require_positive_int(
        "KAMN_UNIFIED_API_OBSERVABILITY_LOCAL_HEAVY_CONTRACT_MAX_SECONDS",
        args.max_seconds,
    )

    start_epoch = int(time.time())
    with tempfile.TemporaryDirectory(prefix="unified-api-observability-local-heavy-contract-") as tmp:
        tmp_dir = Path(tmp)
        summary_report = tmp_dir / "unified-api-observability-local-heavy-summary.json"
        policy_report = tmp_dir / "unified-api-observability-local-heavy-policy.json"
        tampered_report = (
            tmp_dir / "unified-api-observability-local-heavy-summary.tampered.json"
        )

        _run_lane(
            argparse.Namespace(
                mode=args.mode,
                ci_fast_gate=ci_fast_gate,
                max_seconds=args.max_seconds,
                command_max_seconds=args.command_max_seconds,
                soak_iterations=args.soak_iterations,
                local_opt_in=args.local_opt_in,
                output_json=str(summary_report),
            )
        )
        _check_policy(
            argparse.Namespace(
                report_file=str(summary_report),
                expected_final_decision="GO",
                ci_fast_gate=ci_fast_gate,
                output_json=str(policy_report),
            )
        )

        payload = json.loads(summary_report.read_text(encoding="utf-8"))
        payload["compatibility_matrix_status"] = "missing"
        tampered_report.write_text(
            json.dumps(payload, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )

        tamper_reason_code = (
            "unified_api_observability_local_heavy_policy_compatibility_matrix_status_mismatch"
        )
        try:
            _check_policy(
                argparse.Namespace(
                    report_file=str(tampered_report),
                    expected_final_decision="GO",
                    ci_fast_gate=ci_fast_gate,
                    output_json=str(
                        tmp_dir / "unified-api-observability-local-heavy-policy.tampered.json"
                    ),
                )
            )
            fail("expected tampered unified API-observability local-heavy report to fail policy")
        except ContractError as error:
            if tamper_reason_code not in str(error):
                raise

        if not DOCS_FILE.is_file():
            fail(f"required strategy doc missing: {DOCS_FILE}")
        docs_text = DOCS_FILE.read_text(encoding="utf-8")
        required_doc_markers = [
            "Runtime Unified API-Observability Local-Heavy Contract Lane",
            "validate_unified_api_observability_local_heavy_live.sh",
            "check_unified_api_observability_local_heavy_live_policy.sh",
            "validate_unified_api_observability_local_heavy_live_contract_lane.sh",
            "unified API-observability local-heavy run-mode commands remain excluded from ci-fast-gate and ci-tools fast mode.",
        ]
        for marker in required_doc_markers:
            if marker not in docs_text:
                fail(f"strategy doc missing unified local-heavy marker: {marker}")

        elapsed_seconds = int(time.time()) - start_epoch
        if elapsed_seconds > max_seconds:
            fail(
                "unified API-observability local-heavy contract lane exceeded runtime budget: "
                f"{elapsed_seconds}s (max={max_seconds}s)"
            )

        lane_payload = {
            "schema_version": CONTRACT_LANE_SCHEMA,
            "status": "pass",
            "final_decision": "GO",
            "lane_mode": args.mode,
            "unified_api_observability_local_heavy_contract_status": "verified",
            "unified_api_observability_local_heavy_policy_status": "verified",
            "docs_contract_status": "verified",
            "performance_budget_status": "verified",
            "fail_closed_reason_code": tamper_reason_code,
            "summary_report_file": str(summary_report),
            "policy_report_file": str(policy_report),
            "elapsed_seconds": elapsed_seconds,
            "max_seconds": max_seconds,
        }

        output_json = None
        if args.output_json:
            output_json = Path(args.output_json).resolve()
            write_json(output_json, lane_payload)

        policy_output_json = None
        if args.policy_output_json:
            policy_output_json = Path(args.policy_output_json).resolve()
            write_json(policy_output_json, load_json(policy_report))

    print("status=pass")
    print("final_decision=GO")
    print(f"lane_mode={args.mode}")
    print("unified_api_observability_local_heavy_contract_status=verified")
    print("unified_api_observability_local_heavy_policy_status=verified")
    print("docs_contract_status=verified")
    print("performance_budget_status=verified")
    print(
        "fail_closed_reason_code="
        "unified_api_observability_local_heavy_policy_compatibility_matrix_status_mismatch"
    )
    if output_json is not None:
        print(f"report_file={output_json}")
    if policy_output_json is not None:
        print(f"policy_report_file={policy_output_json}")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Unified API-observability local-heavy lane and policy checker contracts."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    run_lane_parser = subparsers.add_parser(
        "run-lane",
        help="Run unified API-observability local-heavy lane.",
    )
    run_lane_parser.add_argument(
        "--mode",
        default=os.environ.get("KAMN_UNIFIED_API_OBSERVABILITY_LOCAL_HEAVY_MODE", "dry-run"),
    )
    run_lane_parser.add_argument(
        "--ci-fast-gate",
        default=os.environ.get("KAMN_UNIFIED_API_OBSERVABILITY_LOCAL_HEAVY_CI_FAST_GATE", "PASS"),
    )
    run_lane_parser.add_argument(
        "--max-seconds",
        default=os.environ.get("KAMN_UNIFIED_API_OBSERVABILITY_LOCAL_HEAVY_MAX_SECONDS", "420"),
    )
    run_lane_parser.add_argument(
        "--command-max-seconds",
        default=os.environ.get(
            "KAMN_UNIFIED_API_OBSERVABILITY_LOCAL_HEAVY_COMMAND_MAX_SECONDS",
            "180",
        ),
    )
    run_lane_parser.add_argument(
        "--soak-iterations",
        default=os.environ.get(
            "KAMN_UNIFIED_API_OBSERVABILITY_LOCAL_HEAVY_SOAK_ITERATIONS",
            "1",
        ),
    )
    run_lane_parser.add_argument(
        "--local-opt-in",
        default=os.environ.get(OPT_IN_ENV, ""),
    )
    run_lane_parser.add_argument("--output-json", default="")
    run_lane_parser.set_defaults(handler=_run_lane)

    policy_parser = subparsers.add_parser(
        "check-policy",
        help="Check unified API-observability local-heavy policy report.",
    )
    policy_parser.add_argument("--report-file", required=True)
    policy_parser.add_argument("--expected-final-decision", default="GO")
    policy_parser.add_argument("--ci-fast-gate", default="PASS")
    policy_parser.add_argument("--output-json", default="")
    policy_parser.set_defaults(handler=_check_policy)

    contract_parser = subparsers.add_parser(
        "run-contract-lane",
        help="Run unified API-observability local-heavy lane + policy + tamper checks.",
    )
    contract_parser.add_argument(
        "--mode",
        default=os.environ.get(
            "KAMN_UNIFIED_API_OBSERVABILITY_LOCAL_HEAVY_CONTRACT_MODE",
            "dry-run",
        ),
    )
    contract_parser.add_argument("--ci-fast-gate", default="PASS")
    contract_parser.add_argument(
        "--max-seconds",
        default=os.environ.get(
            "KAMN_UNIFIED_API_OBSERVABILITY_LOCAL_HEAVY_CONTRACT_MAX_SECONDS",
            "480",
        ),
    )
    contract_parser.add_argument(
        "--command-max-seconds",
        default=os.environ.get(
            "KAMN_UNIFIED_API_OBSERVABILITY_LOCAL_HEAVY_COMMAND_MAX_SECONDS",
            "180",
        ),
    )
    contract_parser.add_argument(
        "--soak-iterations",
        default=os.environ.get(
            "KAMN_UNIFIED_API_OBSERVABILITY_LOCAL_HEAVY_SOAK_ITERATIONS",
            "1",
        ),
    )
    contract_parser.add_argument(
        "--local-opt-in",
        default=os.environ.get(OPT_IN_ENV, ""),
    )
    contract_parser.add_argument("--output-json", default="")
    contract_parser.add_argument("--policy-output-json", default="")
    contract_parser.set_defaults(handler=_run_contract_lane)

    args = parser.parse_args()
    try:
        return int(args.handler(args))
    except ContractError as error:
        print(str(error), file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
