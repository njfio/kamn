#!/usr/bin/env python3
"""Deployment SLO rollback lane runner and report emitter."""

from __future__ import annotations

import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import time
from typing import Any

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import ContractError, fail, write_json  # noqa: E402

ROOT_DIR = SCRIPT_DIR.parent.parent
GENERATOR = ROOT_DIR / "scripts/deploy/generate_dr_evidence_bundle.sh"
SLO_CHECKER = ROOT_DIR / "scripts/deploy/check_release_slo_gates.sh"
RUNBOOK_DOC = ROOT_DIR / "docs/foundation/upgrade-rollback-runbook.md"


def usage() -> None:
    print(
        "Usage:\n"
        "  bash scripts/deploy/run_deployment_slo_rollback_lane.sh \\\n"
        "    --output-json <path>"
    )


def _extract_value(output: str, key: str) -> str:
    for line in output.splitlines():
        if "=" not in line:
            continue
        candidate_key, value = line.split("=", 1)
        if candidate_key == key:
            return value
    return ""


def _run_command(command: list[str]) -> tuple[int, str]:
    completed = subprocess.run(command, capture_output=True, text=True, check=False)
    output = completed.stdout
    if completed.stderr:
        output = f"{output}{completed.stderr}"
    return completed.returncode, output


def _parse_bool_env(name: str, raw_value: str) -> bool:
    if raw_value == "true":
        return True
    if raw_value == "false":
        return False
    fail(f"invalid boolean for {name}: {raw_value}")


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


def _read_required_doc_snippets() -> list[str]:
    return [
        "## Deployment SLO Evidence and Rollback Automation Contract",
        "run_deployment_slo_rollback_lane.sh",
        "check_deployment_slo_rollback_policy.sh",
        "run_deployment_slo_rollback_contract_lane.sh",
        "kamn.deploy.slo-rollback-report.v1",
        "KAMN_DEPLOYMENT_SLO_ROLLBACK_MAX_SECONDS",
        "KAMN_DEPLOYMENT_SLO_ROLLBACK_CONTRACT_MAX_SECONDS",
        "Regression: #944",
    ]


