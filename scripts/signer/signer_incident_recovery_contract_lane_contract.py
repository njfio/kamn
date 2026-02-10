#!/usr/bin/env python3
"""Signer incident-recovery contract-lane runner."""

from __future__ import annotations

import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time


def usage() -> None:
    print(
        "Usage:\n"
        "  bash scripts/signer/run_signer_incident_recovery_contract_lane.sh \\\n"
        "    [--output-file <path>]"
    )


def fail(message: str) -> int:
    print(message, file=sys.stderr)
    return 1


def run_capture(command: list[str], *, env: dict[str, str] | None = None) -> tuple[int, str]:
    result = subprocess.run(
        command,
        capture_output=True,
        text=True,
        check=False,
        env=env,
    )
    output = (result.stdout or "") + (result.stderr or "")
    return result.returncode, output


def parse_args(argv: list[str]) -> tuple[int, str]:
    output_file = ""
    index = 0
    while index < len(argv):
        arg = argv[index]
        if arg == "--output-file":
            if index + 1 >= len(argv):
                return fail("unknown argument: --output-file"), output_file
            output_file = argv[index + 1]
            index += 2
            continue
        if arg in {"--help", "-h"}:
            usage()
            return 0, output_file
        return fail(f"unknown argument: {arg}"), output_file
    return 200, output_file


