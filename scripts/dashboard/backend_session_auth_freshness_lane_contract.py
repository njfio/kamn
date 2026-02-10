#!/usr/bin/env python3
"""Dashboard backend session/auth freshness lane runner and report emitter."""

from __future__ import annotations

import os
from pathlib import Path
import re
import subprocess
import sys
import time
from typing import Any

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import ContractError, fail, write_json  # noqa: E402

ROOT_DIR = SCRIPT_DIR.parent.parent
DASHBOARD_TEST_SCRIPT = ROOT_DIR / "scripts/frontend/test_dashboard_package.sh"
BACKEND_DOC = ROOT_DIR / "docs/foundation/operator-dashboard-backend-apis.md"


def usage() -> None:
    print(
        "Usage:\n"
        "  bash scripts/dashboard/run_backend_session_auth_freshness_lane.sh \\\n"
        "    [--output-json <path>]"
    )


def _parse_args(argv: list[str]) -> Path:
    output_json = ""
    index = 0
    while index < len(argv):
        arg = argv[index]
        if arg == "--output-json":
            if index + 1 >= len(argv):
                fail("unknown argument: --output-json")
            output_json = argv[index + 1]
            index += 2
            continue
        if arg in {"--help", "-h"}:
            usage()
            raise SystemExit(0)
        fail(f"unknown argument: {arg}")

    if output_json == "":
        usage()
        fail("--output-json is required")
    return Path(output_json)


def _parse_bool_env(name: str, raw_value: str) -> bool:
    if raw_value == "true":
        return True
    if raw_value == "false":
        return False
    fail(f"invalid boolean for {name}: {raw_value}")


def _run_command(command: list[str]) -> tuple[int, str]:
    completed = subprocess.run(command, capture_output=True, text=True, check=False)
    output = completed.stdout
    if completed.stderr:
        output = f"{output}{completed.stderr}"
    return completed.returncode, output