def main(argv: list[str]) -> int:
    output_json = _parse_args(argv)

    if not GENERATOR.is_file() or not os.access(GENERATOR, os.X_OK):
        fail("expected deployment DR evidence generator to be executable")
    if not SLO_CHECKER.is_file() or not os.access(SLO_CHECKER, os.X_OK):
        fail("expected deployment SLO gate checker to be executable")
    if not RUNBOOK_DOC.is_file():
        fail("expected upgrade rollback runbook doc to exist")

    max_seconds_raw = os.getenv("KAMN_DEPLOYMENT_SLO_ROLLBACK_MAX_SECONDS", "180")
    if re.fullmatch(r"[0-9]+", max_seconds_raw) is None:
        fail("KAMN_DEPLOYMENT_SLO_ROLLBACK_MAX_SECONDS must be a non-negative integer")
    max_seconds = int(max_seconds_raw)

    skip_commands = _parse_bool_env(
        "skip_commands",
        os.getenv("KAMN_DEPLOYMENT_SLO_ROLLBACK_SKIP_COMMANDS", "false"),
    )
    force_rollback_automation_missing = _parse_bool_env(
        "force_rollback_automation_missing",
        os.getenv("KAMN_DEPLOYMENT_SLO_ROLLBACK_FORCE_ROLLBACK_AUTOMATION_MISSING", "false"),
    )
    force_slo_gate_missing = _parse_bool_env(
        "force_slo_gate_missing",
        os.getenv("KAMN_DEPLOYMENT_SLO_ROLLBACK_FORCE_SLO_GATE_MISSING", "false"),
    )
    force_docs_contract_missing = _parse_bool_env(
        "force_docs_contract_missing",
        os.getenv("KAMN_DEPLOYMENT_SLO_ROLLBACK_FORCE_DOCS_CONTRACT_MISSING", "false"),
    )
    force_lane_failure = _parse_bool_env(
        "force_lane_failure",
        os.getenv("KAMN_DEPLOYMENT_SLO_ROLLBACK_FORCE_LANE_FAILURE", "false"),
    )

    start_epoch = int(time.time())
    commands: list[str] = []

    generator_output = ""
    policy_output = ""
    generator_exit_code = 0
    policy_exit_code = 0
    deployment_lane_passed = True

    demo_drill_id = "dr-rollback-contract-2026-02-09"
    recovery_rto_seconds = 240
    recovery_rpo_seconds = 90
    max_rto_seconds = 300
    max_rpo_seconds = 120
    rollback_restored = "true"
    evidence_complete = "true"
    ci_fast_gate = "PASS"

    with tempfile.TemporaryDirectory() as temp_dir:
        bundle_file = Path(temp_dir) / "deployment-slo-rollback-dr-evidence.json"

        if force_lane_failure:
            deployment_lane_passed = False
            generator_exit_code = 1
            policy_exit_code = 1
        elif not skip_commands:
            commands.append(
                "bash scripts/deploy/generate_dr_evidence_bundle.sh "
                f"--output-file {bundle_file} "
                f"--drill-id {demo_drill_id} "
                f"--recovery-rto-seconds {recovery_rto_seconds} "
                f"--recovery-rpo-seconds {recovery_rpo_seconds} "
                f"--max-rto-seconds {max_rto_seconds} "
                f"--max-rpo-seconds {max_rpo_seconds} "
                f"--rollback-restored {rollback_restored} "
                f"--evidence-complete {evidence_complete} "
                f"--ci-fast-gate {ci_fast_gate}"
            )
            generator_exit_code, generator_output = _run_command(
                [
                    "bash",
                    str(GENERATOR),
                    "--output-file",
                    str(bundle_file),
                    "--drill-id",
                    demo_drill_id,
                    "--recovery-rto-seconds",
                    str(recovery_rto_seconds),
                    "--recovery-rpo-seconds",
                    str(recovery_rpo_seconds),
                    "--max-rto-seconds",
                    str(max_rto_seconds),
                    "--max-rpo-seconds",
                    str(max_rpo_seconds),
                    "--rollback-restored",
                    rollback_restored,
                    "--evidence-complete",
                    evidence_complete,
                    "--ci-fast-gate",
                    ci_fast_gate,
                ]
            )

            if generator_exit_code == 0:
                commands.append(
                    "bash scripts/deploy/check_release_slo_gates.sh "
                    f"--bundle-file {bundle_file}"
                )
                policy_exit_code, policy_output = _run_command(
                    ["bash", str(SLO_CHECKER), "--bundle-file", str(bundle_file)]
                )
            else:
                policy_exit_code = 1

            if generator_exit_code != 0 or policy_exit_code != 0:
                deployment_lane_passed = False

        dr_bundle_final_decision = "unknown"
        if not skip_commands and generator_output:
            maybe_dr_decision = _extract_value(generator_output, "final_decision")
            if maybe_dr_decision:
                dr_bundle_final_decision = maybe_dr_decision

        policy_final_decision = "unknown"
        if not skip_commands and policy_output:
            maybe_policy_decision = _extract_value(policy_output, "final_decision")
            if maybe_policy_decision:
                policy_final_decision = maybe_policy_decision

        slo_gate_passed = True
        if not skip_commands and (not deployment_lane_passed or policy_final_decision != "GO"):
            slo_gate_passed = False

        rollback_automation_passed = True
        if not skip_commands:
            if not deployment_lane_passed:
                rollback_automation_passed = False
            else:
                try:
                    payload = json.loads(bundle_file.read_text(encoding="utf-8"))
                    dr_evidence = payload.get("dr_evidence", {})
                    if dr_evidence.get("rollback_restored") is not True:
                        rollback_automation_passed = False
                    if dr_evidence.get("evidence_complete") is not True:
                        rollback_automation_passed = False
                except (OSError, json.JSONDecodeError):
                    rollback_automation_passed = False

        if force_rollback_automation_missing:
            rollback_automation_passed = False
        if force_slo_gate_missing:
            slo_gate_passed = False

        docs_contract_passed = True
        runbook_text = RUNBOOK_DOC.read_text(encoding="utf-8")
        for snippet in _read_required_doc_snippets():
            if snippet not in runbook_text:
                docs_contract_passed = False
                break
        if force_docs_contract_missing:
            docs_contract_passed = False

        elapsed_seconds = int(time.time()) - start_epoch

        reason_codes: list[str] = []
        if not deployment_lane_passed:
            reason_codes.append("deployment_lane_failed")
        if not slo_gate_passed:
            reason_codes.append("slo_gate_missing")
        if not rollback_automation_passed:
            reason_codes.append("rollback_automation_missing")
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
        reason_key = f"deployment_slo_rollback_reason_codes:{final_decision}:v1"
        reason_codes_csv = "none" if not reason_codes else ",".join(reason_codes)

        payload: dict[str, Any] = {
            "schema_version": "kamn.deploy.slo-rollback-report.v1",
            "evidence_key": "deployment_slo_rollback:v1",
            "status": status,
            "final_decision": final_decision,
            "reason_key": reason_key,
            "elapsed_seconds": elapsed_seconds,
            "max_seconds": max_seconds,
            "skip_commands": skip_commands,
            "dr_bundle_file": str(bundle_file),
            "generator_exit_code": generator_exit_code,
            "policy_exit_code": policy_exit_code,
            "dr_bundle_final_decision": dr_bundle_final_decision,
            "policy_final_decision": policy_final_decision,
            "deployment_lane_passed": deployment_lane_passed,
            "slo_gate_passed": slo_gate_passed,
            "rollback_automation_passed": rollback_automation_passed,
            "docs_contract_passed": docs_contract_passed,
            "command_count": len(commands),
            "commands": commands,
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
        fail(f"deployment slo/rollback lane failed closed: {reason_codes_csv}")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except ContractError as error:
        print(error, file=sys.stderr)
        raise SystemExit(1)