def main(argv: list[str]) -> int:
    parse_status, output_file_arg = parse_args(argv)
    if parse_status != 200:
        return parse_status

    root_dir = Path(__file__).resolve().parents[2]
    lane_script = root_dir / "scripts/signer/run_signer_incident_recovery_lane.sh"
    checker = root_dir / "scripts/signer/check_signer_incident_recovery_policy.sh"
    runbook_doc = root_dir / "docs/foundation/upgrade-rollback-runbook.md"
    checklist_doc = root_dir / "docs/foundation/release-gonogo-checklist.md"

    if not lane_script.is_file() or not os.access(lane_script, os.X_OK):
        return fail("expected signer incident recovery lane script to be executable")
    if not checker.is_file() or not os.access(checker, os.X_OK):
        return fail("expected signer incident recovery policy checker to be executable")
    if not runbook_doc.is_file():
        return fail("expected upgrade rollback runbook doc to exist")
    if not checklist_doc.is_file():
        return fail("expected release go/no-go checklist doc to exist")

    max_contract_seconds_raw = os.getenv(
        "KAMN_SIGNER_INCIDENT_RECOVERY_CONTRACT_MAX_SECONDS", "240"
    )
    if not max_contract_seconds_raw.isdigit() or int(max_contract_seconds_raw) <= 0:
        return fail(
            "KAMN_SIGNER_INCIDENT_RECOVERY_CONTRACT_MAX_SECONDS must be a positive integer"
        )
    max_contract_seconds = int(max_contract_seconds_raw)
    start_epoch = int(time.time())

    with tempfile.TemporaryDirectory() as temp_dir:
        if output_file_arg:
            output_file = Path(output_file_arg)
        else:
            output_file = Path(temp_dir) / "signer-incident-recovery-contract-report.json"

        go_env = os.environ.copy()
        go_env["KAMN_SIGNER_INCIDENT_RECOVERY_MAX_SECONDS"] = str(max_contract_seconds)
        go_code, go_output = run_capture(
            ["bash", str(lane_script), "--output-json", str(output_file)],
            env=go_env,
        )
        if go_code != 0 or "status=pass" not in go_output:
            return fail("expected signer incident recovery lane GO run to report pass status")
        if "final_decision=GO" not in go_output:
            return fail("expected signer incident recovery lane GO run to report GO decision")
        if (
            "reason_key=signer_incident_recovery_reason_codes:GO:v1"
            not in go_output
        ):
            return fail(
                "expected signer incident recovery lane GO run to emit deterministic GO reason key"
            )

        go_policy_code, go_policy_output = run_capture(
            ["bash", str(checker), "--report-file", str(output_file)]
        )
        if go_policy_code != 0 or "status=ok" not in go_policy_output:
            return fail("expected signer incident recovery policy checker status marker for GO report")
        if "final_decision=GO" not in go_policy_output:
            return fail("expected signer incident recovery policy checker GO decision for GO report")
        if "failed_checks=none" not in go_policy_output:
            return fail(
                "expected signer incident recovery policy checker no failed checks for GO report"
            )

        no_go_report = Path(temp_dir) / "signer-incident-recovery-no-go.json"
        no_go_env = os.environ.copy()
        no_go_env["KAMN_SIGNER_INCIDENT_RECOVERY_SKIP_COMMANDS"] = "true"
        no_go_env["KAMN_SIGNER_INCIDENT_RECOVERY_FORCE_RUNBOOK_GAP"] = "true"
        no_go_code, no_go_output = run_capture(
            ["bash", str(lane_script), "--output-json", str(no_go_report)],
            env=no_go_env,
        )
        if no_go_code == 0:
            return fail(
                "expected forced runbook-gap signer incident recovery lane run to fail closed"
            )
        if "incident_runbook_step_missing" not in no_go_output:
            return fail(
                "expected forced runbook-gap lane run to emit incident_runbook_step_missing reason code"
            )

        no_go_policy_code, no_go_policy_output = run_capture(
            ["bash", str(checker), "--report-file", str(no_go_report)]
        )
        if no_go_policy_code != 0 or "final_decision=NO-GO" not in no_go_policy_output:
            return fail(
                "expected signer incident recovery policy checker NO-GO decision for runbook-gap report"
            )
        if "incident_runbook_step_missing" not in no_go_policy_output:
            return fail(
                "expected signer incident recovery policy checker failed checks to include incident_runbook_step_missing"
            )

        tampered_report = Path(temp_dir) / "signer-incident-recovery-no-go-tampered.json"
        tampered_payload = json.loads(no_go_report.read_text(encoding="utf-8"))
        tampered_payload["final_decision"] = "GO"
        tampered_payload["reason_key"] = "signer_incident_recovery_reason_codes:GO:v1"
        tampered_report.write_text(
            json.dumps(tampered_payload, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )
        tampered_policy_code, tampered_policy_output = run_capture(
            ["bash", str(checker), "--report-file", str(tampered_report)]
        )
        if tampered_policy_code == 0:
            return fail(
                "expected signer incident recovery policy checker to reject tampered NO-GO report"
            )
        if "policy decision mismatch" not in tampered_policy_output:
            return fail(
                "expected signer incident recovery policy checker mismatch marker for tampered report"
            )

        runbook_text = runbook_doc.read_text(encoding="utf-8")
        checklist_text = checklist_doc.read_text(encoding="utf-8")
        required_runbook_snippets = (
            "## Secure-Signer Incident Recovery Contract Lanes (Issue #989)",
            "run_signer_incident_recovery_lane.sh",
            "check_signer_incident_recovery_policy.sh",
            "run_signer_incident_recovery_contract_lane.sh",
            "run_signer_incident_recovery_deep_lane.sh",
            "KAMN_SIGNER_INCIDENT_RECOVERY_DEEP_CADENCE",
            "Regression: #989",
        )
        for snippet in required_runbook_snippets:
            if snippet not in runbook_text:
                return fail(
                    "expected upgrade rollback runbook to reference signer incident recovery contracts"
                )

        required_checklist_snippets = (
            "## Signer Incident Recovery Contract and Deep-Lane Cadence (Issue #989)",
            "run_signer_incident_recovery_contract_lane.sh",
            "run_signer_incident_recovery_deep_lane.sh",
            "check_signer_incident_recovery_policy.sh",
            "signer_incident_recovery_reason_codes:GO:v1",
            "Regression: #989",
        )
        for snippet in required_checklist_snippets:
            if snippet not in checklist_text:
                return fail(
                    "expected release go/no-go checklist to reference signer incident recovery contracts"
                )

        elapsed_seconds = int(time.time()) - start_epoch
        if elapsed_seconds > max_contract_seconds:
            return fail(
                "signer incident recovery contract lane exceeded runtime budget: "
                f"{elapsed_seconds}s"
            )

        print("status=ok")
        print(f"report_file={output_file}")
        print("final_decision=GO")
        print("signer incident recovery contract lane tests passed.")
        return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
