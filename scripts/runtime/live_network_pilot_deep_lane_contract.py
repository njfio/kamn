#!/usr/bin/env python3
"""Live-network pilot deep lane contract runner."""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time

SCRIPT_DIR = Path(__file__).resolve().parent
ROOT_DIR = SCRIPT_DIR.parent.parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import ContractError, fail, load_json  # noqa: E402


def _extract_line_value(output: str, key: str) -> str:
    prefix = f"{key}="
    for line in output.splitlines():
        if line.startswith(prefix):
            return line[len(prefix) :]
    return ""


def _load_json_field(path: Path, key: str, default: str | int) -> str | int:
    payload = load_json(path)
    return payload.get(key, default)


def _is_executable(path: Path) -> bool:
    return path.is_file() and os.access(path, os.X_OK)


def _require_bool_flag(name: str, raw_value: str) -> str:
    if raw_value not in {"true", "false"}:
        fail(f"{name} must be true or false")
    return raw_value


def run_live_network_pilot_deep_lane(args: argparse.Namespace) -> int:
    if not args.max_seconds.isdigit():
        fail("--max-seconds must be a non-negative integer")
    max_seconds = int(args.max_seconds)

    event_name = args.event_name
    if event_name not in {"schedule", "workflow_dispatch"}:
        fail("scheduled/manual-only cadence policy requires event schedule or workflow_dispatch")
    cadence = "scheduled" if event_name == "schedule" else "manual"
    smoke_skip_commands = _require_bool_flag(
        "KAMN_LIVE_NETWORK_PILOT_DEEP_SMOKE_SKIP_COMMANDS",
        os.environ.get("KAMN_LIVE_NETWORK_PILOT_DEEP_SMOKE_SKIP_COMMANDS", "true"),
    )

    smoke_lane = ROOT_DIR / "scripts/runtime/run_live_network_smoke_lane.sh"
    failover_suite = ROOT_DIR / "scripts/runtime/run_failover_sync_drill_suite.sh"
    summary_generator = ROOT_DIR / "scripts/runtime/generate_live_network_pilot_artifact_summary.sh"
    summary_checker = ROOT_DIR / "scripts/runtime/check_live_network_pilot_artifact_summary_policy.sh"

    if not all(
        _is_executable(dependency)
        for dependency in (smoke_lane, failover_suite, summary_generator, summary_checker)
    ):
        fail("expected live-network deep lane dependencies to be executable")

    output_json = Path(args.output_json)
    output_json.parent.mkdir(parents=True, exist_ok=True)

    smoke_report_handle = tempfile.NamedTemporaryFile(delete=False)
    smoke_report_handle.close()
    failover_report_handle = tempfile.NamedTemporaryFile(delete=False)
    failover_report_handle.close()
    smoke_report = Path(smoke_report_handle.name)
    failover_report = Path(failover_report_handle.name)

    start_epoch = int(time.time())
    try:
        smoke_run = subprocess.run(
            ["bash", str(smoke_lane), "--output-json", str(smoke_report)],
            env={
                **os.environ,
                "KAMN_LIVE_NETWORK_SMOKE_SKIP_COMMANDS": smoke_skip_commands,
            },
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=False,
            text=True,
        )
        smoke_code = smoke_run.returncode

        if not smoke_report.is_file() or smoke_report.stat().st_size == 0:
            smoke_status = "fail"
            smoke_decision = "NO-GO"
            smoke_elapsed_seconds = 0
        else:
            smoke_status = str(_load_json_field(smoke_report, "status", "fail"))
            smoke_decision = str(_load_json_field(smoke_report, "final_decision", "NO-GO"))
            smoke_elapsed_seconds = int(_load_json_field(smoke_report, "elapsed_seconds", 0))

        failover_args = [
            "bash",
            str(failover_suite),
            "--event-name",
            "schedule",
            "--output-json",
            str(failover_report),
        ]
        if args.skip_suite:
            failover_args.append("--skip-suite")

        failover_run = subprocess.run(
            failover_args,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=False,
            text=True,
        )
        failover_code = failover_run.returncode

        if not failover_report.is_file() or failover_report.stat().st_size == 0:
            failover_status = "fail"
        else:
            failover_status = str(_load_json_field(failover_report, "status", "fail"))

        elapsed_seconds = int(time.time()) - start_epoch
        budget_status = "within" if elapsed_seconds <= max_seconds else "exceeded"

        evidence_complete = (
            smoke_report.is_file()
            and smoke_report.stat().st_size > 0
            and failover_report.is_file()
            and failover_report.stat().st_size > 0
        )

        deep_status = "pass"
        deep_decision = "GO"
        if (
            smoke_code != 0
            or failover_code != 0
            or failover_status != "pass"
            or budget_status != "within"
        ):
            deep_status = "fail"
            deep_decision = "NO-GO"

        summary_run = subprocess.run(
            [
                "bash",
                str(summary_generator),
                "--output-file",
                str(output_json),
                "--event-name",
                event_name,
                "--cadence",
                cadence,
                "--smoke-status",
                smoke_status,
                "--smoke-decision",
                smoke_decision,
                "--smoke-elapsed-seconds",
                str(smoke_elapsed_seconds),
                "--deep-status",
                deep_status,
                "--deep-decision",
                deep_decision,
                "--deep-elapsed-seconds",
                str(elapsed_seconds),
                "--budget-status",
                budget_status,
                "--evidence-complete",
                "true" if evidence_complete else "false",
                "--ci-fast-gate",
                "PASS",
            ],
            capture_output=True,
            check=False,
            text=True,
        )
        if summary_run.returncode != 0:
            fail((summary_run.stderr or summary_run.stdout or "summary generation failed").strip())

        if "status=generated" not in summary_run.stdout.splitlines():
            fail("expected live-network pilot artifact summary generator to produce status=generated")

        check_run = subprocess.run(
            ["bash", str(summary_checker), "--summary-file", str(output_json)],
            capture_output=True,
            check=False,
            text=True,
        )
        if check_run.returncode != 0:
            fail((check_run.stderr or check_run.stdout or "summary checker failed").strip())

        final_decision = _extract_line_value(check_run.stdout, "final_decision")
        if not final_decision:
            fail("live-network pilot summary checker did not emit final_decision")
        failed_checks = _extract_line_value(check_run.stdout, "failed_checks")
        if final_decision != "GO":
            detail = ""
            if failed_checks:
                detail = f"; failed_checks={failed_checks}"
            fail(f"live-network pilot deep lane produced final_decision={final_decision}{detail}")

        print("live-network pilot deep lane tests passed.")
        return 0
    finally:
        smoke_report.unlink(missing_ok=True)
        failover_report.unlink(missing_ok=True)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Run the live-network pilot deep lane.")
    parser.add_argument("--event-name", default=os.environ.get("GITHUB_EVENT_NAME", "schedule"))
    parser.add_argument("--output-json", default=str(ROOT_DIR / "live-network-pilot-report.json"))
    parser.add_argument(
        "--max-seconds",
        default=os.environ.get("KAMN_LIVE_NETWORK_PILOT_DEEP_MAX_SECONDS", "300"),
    )
    parser.add_argument("--skip-suite", action="store_true")
    parser.set_defaults(handler=run_live_network_pilot_deep_lane)
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
