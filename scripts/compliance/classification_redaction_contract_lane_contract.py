#!/usr/bin/env python3
"""Classification/redaction compliance contract-lane runner."""

from __future__ import annotations

import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time


def usage() -> None:
    """Print CLI usage."""
    print(
        "Usage:\n"
        "  bash scripts/compliance/run_classification_redaction_contract_lane.sh \\\n"
        "    [--output-file <path>]"
    )


def fail(message: str) -> int:
    """Emit stable error and return non-zero."""
    print(message, file=sys.stderr)
    return 1


def extract_value(output: str, key: str) -> str:
    """Extract key=value line value from contract command output."""
    for line in output.splitlines():
        if line.startswith(f"{key}="):
            return line.split("=", 1)[1]
    return ""


def run_capture(command: list[str], env: dict[str, str] | None = None) -> tuple[int, str]:
    """Run command and return (exit_code, merged output)."""
    result = subprocess.run(
        command,
        capture_output=True,
        text=True,
        check=False,
        env=env,
    )
    return result.returncode, (result.stdout or "") + (result.stderr or "")


def parse_args(argv: list[str]) -> tuple[int, str]:
    """Parse CLI args and return exit code/output file path."""
    output_file = "/tmp/classification-redaction-contract-report.json"
    index = 0
    while index < len(argv):
        argument = argv[index]
        if argument == "--output-file":
            if index + 1 >= len(argv):
                return fail("unknown argument: --output-file"), output_file
            output_file = argv[index + 1]
            index += 2
            continue
        if argument in {"--help", "-h"}:
            usage()
            return 0, output_file
        return fail(f"unknown argument: {argument}"), output_file
    return 200, output_file


def main(argv: list[str]) -> int:
    """Execute classification/redaction contract lane checks."""
    parse_status, output_file = parse_args(argv)
    if parse_status != 200:
        return parse_status

    root_dir = Path(__file__).resolve().parents[2]
    lane_script = root_dir / "scripts/compliance/run_classification_redaction_lane.sh"
    policy_checker = root_dir / "scripts/compliance/check_classification_redaction_policy.sh"

    if not lane_script.is_file() or not os.access(lane_script, os.X_OK):
        return fail("classification/redaction lane script is not executable")
    if not policy_checker.is_file() or not os.access(policy_checker, os.X_OK):
        return fail("classification/redaction policy checker script is not executable")

    max_runtime_raw = os.getenv("KAMN_CLASSIFICATION_REDACTION_CONTRACT_MAX_SECONDS", "240")
    if not max_runtime_raw.isdigit():
        return fail("KAMN_CLASSIFICATION_REDACTION_CONTRACT_MAX_SECONDS must be an integer >= 0")
    max_runtime = int(max_runtime_raw)

    start_epoch = int(time.time())

    with tempfile.TemporaryDirectory() as temp_dir:
        go_report = Path(temp_dir) / "classification-redaction-go.json"
        env_go = os.environ.copy()
        env_go["KAMN_CLASSIFICATION_REDACTION_SKIP_COMMANDS"] = "true"
        go_code, go_output = run_capture(
            ["bash", str(lane_script), "--output-file", str(go_report)], env_go
        )
        if go_code != 0:
            return fail("expected classification/redaction contract lane GO path to produce GO decision")
        if "final_decision=GO" not in go_output:
            return fail("expected classification/redaction contract lane GO path to produce GO decision")
        if "reason_key=classification_redaction_reason_codes:GO:v1" not in go_output:
            return fail("expected classification/redaction contract lane GO path reason_key marker")

        go_policy_code, go_policy_output = run_capture(
            ["bash", str(policy_checker), "--report-file", str(go_report)]
        )
        if go_policy_code != 0 or "final_decision=GO" not in go_policy_output:
            return fail("expected classification/redaction policy checker GO path decision")

        no_go_report = Path(temp_dir) / "classification-redaction-no-go.json"
        env_no_go = os.environ.copy()
        env_no_go["KAMN_CLASSIFICATION_REDACTION_SKIP_COMMANDS"] = "true"
        env_no_go["KAMN_CLASSIFICATION_REDACTION_FORCE_DOCS_CONTRACT_MISSING"] = "true"
        no_go_code, no_go_output = run_capture(
            ["bash", str(lane_script), "--output-file", str(no_go_report)],
            env_no_go,
        )
        if no_go_code != 0:
            return fail("expected classification/redaction contract lane forced docs drift to produce NO-GO")
        if "final_decision=NO-GO" not in no_go_output:
            return fail("expected classification/redaction contract lane forced docs drift to produce NO-GO")
        if "reason_key=classification_redaction_reason_codes:NO-GO:v1" not in no_go_output:
            return fail("expected classification/redaction contract lane NO-GO reason_key marker")

        no_go_policy_code, no_go_policy_output = run_capture(
            ["bash", str(policy_checker), "--report-file", str(no_go_report)]
        )
        if no_go_policy_code != 0 or "final_decision=NO-GO" not in no_go_policy_output:
            return fail("expected classification/redaction policy checker NO-GO path decision")

        tampered_report = Path(temp_dir) / "classification-redaction-tampered.json"
        tampered_report.write_text(no_go_report.read_text(encoding="utf-8"), encoding="utf-8")
        import json

        payload = json.loads(tampered_report.read_text(encoding="utf-8"))
        payload["reason_key"] = "classification_redaction_reason_codes:GO:v1"
        tampered_report.write_text(
            json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8"
        )

        tampered_code, tampered_output = run_capture(
            ["bash", str(policy_checker), "--report-file", str(tampered_report)]
        )
        if tampered_code == 0:
            return fail("expected classification/redaction reason_key tamper to fail policy checker")
        if "reason_key mismatch" not in tampered_output:
            return fail(
                "expected explicit reason_key mismatch failure from classification/redaction policy checker"
            )

        output_path = Path(output_file)
        output_path.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(go_report, output_path)

        runtime_seconds = int(time.time()) - start_epoch
        if runtime_seconds > max_runtime:
            return fail(
                "classification/redaction contract lane exceeded runtime budget "
                f"({runtime_seconds}s > {max_runtime}s)"
            )

        print("status=ok")
        print(f"output_file={output_path}")
        print(f"final_decision={extract_value(go_output, 'final_decision')}")
        print(f"reason_key={extract_value(go_output, 'reason_key')}")
        print("classification/redaction compliance contract lane tests passed.")
        return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
