#!/usr/bin/env python3
"""Governance lifecycle rollback lane runner and report emitter."""

from __future__ import annotations

from datetime import datetime, timezone
import os
from pathlib import Path
import re
import subprocess
import sys
import time

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import ContractError, fail, write_json  # noqa: E402

ROOT_DIR = SCRIPT_DIR.parent.parent
GOVERNANCE_DOC = ROOT_DIR / "docs/foundation/governance-proposal-vote-execution.md"
ROLLBACK_DOC = ROOT_DIR / "docs/foundation/upgrade-rollback-runbook.md"
LIFECYCLE_TEST = ROOT_DIR / "crates/kamn-core/tests/governance_workflow.rs"
ROLLBACK_TEST = ROOT_DIR / "crates/kamn-core/tests/upgrade_orchestration.rs"
REASON_TAXONOMY_VERSION = "kamn.governance.lifecycle-rollback-reason-taxonomy.v1"
REASON_TAXONOMY_CODES = (
    "docs_contract_missing",
    "governance_lifecycle_lane_failed",
    "lifecycle_contract_missing",
    "rollback_contract_missing",
    "rollback_gate_progress_stalled",
    "runbook_marker_parity_bypass_detected",
    "runtime_budget_exceeded",
)
REASON_TAXONOMY_CODES_CSV = ",".join(REASON_TAXONOMY_CODES)


def usage() -> None:
    print(
        "Usage:\n"
        "  bash scripts/governance/run_governance_lifecycle_rollback_lane.sh \\\n"
        "    [--output-file <path>]"
    )


def _parse_args(argv: list[str]) -> Path:
    output_file = ROOT_DIR / "governance-lifecycle-rollback-report.json"
    index = 0
    while index < len(argv):
        arg = argv[index]
        if arg == "--output-file":
            if index + 1 >= len(argv):
                fail("unknown argument: --output-file")
            output_file = Path(argv[index + 1])
            index += 2
            continue
        if arg in {"--help", "-h"}:
            usage()
            raise SystemExit(0)
        fail(f"unknown argument: {arg}")

    return output_file


def _parse_bool_env(name: str, raw_value: str) -> bool:
    if raw_value == "true":
        return True
    if raw_value == "false":
        return False
    fail(f"{name} must be true or false")


def _run_command(command: list[str]) -> int:
    completed = subprocess.run(
        command,
        check=False,
        stdout=subprocess.DEVNULL,
    )
    return completed.returncode


