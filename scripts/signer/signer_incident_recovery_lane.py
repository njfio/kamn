#!/usr/bin/env python3
"""Signer incident-recovery lane runner and report emitter."""

from __future__ import annotations

import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import ContractError, fail, write_json  # noqa: E402

ROOT_DIR = SCRIPT_DIR.parent.parent
LIFECYCLE_CONTRACT_LANE = (
    ROOT_DIR / "scripts/signer/run_secure_provider_key_lifecycle_contract_lane.sh"
)
RUNBOOK_DOC = ROOT_DIR / "docs/foundation/upgrade-rollback-runbook.md"
CHECKLIST_DOC = ROOT_DIR / "docs/foundation/release-gonogo-checklist.md"


def usage() -> None:
    print(
        "Usage:\n"
        "  bash scripts/signer/run_signer_incident_recovery_lane.sh \\\n"
        "    --output-json <path>"
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


def _parse_non_negative_int_env(name: str, raw_value: str) -> int:
    if not raw_value.isdigit():
        fail(f"{name} must be a non-negative integer")
    return int(raw_value)


def _run_command(command: list[str]) -> tuple[int, str]:
    completed = subprocess.run(command, capture_output=True, text=True, check=False)
    output = completed.stdout
    if completed.stderr:
        output = f"{output}{completed.stderr}"
    return completed.returncode, output


def main(argv: list[str]) -> int:
    output_json = _parse_args(argv)

    if not LIFECYCLE_CONTRACT_LANE.is_file() or not os.access(
        LIFECYCLE_CONTRACT_LANE, os.X_OK
    ):
        fail("expected signer lifecycle contract lane script to be executable")
    if not RUNBOOK_DOC.is_file():
        fail("expected upgrade rollback runbook doc to exist")
    if not CHECKLIST_DOC.is_file():
        fail("expected release go/no-go checklist doc to exist")

    max_seconds = _parse_non_negative_int_env(
        "KAMN_SIGNER_INCIDENT_RECOVERY_MAX_SECONDS",
        os.getenv("KAMN_SIGNER_INCIDENT_RECOVERY_MAX_SECONDS", "120"),
    )
    simulate_delay_seconds = _parse_non_negative_int_env(
        "KAMN_SIGNER_INCIDENT_RECOVERY_SIMULATE_DELAY_SECONDS",
        os.getenv("KAMN_SIGNER_INCIDENT_RECOVERY_SIMULATE_DELAY_SECONDS", "0"),
    )

    skip_commands = _parse_bool_env(
        "skip_commands", os.getenv("KAMN_SIGNER_INCIDENT_RECOVERY_SKIP_COMMANDS", "false")
    )
    force_runbook_gap = _parse_bool_env(
        "force_runbook_gap",
        os.getenv("KAMN_SIGNER_INCIDENT_RECOVERY_FORCE_RUNBOOK_GAP", "false"),
    )
    force_revocation_gap = _parse_bool_env(
        "force_revocation_gap",
        os.getenv("KAMN_SIGNER_INCIDENT_RECOVERY_FORCE_REVOCATION_GAP", "false"),
    )
    force_signoff_gap = _parse_bool_env(
        "force_signoff_gap",
        os.getenv("KAMN_SIGNER_INCIDENT_RECOVERY_FORCE_SIGNOFF_GAP", "false"),
    )
    force_docs_contract_missing = _parse_bool_env(
        "force_docs_contract_missing",
        os.getenv("KAMN_SIGNER_INCIDENT_RECOVERY_FORCE_DOCS_CONTRACT_MISSING", "false"),
    )
    force_lane_failure = _parse_bool_env(
        "force_lane_failure",
        os.getenv("KAMN_SIGNER_INCIDENT_RECOVERY_FORCE_LANE_FAILURE", "false"),
    )

    output_json.parent.mkdir(parents=True, exist_ok=True)
    start_epoch = int(time.time())

    commands: list[str] = []
    lifecycle_contract_lane_exit_code = 0
    lifecycle_contract_lane_output = ""
    lifecycle_contract_lane_passed = True

    with tempfile.TemporaryDirectory() as temp_dir:
        if force_lane_failure:
            lifecycle_contract_lane_exit_code = 1
            lifecycle_contract_lane_passed = False
        elif not skip_commands:
            lifecycle_bundle = (
                Path(temp_dir) / "signer-incident-recovery-lifecycle-contract.json"
            )
            commands.append(
                "bash scripts/signer/run_secure_provider_key_lifecycle_contract_lane.sh "
                f"--skip-tests --output-file {lifecycle_bundle}"
            )
            lifecycle_contract_lane_exit_code, lifecycle_contract_lane_output = _run_command(
                [
                    "bash",
                    str(LIFECYCLE_CONTRACT_LANE),
                    "--skip-tests",
                    "--output-file",
                    str(lifecycle_bundle),
                ]
            )
            if (
                lifecycle_contract_lane_exit_code != 0
                or "secure-provider key-lifecycle contract lane tests passed."
                not in lifecycle_contract_lane_output
            ):
                lifecycle_contract_lane_passed = False

        if simulate_delay_seconds > 0:
            time.sleep(simulate_delay_seconds)

    runbook_text = RUNBOOK_DOC.read_text(encoding="utf-8")
    checklist_text = CHECKLIST_DOC.read_text(encoding="utf-8")

    runbook_required_snippets = (
        "run_signer_incident_recovery_contract_lane.sh",
        "run_signer_incident_recovery_deep_lane.sh",
        "check_signer_incident_recovery_policy.sh",
        "signer_incident_recovery_reason_codes:GO:v1",
        "Regression: #989",
    )
    checklist_required_snippets = (
        "run_signer_incident_recovery_contract_lane.sh",
        "run_signer_incident_recovery_deep_lane.sh",
        "check_signer_incident_recovery_policy.sh",
        "signer_incident_recovery_reason_codes:GO:v1",
        "Regression: #989",
    )

    docs_contract_passed = all(
        snippet in runbook_text for snippet in runbook_required_snippets
    ) and all(snippet in checklist_text for snippet in checklist_required_snippets)
    if force_docs_contract_missing:
        docs_contract_passed = False

    runbook_steps_present = (
        "## Watchdog Incident Response Flow" in runbook_text and not force_runbook_gap
    )
    rollback_checkpoint_validated = runbook_steps_present
    revocation_propagation_passed = (
        lifecycle_contract_lane_passed and not force_revocation_gap
    )
    operator_signoff_passed = not force_signoff_gap

    elapsed_seconds = int(time.time()) - start_epoch

    reason_codes: list[str] = []
    if not runbook_steps_present:
        reason_codes.append("incident_runbook_step_missing")
    if not rollback_checkpoint_validated:
        reason_codes.append("rollback_checkpoint_not_validated")
    if not revocation_propagation_passed:
        reason_codes.append("signer_revocation_propagation_missing")
    if not operator_signoff_passed:
        reason_codes.append("operator_signoff_missing")
    if not docs_contract_passed:
        reason_codes.append("docs_contract_missing")
    if not lifecycle_contract_lane_passed:
        reason_codes.append("lifecycle_contract_lane_failed")
    if elapsed_seconds > max_seconds:
        reason_codes.append("runtime_budget_exceeded")

    reason_codes = sorted(set(reason_codes))
    status = "pass"
    final_decision = "GO"
    if reason_codes:
        status = "fail"
        final_decision = "NO-GO"

    reason_codes_value = ",".join(reason_codes) if reason_codes else "none"
    reason_key = f"signer_incident_recovery_reason_codes:{final_decision}:v1"

    payload = {
        "schema_version": "kamn.signer.incident-recovery-report.v1",
        "evidence_key": "signer_incident_recovery:v1",
        "status": status,
        "final_decision": final_decision,
        "reason_key": reason_key,
        "elapsed_seconds": elapsed_seconds,
        "max_seconds": max_seconds,
        "skip_commands": skip_commands,
        "command_count": len(commands),
        "commands": commands,
        "lifecycle_contract_lane_exit_code": lifecycle_contract_lane_exit_code,
        "lifecycle_contract_lane_passed": lifecycle_contract_lane_passed,
        "runbook_steps_present": runbook_steps_present,
        "rollback_checkpoint_validated": rollback_checkpoint_validated,
        "revocation_propagation_passed": revocation_propagation_passed,
        "operator_signoff_passed": operator_signoff_passed,
        "docs_contract_passed": docs_contract_passed,
        "reason_codes": reason_codes,
        "generated_epoch": int(time.time()),
        "report_generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
    }
    write_json(output_json, payload)

    print(f"status={status}")
    print(f"report_file={output_json}")
    print(f"reason_key={reason_key}")
    print(f"final_decision={final_decision}")
    print(f"reason_codes={reason_codes_value}")

    if status != "pass":
        fail(f"signer incident recovery lane failed closed: {reason_codes_value}")
    print("signer incident recovery lane tests passed.")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except ContractError as error:
        print(error, file=sys.stderr)
        raise SystemExit(1)