def main(argv: list[str]) -> int:
    output_json = _parse_args(argv)

    if not DASHBOARD_TEST_SCRIPT.is_file() or not os.access(DASHBOARD_TEST_SCRIPT, os.X_OK):
        fail("expected dashboard package test script to be executable")
    if not BACKEND_DOC.is_file():
        fail("expected backend dashboard contract doc to exist")

    max_seconds_raw = os.getenv("KAMN_DASHBOARD_BACKEND_SESSION_MAX_SECONDS", "180")
    if re.fullmatch(r"[0-9]+", max_seconds_raw) is None:
        fail("KAMN_DASHBOARD_BACKEND_SESSION_MAX_SECONDS must be a non-negative integer")
    max_seconds = int(max_seconds_raw)

    skip_commands = _parse_bool_env(
        "skip_commands",
        os.getenv("KAMN_DASHBOARD_BACKEND_SESSION_SKIP_COMMANDS", "false"),
    )
    force_session_guard_missing = _parse_bool_env(
        "force_session_guard_missing",
        os.getenv("KAMN_DASHBOARD_BACKEND_SESSION_FORCE_SESSION_GUARD_MISSING", "false"),
    )
    force_freshness_guard_missing = _parse_bool_env(
        "force_freshness_guard_missing",
        os.getenv("KAMN_DASHBOARD_BACKEND_SESSION_FORCE_FRESHNESS_GUARD_MISSING", "false"),
    )
    force_docs_contract_missing = _parse_bool_env(
        "force_docs_contract_missing",
        os.getenv("KAMN_DASHBOARD_BACKEND_SESSION_FORCE_DOCS_CONTRACT_MISSING", "false"),
    )
    force_lane_failure = _parse_bool_env(
        "force_lane_failure",
        os.getenv("KAMN_DASHBOARD_BACKEND_SESSION_FORCE_LANE_FAILURE", "false"),
    )

    output_json.parent.mkdir(parents=True, exist_ok=True)
    start_epoch = int(time.time())

    commands: list[str] = []
    dashboard_output = ""
    dashboard_exit_code = 0

    if force_lane_failure:
        dashboard_exit_code = 1
    elif not skip_commands:
        commands.append("bash scripts/frontend/test_dashboard_package.sh")
        dashboard_exit_code, dashboard_output = _run_command(["bash", str(DASHBOARD_TEST_SCRIPT)])

    frontend_contract_passed = dashboard_exit_code == 0

    session_guard_passed = True
    freshness_guard_passed = True
    if not skip_commands:
        if not frontend_contract_passed:
            session_guard_passed = False
            freshness_guard_passed = False
        else:
            if "regression rejects live backend access without operator session" not in dashboard_output:
                session_guard_passed = False
            if "regression rejects expired or unauthorized session role" not in dashboard_output:
                session_guard_passed = False
            if "functional marks stale banner when snapshot age exceeds threshold" not in dashboard_output:
                freshness_guard_passed = False

    if force_session_guard_missing:
        session_guard_passed = False
    if force_freshness_guard_missing:
        freshness_guard_passed = False

    docs_contract_passed = True
    required_doc_snippets = (
        "## Backend Session/Auth Freshness Contract",
        "backend_session_auth_freshness_policy_contract.py",
        "backend_session_auth_freshness_lane_contract.py",
        "run_backend_session_auth_freshness_lane.sh",
        "check_backend_session_auth_freshness_policy.sh",
        "run_backend_session_auth_freshness_contract_lane.sh",
        "kamn.dashboard.backend-session-auth-freshness-report.v1",
        "KAMN_DASHBOARD_BACKEND_SESSION_MAX_SECONDS",
        "KAMN_DASHBOARD_BACKEND_SESSION_CONTRACT_MAX_SECONDS",
        "Regression: #941",
    )
    backend_doc_text = BACKEND_DOC.read_text(encoding="utf-8")
    for snippet in required_doc_snippets:
        if snippet not in backend_doc_text:
            docs_contract_passed = False
            break
    if force_docs_contract_missing:
        docs_contract_passed = False

    elapsed_seconds = int(time.time()) - start_epoch

    reason_codes: list[str] = []
    if not frontend_contract_passed:
        reason_codes.append("backend_lane_failed")
    if not session_guard_passed:
        reason_codes.append("session_guard_missing")
    if not freshness_guard_passed:
        reason_codes.append("freshness_guard_missing")
    if not docs_contract_passed:
        reason_codes.append("docs_contract_missing")
    if elapsed_seconds > max_seconds:
        reason_codes.append("runtime_budget_exceeded")

    if reason_codes:
        reason_codes = sorted(set(reason_codes))

    status = "pass"
    final_decision = "GO"
    if reason_codes:
        status = "fail"
        final_decision = "NO-GO"
    reason_key = f"dashboard_backend_session_auth_freshness_reason_codes:{final_decision}:v1"
    reason_codes_csv = "none" if not reason_codes else ",".join(reason_codes)

    payload: dict[str, Any] = {
        "schema_version": "kamn.dashboard.backend-session-auth-freshness-report.v1",
        "evidence_key": "dashboard_backend_session_auth_freshness:v1",
        "status": status,
        "final_decision": final_decision,
        "reason_key": reason_key,
        "elapsed_seconds": elapsed_seconds,
        "max_seconds": max_seconds,
        "skip_commands": skip_commands,
        "dashboard_package_exit_code": dashboard_exit_code,
        "command_count": len(commands),
        "commands": commands,
        "frontend_contract_passed": frontend_contract_passed,
        "session_guard_passed": session_guard_passed,
        "freshness_guard_passed": freshness_guard_passed,
        "docs_contract_passed": docs_contract_passed,
        "reason_codes": reason_codes,
    }
    write_json(output_json, payload)

    print(f"status={status}")
    print(f"final_decision={final_decision}")
    print(f"elapsed_seconds={elapsed_seconds}")
    print(f"reason_codes={reason_codes_csv}")
    print(f"reason_key={reason_key}")
    print(f"report_file={output_json}")

    if status != "pass":
        fail(f"dashboard backend session/auth freshness lane failed closed: {reason_codes_csv}")
    print("dashboard backend session/auth freshness lane tests passed.")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except ContractError as error:
        print(error, file=sys.stderr)
        raise SystemExit(1)
