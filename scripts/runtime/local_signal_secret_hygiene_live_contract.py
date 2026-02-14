#!/usr/bin/env python3
"""Local signal + secret-hygiene live lane and policy checker contracts."""

from __future__ import annotations

import argparse
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
    require_non_negative_int,
    require_positive_int,
    write_json,
)

RUN_LANE_SCHEMA = "kamn.runtime.local-signal-secret-hygiene-live-report.v1"
POLICY_SCHEMA = "kamn.runtime.local-signal-secret-hygiene-live-policy-report.v1"
SIGNAL_REPORT_SCHEMA = "kamn.runtime.daemon-os-signal-live-validation.v1"
SECRET_REPORT_SCHEMA = "kamn.kolme.local-live-deployment-preflight-summary.v1"
SECRET_POLICY_SCHEMA = "kamn.kolme.local-live-deployment-preflight-policy-report.v1"
EXPECTED_FALLBACK_REASON_CODE = "fallback_signer_secret_present_violation"
OPT_IN_ENV = "KAMN_LOCAL_SIGNAL_SECRET_HYGIENE_OPT_IN"


def _extract_line_value(output: str, key: str) -> str:
    prefix = f"{key}="
    for line in output.splitlines():
        if line.startswith(prefix):
            return line[len(prefix) :]
    return ""


def _normalize_path(raw_path: str) -> Path:
    path = Path(raw_path)
    if not path.is_absolute():
        path = ROOT_DIR / path
    return path


def _run_command(command: list[str], *, timeout_seconds: int) -> str:
    completed = subprocess.run(
        command,
        cwd=ROOT_DIR,
        capture_output=True,
        text=True,
        check=False,
        timeout=timeout_seconds,
    )
    if completed.returncode != 0:
        detail = (completed.stderr or completed.stdout or "command failed").strip()
        fail(f"lane command failed: {' '.join(command)}: {detail}")
    return completed.stdout


def _require_marker(output: str, key: str, expected_value: str, label: str) -> None:
    actual_value = _extract_line_value(output, key)
    if actual_value != expected_value:
        fail(
            f"{label} did not emit {key}={expected_value} "
            f"(observed: {actual_value or 'missing'})"
        )


def _is_non_negative_int(value: Any) -> bool:
    return isinstance(value, int) and not isinstance(value, bool) and value >= 0