def main(argv: list[str]) -> int:
    output_file = _parse_args(argv)

    max_runtime_seconds_raw = os.getenv(
        "KAMN_GOVERNANCE_LIFECYCLE_ROLLBACK_MAX_SECONDS", "180"
    )
    if re.fullmatch(r"[0-9]+", max_runtime_seconds_raw) is None:
        fail("KAMN_GOVERNANCE_LIFECYCLE_ROLLBACK_MAX_SECONDS must be an integer >= 0")
    max_runtime_seconds = int(max_runtime_seconds_raw)

    skip_commands = _parse_bool_env(
        "KAMN_GOVERNANCE_LIFECYCLE_ROLLBACK_SKIP_COMMANDS",
        os.getenv("KAMN_GOVERNANCE_LIFECYCLE_ROLLBACK_SKIP_COMMANDS", "false"),
    )

    lifecycle_contract_present = LIFECYCLE_TEST.is_file()
    rollback_contract_present = ROLLBACK_TEST.is_file()
    docs_contract_present = True
    lane_failed = False
    commands: list[str] = []

    if os.getenv("KAMN_GOVERNANCE_LIFECYCLE_FORCE_LIFECYCLE_MISSING", "false") == "true":
        lifecycle_contract_present = False
    if os.getenv("KAMN_GOVERNANCE_LIFECYCLE_FORCE_ROLLBACK_MISSING", "false") == "true":
        rollback_contract_present = False

    start_epoch = int(time.time())

    if not skip_commands:
        if lifecycle_contract_present:
            command = [
                "cargo",
                "test",
                "-p",
                "kamn-core",
                "--test",
                "governance_workflow",
                "governance_workflow_functional_submit_vote_execute_flow",
            ]
            commands.append(" ".join(command))
            if _run_command(command) != 0:
                lane_failed = True

            command = [
                "cargo",
                "test",
                "-p",
                "kamn-core",
                "--test",
                "governance_workflow",
                "governance_workflow_regression_rejects_late_votes_after_deadline",
            ]
            commands.append(" ".join(command))
            if _run_command(command) != 0:
                lane_failed = True

        if rollback_contract_present:
            command = [
                "cargo",
                "test",
                "-p",
                "kamn-core",
                "--test",
                "upgrade_orchestration",
                "upgrade_orchestration_functional_activate_then_rollback_restores_version_and_audits_event",
            ]
            commands.append(" ".join(command))
            if _run_command(command) != 0:
                lane_failed = True

            command = [
                "cargo",
                "test",
                "-p",
                "kamn-core",
                "--test",
                "upgrade_orchestration",
                "upgrade_orchestration_regression_rejects_rollback_before_activation",
            ]
            commands.append(" ".join(command))
            if _run_command(command) != 0:
                lane_failed = True

    if os.getenv("KAMN_GOVERNANCE_LIFECYCLE_FORCE_LANE_FAILURE", "false") == "true":
        lane_failed = True

    required_doc_markers = (
        "governance_lifecycle_rollback_policy_contract.py",
        "governance_lifecycle_rollback_lane_contract.py",
        "run_governance_lifecycle_rollback_lane.sh",
        "check_governance_lifecycle_rollback_policy.sh",
        "run_governance_lifecycle_rollback_contract_lane.sh",
        "kamn.governance.lifecycle-rollback-report.v1",
        "kamn.governance.lifecycle-rollback-reason-taxonomy.v1",
        "governance_lifecycle_rollback_reason_codes:GO:v1",
        "governance_lifecycle_rollback_reason_codes:NO-GO:v1",
        "illegal lifecycle transitions and rollback integrity drift must fail closed (`Regression: #910`).",
        "rollback gate drift and runbook marker parity bypass acceptance must fail closed (`Regression: #4576`).",
        "rollback_gate_progress_stalled",
        "runbook_marker_parity_bypass_detected",
    )

    governance_text = GOVERNANCE_DOC.read_text(encoding="utf-8")
    rollback_text = ROLLBACK_DOC.read_text(encoding="utf-8")
    for marker in required_doc_markers:
        if marker not in governance_text or marker not in rollback_text:
            docs_contract_present = False
            break

    if os.getenv("KAMN_GOVERNANCE_LIFECYCLE_FORCE_DOCS_CONTRACT_MISSING", "false") == "true":
        docs_contract_present = False

    runtime_seconds = int(time.time()) - start_epoch
    runtime_budget_ok = runtime_seconds <= max_runtime_seconds

    decision_reasons: list[str] = []
    if lane_failed:
        decision_reasons.append("governance_lifecycle_lane_failed")
        decision_reasons.append("rollback_gate_progress_stalled")
    if not lifecycle_contract_present:
        decision_reasons.append("lifecycle_contract_missing")
    if not rollback_contract_present:
        decision_reasons.append("rollback_contract_missing")
    if not docs_contract_present:
        decision_reasons.append("docs_contract_missing")
        decision_reasons.append("runbook_marker_parity_bypass_detected")
    if not runtime_budget_ok:
        decision_reasons.append("runtime_budget_exceeded")

    final_decision = "GO" if not decision_reasons else "NO-GO"
    reason_key = f"governance_lifecycle_rollback_reason_codes:{final_decision}:v1"

    payload = {
        "schema_version": "kamn.governance.lifecycle-rollback-report.v1",
        "generated_at": datetime.now(tz=timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "max_runtime_seconds": max_runtime_seconds,
        "runtime_seconds": runtime_seconds,
        "checks": {
            "lane_failed": lane_failed,
            "lifecycle_contract_present": lifecycle_contract_present,
            "rollback_contract_present": rollback_contract_present,
            "docs_contract_present": docs_contract_present,
            "runtime_budget_ok": runtime_budget_ok,
        },
        "commands": commands,
        "reason_taxonomy_version": REASON_TAXONOMY_VERSION,
        "reason_taxonomy_codes_csv": REASON_TAXONOMY_CODES_CSV,
        "decision_reasons": decision_reasons,
        "final_decision": final_decision,
        "reason_key": reason_key,
    }
    write_json(output_file, payload)

    print("status=ok")
    print(f"output_file={output_file}")
    print(f"final_decision={final_decision}")
    print(f"reason_key={reason_key}")
    print(f"reason_taxonomy_version={REASON_TAXONOMY_VERSION}")
    print(f"reason_taxonomy_codes_csv={REASON_TAXONOMY_CODES_CSV}")
    print(f"runtime_seconds={runtime_seconds}")
    print(f"max_runtime_seconds={max_runtime_seconds}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except ContractError as error:
        print(error, file=sys.stderr)
        raise SystemExit(1)