def _run_lane(args: argparse.Namespace) -> int:
    mode = require_enum("--mode", args.mode.strip(), ("dry-run", "run"))
    max_seconds = require_positive_int(
        "KAMN_LOCAL_SIGNAL_SECRET_HYGIENE_MAX_SECONDS",
        args.max_seconds,
    )
    signal_max_seconds = require_positive_int(
        "KAMN_LOCAL_SIGNAL_SECRET_HYGIENE_SIGNAL_MAX_SECONDS",
        args.signal_max_seconds,
    )
    secret_max_seconds = require_positive_int(
        "KAMN_LOCAL_SIGNAL_SECRET_HYGIENE_SECRET_MAX_SECONDS",
        args.secret_max_seconds,
    )
    ci_fast_gate = require_enum("--ci-fast-gate", args.ci_fast_gate.strip(), ("PASS", "FAIL"))
    required_approvals = require_positive_int("--required-approvals", args.required_approvals)
    received_approvals = require_non_negative_int("--received-approvals", args.received_approvals)
    signer_rotation_epoch = require_positive_int(
        "--signer-rotation-epoch",
        args.signer_rotation_epoch,
    )
    signer_previous_rotation_epoch = require_positive_int(
        "--signer-previous-rotation-epoch",
        args.signer_previous_rotation_epoch,
    )
    signer_rotation_freshness_max_delta = require_non_negative_int(
        "--signer-rotation-freshness-max-delta",
        args.signer_rotation_freshness_max_delta,
    )

    if mode == "run" and args.require_opt_in and args.local_opt_in != "1":
        fail(f"run mode requires explicit local-only opt-in via {OPT_IN_ENV}=1")

    signal_script = _normalize_path(args.signal_script)
    secret_lane_script = _normalize_path(args.secret_lane_script)
    secret_policy_script = _normalize_path(args.secret_policy_script)
    for script_path in (signal_script, secret_lane_script, secret_policy_script):
        if not script_path.is_file():
            fail(f"expected executable script not found: {script_path}")
        if not script_path.stat().st_mode & 0o111:
            fail(f"expected executable script: {script_path}")

    start_epoch = int(time.time())
    expected_secret_reason_code = (
        "dry_run_no_commands_executed" if mode == "dry-run" else "deployment_preflight_passed"
    )

    with tempfile.TemporaryDirectory(prefix="kamn-local-signal-secret-") as temp_dir:
        temp_path = Path(temp_dir)
        signal_report_file = temp_path / "signal-live-summary.json"
        secret_report_file = temp_path / "secret-hygiene-summary.json"
        secret_policy_file = temp_path / "secret-hygiene-policy.json"

        signal_output = _run_command(
            [
                "bash",
                str(signal_script),
                "--max-seconds",
                str(signal_max_seconds),
                "--output-json",
                str(signal_report_file),
            ],
            timeout_seconds=signal_max_seconds + 30,
        )
        _require_marker(signal_output, "status", "pass", "signal validation lane")
        _require_marker(signal_output, "final_decision", "GO", "signal validation lane")
        _require_marker(
            signal_output,
            "os_signal_shutdown_status",
            "verified",
            "signal validation lane",
        )
        _require_marker(
            signal_output,
            "failure_case_status",
            "verified",
            "signal validation lane",
        )

        signal_report = load_json(signal_report_file)
        if signal_report.get("schema_version") != SIGNAL_REPORT_SCHEMA:
            fail("signal validation report schema mismatch")

        secret_lane_command = [
            "bash",
            str(secret_lane_script),
            "--mode",
            mode,
            "--output-json",
            str(secret_report_file),
            "--runtime-mode",
            args.runtime_mode,
            "--signer-profile",
            args.signer_profile,
            "--max-seconds",
            str(secret_max_seconds),
            "--required-approvals",
            str(required_approvals),
            "--received-approvals",
            str(received_approvals),
            "--signer-key-source-contract-version",
            args.signer_key_source_contract_version,
            "--signer-key-source",
            args.signer_key_source,
            "--signer-rotation-epoch",
            str(signer_rotation_epoch),
            "--signer-previous-rotation-epoch",
            str(signer_previous_rotation_epoch),
            "--signer-rotation-freshness-max-delta",
            str(signer_rotation_freshness_max_delta),
        ]
        if args.quorum_evidence_file:
            secret_lane_command.extend(["--quorum-evidence-file", args.quorum_evidence_file])
        if args.custody_evidence_file:
            secret_lane_command.extend(["--custody-evidence-file", args.custody_evidence_file])
        if args.signer_provenance_file:
            secret_lane_command.extend(["--signer-provenance-file", args.signer_provenance_file])

        secret_output = _run_command(
            secret_lane_command,
            timeout_seconds=secret_max_seconds + 30,
        )
        _require_marker(secret_output, "status", "ok", "secret hygiene lane")
        _require_marker(secret_output, "lane_mode", mode, "secret hygiene lane")
        _require_marker(secret_output, "ci_fast_gate_eligible", "true", "secret hygiene lane")

        secret_report = load_json(secret_report_file)
        if secret_report.get("schema_version") != SECRET_REPORT_SCHEMA:
            fail("secret-hygiene summary report schema mismatch")
        if secret_report.get("fallback_signer_secret_present") is not False:
            fail("secret-hygiene summary indicates fallback signer secret present")
        observed_secret_reason_code = secret_report.get("reason_code")
        if observed_secret_reason_code != expected_secret_reason_code:
            fail(
                "secret-hygiene summary reason code mismatch: "
                f"expected {expected_secret_reason_code}, observed {observed_secret_reason_code}"
            )

        secret_policy_output = _run_command(
            [
                "python3",
                str(secret_policy_script),
                "--report-file",
                str(secret_report_file),
                "--expected-final-decision",
                "GO",
                "--ci-fast-gate",
                ci_fast_gate,
                "--require-reason-code",
                expected_secret_reason_code,
                "--output-json",
                str(secret_policy_file),
            ],
            timeout_seconds=secret_max_seconds + 30,
        )
        _require_marker(secret_policy_output, "status", "ok", "secret hygiene policy checker")
        _require_marker(
            secret_policy_output,
            "final_decision",
            "GO",
            "secret hygiene policy checker",
        )

        secret_policy_report = load_json(secret_policy_file)
        if secret_policy_report.get("schema_version") != SECRET_POLICY_SCHEMA:
            fail("secret-hygiene policy report schema mismatch")
        if secret_policy_report.get("final_decision") != "GO":
            fail("secret-hygiene policy report final_decision must be GO")

        elapsed_seconds = int(time.time()) - start_epoch
        if elapsed_seconds > max_seconds:
            fail(
                "local signal/secret hygiene lane exceeded runtime budget: "
                f"{elapsed_seconds}s (max={max_seconds}s)"
            )

        commands = [
            str(signal_script),
            str(secret_lane_script),
            str(secret_policy_script),
        ]
        report_payload = {
            "schema_version": RUN_LANE_SCHEMA,
            "status": "pass",
            "final_decision": "GO",
            "lane_mode": mode,
            "signal_shutdown_status": "verified",
            "signal_failure_case_status": "verified",
            "secret_hygiene_status": "verified",
            "secret_hygiene_policy_status": "verified",
            "secret_hygiene_reason_code": observed_secret_reason_code,
            "secret_hygiene_expected_reason_code": expected_secret_reason_code,
            "fallback_secret_guard_status": "verified",
            "fallback_secret_fail_closed_reason_code": EXPECTED_FALLBACK_REASON_CODE,
            "ci_fast_gate_exclusion_status": "verified",
            "performance_budget_status": "verified",
            "command_count": len(commands),
            "commands": commands,
            "elapsed_seconds": elapsed_seconds,
            "max_seconds": max_seconds,
            "ci_fast_gate": ci_fast_gate,
        }

        output_json = None
        if args.output_json:
            output_json = Path(args.output_json).resolve()
            write_json(output_json, report_payload)

    print("status=pass")
    print("final_decision=GO")
    print(f"lane_mode={mode}")
    print("signal_shutdown_status=verified")
    print("signal_failure_case_status=verified")
    print("secret_hygiene_status=verified")
    print("secret_hygiene_policy_status=verified")
    print("fallback_secret_guard_status=verified")
    print(f"fallback_secret_fail_closed_reason_code={EXPECTED_FALLBACK_REASON_CODE}")
    print("ci_fast_gate_exclusion_status=verified")
    print("performance_budget_status=verified")
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
        args.expected_final_decision,
        ("GO", "NO-GO"),
    )
    ci_fast_gate = require_enum("--ci-fast-gate", args.ci_fast_gate, ("PASS", "FAIL"))

    required_fields = [
        "schema_version",
        "status",
        "final_decision",
        "lane_mode",
        "signal_shutdown_status",
        "signal_failure_case_status",
        "secret_hygiene_status",
        "secret_hygiene_policy_status",
        "secret_hygiene_reason_code",
        "secret_hygiene_expected_reason_code",
        "fallback_secret_guard_status",
        "fallback_secret_fail_closed_reason_code",
        "ci_fast_gate_exclusion_status",
        "performance_budget_status",
        "command_count",
        "elapsed_seconds",
    ]
    missing_fields = [field_name for field_name in required_fields if field_name not in report]
    if missing_fields:
        fail(f"missing required report fields: {','.join(missing_fields)}")

    decision = DecisionAccumulator()
    decision.reject_if(
        report.get("schema_version") != RUN_LANE_SCHEMA,
        "local_signal_secret_hygiene_policy_schema_mismatch",
    )
    decision.reject_if(
        report.get("status") not in {"pass", "fail"},
        "local_signal_secret_hygiene_policy_status_invalid",
    )
    decision.reject_if(
        report.get("final_decision") not in {"GO", "NO-GO"},
        "local_signal_secret_hygiene_policy_final_decision_invalid",
    )
    decision.reject_if(
        report.get("final_decision") != expected_final_decision,
        "local_signal_secret_hygiene_policy_final_decision_mismatch",
    )

    for field_name in (
        "signal_shutdown_status",
        "signal_failure_case_status",
        "secret_hygiene_status",
        "secret_hygiene_policy_status",
        "fallback_secret_guard_status",
        "ci_fast_gate_exclusion_status",
        "performance_budget_status",
    ):
        decision.reject_if(
            report.get(field_name) != "verified",
            f"local_signal_secret_hygiene_policy_marker_missing:{field_name}",
        )

    lane_mode = report.get("lane_mode")
    decision.reject_if(
        lane_mode not in {"dry-run", "run"},
        "local_signal_secret_hygiene_policy_lane_mode_invalid",
    )

    expected_reason_code = report.get("secret_hygiene_expected_reason_code")
    observed_reason_code = report.get("secret_hygiene_reason_code")
    decision.reject_if(
        expected_reason_code not in {"dry_run_no_commands_executed", "deployment_preflight_passed"},
        "local_signal_secret_hygiene_policy_secret_reason_code_invalid",
    )
    decision.reject_if(
        observed_reason_code != expected_reason_code,
        "local_signal_secret_hygiene_policy_secret_reason_code_mismatch",
    )
    decision.reject_if(
        report.get("fallback_secret_fail_closed_reason_code") != EXPECTED_FALLBACK_REASON_CODE,
        "local_signal_secret_hygiene_policy_secret_reason_code_mismatch",
    )

    decision.reject_if(
        not _is_non_negative_int(report.get("command_count")),
        "local_signal_secret_hygiene_policy_command_count_invalid",
    )
    decision.reject_if(
        not _is_non_negative_int(report.get("elapsed_seconds")),
        "local_signal_secret_hygiene_policy_elapsed_seconds_invalid",
    )
    decision.reject_if(ci_fast_gate != "PASS", "ci_fast_gate_failed")

    final_decision, reason_codes = decision.finalize("none")
    status = "pass" if final_decision == "GO" else "fail"
    policy_status = "verified" if final_decision == "GO" else "rejected"

    policy_report = {
        "schema_version": POLICY_SCHEMA,
        "status": status,
        "final_decision": final_decision,
        "local_signal_secret_hygiene_policy_status": policy_status,
        "expected_final_decision": expected_final_decision,
        "observed_final_decision": report.get("final_decision"),
        "reason_codes": reason_codes,
        "ci_fast_gate": ci_fast_gate,
        "source_report_file": str(report_file),
        "generated_at_epoch": int(time.time()),
    }

    output_json = None
    if args.output_json:
        output_json = Path(args.output_json).resolve()
        write_json(output_json, policy_report)

    reason_codes_csv = ",".join(reason_codes)
    print(f"status={'ok' if final_decision == 'GO' else 'error'}")
    print(f"final_decision={final_decision}")
    print(f"local_signal_secret_hygiene_policy_status={policy_status}")
    print(f"failed_checks={reason_codes_csv}")
    if output_json is not None:
        print(f"policy_report_file={output_json}")

    if final_decision != "GO":
        fail(f"local signal/secret hygiene live policy rejected: {reason_codes_csv}")

    return 0


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Local signal + secret-hygiene live lane and policy checker contracts."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    run_lane_parser = subparsers.add_parser(
        "run-lane",
        help="Execute local signal + secret-hygiene lane in dry-run or run mode.",
    )
    run_lane_parser.add_argument(
        "--mode",
        default=os.environ.get("KAMN_LOCAL_SIGNAL_SECRET_HYGIENE_MODE", "dry-run"),
        help="Lane mode: dry-run|run.",
    )
    run_lane_parser.add_argument(
        "--max-seconds",
        default=os.environ.get("KAMN_LOCAL_SIGNAL_SECRET_HYGIENE_MAX_SECONDS", "240"),
        help="Maximum lane runtime budget in seconds.",
    )
    run_lane_parser.add_argument(
        "--signal-max-seconds",
        default=os.environ.get("KAMN_LOCAL_SIGNAL_SECRET_HYGIENE_SIGNAL_MAX_SECONDS", "180"),
        help="Maximum runtime budget for daemon OS-signal validation.",
    )
    run_lane_parser.add_argument(
        "--secret-max-seconds",
        default=os.environ.get("KAMN_LOCAL_SIGNAL_SECRET_HYGIENE_SECRET_MAX_SECONDS", "30"),
        help="Maximum runtime budget for secret-hygiene preflight validation.",
    )
    run_lane_parser.add_argument(
        "--ci-fast-gate",
        default="PASS",
        help="CI fast-gate marker (PASS|FAIL).",
    )
    run_lane_parser.add_argument(
        "--signal-script",
        default=str(ROOT_DIR / "scripts/runtime/validate_daemon_os_signal_live.sh"),
        help="Daemon OS-signal validation script path.",
    )
    run_lane_parser.add_argument(
        "--secret-lane-script",
        default=str(ROOT_DIR / "scripts/kolme/run_local_kolme_live_deployment_preflight_lane.sh"),
        help="Secret-hygiene preflight lane script path.",
    )
    run_lane_parser.add_argument(
        "--secret-policy-script",
        default=str(ROOT_DIR / "scripts/kolme/check_local_kolme_live_deployment_preflight_policy.py"),
        help="Secret-hygiene preflight policy script path.",
    )
    run_lane_parser.add_argument(
        "--runtime-mode",
        default="kolme-live",
        help="Runtime mode for secret-hygiene preflight checks.",
    )
    run_lane_parser.add_argument(
        "--signer-profile",
        default="ops-primary",
        help="Signer profile for secret-hygiene preflight checks.",
    )
    run_lane_parser.add_argument(
        "--required-approvals",
        default="2",
        help="Required approvals threshold for preflight checks.",
    )
    run_lane_parser.add_argument(
        "--received-approvals",
        default="2",
        help="Received approvals count for preflight checks.",
    )
    run_lane_parser.add_argument(
        "--quorum-evidence-file",
        default="",
        help="Optional quorum evidence file for run mode.",
    )
    run_lane_parser.add_argument(
        "--custody-evidence-file",
        default="",
        help="Optional custody evidence file for run mode.",
    )
    run_lane_parser.add_argument(
        "--signer-provenance-file",
        default="",
        help="Optional signer provenance file for run mode.",
    )
    run_lane_parser.add_argument(
        "--signer-key-source-contract-version",
        default="v1",
        help="Signer key-source contract version.",
    )
    run_lane_parser.add_argument(
        "--signer-key-source",
        default="managed-external",
        help="Signer key-source marker.",
    )
    run_lane_parser.add_argument(
        "--signer-rotation-epoch",
        default="1",
        help="Signer rotation epoch marker.",
    )
    run_lane_parser.add_argument(
        "--signer-previous-rotation-epoch",
        default="1",
        help="Signer previous rotation epoch marker.",
    )
    run_lane_parser.add_argument(
        "--signer-rotation-freshness-max-delta",
        default="2",
        help="Signer rotation freshness delta marker.",
    )
    run_lane_parser.add_argument(
        "--output-json",
        default="",
        help="Optional output path for summary report JSON.",
    )
    run_lane_parser.add_argument(
        "--local-opt-in",
        default=os.environ.get(OPT_IN_ENV, "0"),
        help="Opt-in marker value for run mode checks.",
    )
    run_lane_parser.add_argument(
        "--require-opt-in",
        dest="require_opt_in",
        action="store_true",
        help="Require explicit local-only run-mode opt-in.",
    )
    run_lane_parser.add_argument(
        "--no-require-opt-in",
        dest="require_opt_in",
        action="store_false",
        help="Disable explicit local-only run-mode opt-in guard.",
    )
    run_lane_parser.set_defaults(
        handler=_run_lane,
        require_opt_in=True,
    )

    check_policy_parser = subparsers.add_parser(
        "check-policy",
        help="Validate local signal + secret-hygiene report policy.",
    )
    check_policy_parser.add_argument(
        "--report-file",
        required=True,
        help="Path to local signal + secret-hygiene report JSON.",
    )
    check_policy_parser.add_argument(
        "--expected-final-decision",
        default="GO",
        help="Expected final decision marker (GO|NO-GO).",
    )
    check_policy_parser.add_argument(
        "--ci-fast-gate",
        default="PASS",
        help="CI fast-gate marker (PASS|FAIL).",
    )
    check_policy_parser.add_argument(
        "--output-json",
        default="",
        help="Optional output path for policy report JSON.",
    )
    check_policy_parser.set_defaults(handler=_check_policy)

    args = parser.parse_args()
    if hasattr(args, "max_seconds"):
        args.max_seconds = str(args.max_seconds).strip()
    if hasattr(args, "signal_max_seconds"):
        args.signal_max_seconds = str(args.signal_max_seconds).strip()
    if hasattr(args, "secret_max_seconds"):
        args.secret_max_seconds = str(args.secret_max_seconds).strip()
    if hasattr(args, "required_approvals"):
        args.required_approvals = str(args.required_approvals).strip()
    if hasattr(args, "received_approvals"):
        args.received_approvals = str(args.received_approvals).strip()
    if hasattr(args, "signer_rotation_epoch"):
        args.signer_rotation_epoch = str(args.signer_rotation_epoch).strip()
    if hasattr(args, "signer_previous_rotation_epoch"):
        args.signer_previous_rotation_epoch = str(args.signer_previous_rotation_epoch).strip()
    if hasattr(args, "signer_rotation_freshness_max_delta"):
        args.signer_rotation_freshness_max_delta = str(
            args.signer_rotation_freshness_max_delta
        ).strip()
    if hasattr(args, "mode"):
        args.mode = str(args.mode).strip()
    return args.handler(args)


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ContractError as exc:
        print(str(exc), file=sys.stderr)
        raise SystemExit(1)
