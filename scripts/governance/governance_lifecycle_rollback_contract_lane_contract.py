#!/usr/bin/env python3
"""Governance lifecycle/rollback contract-lane runner."""

from __future__ import annotations

import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time


def usage() -> None:
    """Print usage text."""
    print(
        "Usage:\n"
        "  bash scripts/governance/run_governance_lifecycle_rollback_contract_lane.sh \\\n"
        "    [--output-file <path>]"
    )


def fail(message: str) -> int:
    """Emit stable error and return non-zero."""
    print(message, file=sys.stderr)
    return 1


def extract_value(output: str, key: str) -> str:
    """Extract key=value marker from output."""
    prefix = f"{key}="
    for line in output.splitlines():
        if line.startswith(prefix):
            return line[len(prefix) :]
    return ""


def run_capture(
    command: list[str],
    *,
    env: dict[str, str] | None = None,
    root_dir: Path,
) -> tuple[int, str]:
    """Run command and return exit code + merged output."""
    result = subprocess.run(
        command,
        cwd=root_dir,
        capture_output=True,
        text=True,
        check=False,
        env=env,
    )
    return result.returncode, (result.stdout or "") + (result.stderr or "")


def parse_args(argv: list[str]) -> tuple[int, Path]:
    """Parse args and return (status, output_file)."""
    output_file = Path("/tmp/governance-lifecycle-rollback-contract-report.json")
    index = 0
    while index < len(argv):
        argument = argv[index]
        if argument in {"--help", "-h"}:
            usage()
            return 0, output_file
        if argument == "--output-file":
            if index + 1 >= len(argv):
                return fail("unknown argument: --output-file"), output_file
            output_file = Path(argv[index + 1])
            index += 2
            continue
        return fail(f"unknown argument: {argument}"), output_file
    return 200, output_file


def main(argv: list[str]) -> int:
    """Execute governance lifecycle/rollback contract-lane checks."""
    parse_status, output_file = parse_args(argv)
    if parse_status != 200:
        return parse_status

    root_dir = Path(__file__).resolve().parents[2]
    lane_script = root_dir / "scripts/governance/run_governance_lifecycle_rollback_lane.sh"
    policy_checker = root_dir / "scripts/governance/check_governance_lifecycle_rollback_policy.sh"

    if not lane_script.is_file() or not os.access(lane_script, os.X_OK):
        return fail("governance lifecycle/rollback lane script is not executable")
    if not policy_checker.is_file() or not os.access(policy_checker, os.X_OK):
        return fail("governance lifecycle/rollback policy checker script is not executable")

    max_runtime_raw = os.getenv("KAMN_GOVERNANCE_LIFECYCLE_ROLLBACK_CONTRACT_MAX_SECONDS", "240")
    if not max_runtime_raw.isdigit():
        return fail("KAMN_GOVERNANCE_LIFECYCLE_ROLLBACK_CONTRACT_MAX_SECONDS must be an integer >= 0")
    max_runtime = int(max_runtime_raw)
    start_epoch = int(time.time())

    with tempfile.TemporaryDirectory() as temp_dir:
        temp_path = Path(temp_dir)
        go_report = temp_path / "governance-lifecycle-rollback-go.json"

        go_env = os.environ.copy()
        go_env["KAMN_GOVERNANCE_LIFECYCLE_ROLLBACK_SKIP_COMMANDS"] = "true"
        go_code, go_output = run_capture(
            ["bash", str(lane_script), "--output-file", str(go_report)],
            env=go_env,
            root_dir=root_dir,
        )
        if go_code != 0 or "final_decision=GO" not in go_output:
            return fail("expected governance lifecycle/rollback contract GO path to produce GO decision")
        if "reason_key=governance_lifecycle_rollback_reason_codes:GO:v1" not in go_output:
            return fail("expected governance lifecycle/rollback GO path reason_key marker")

        go_policy_code, go_policy_output = run_capture(
            ["bash", str(policy_checker), "--report-file", str(go_report)],
            root_dir=root_dir,
        )
        if go_policy_code != 0 or "final_decision=GO" not in go_policy_output:
            return fail("expected governance lifecycle/rollback policy checker GO decision")

        no_go_report = temp_path / "governance-lifecycle-rollback-no-go.json"
        no_go_env = os.environ.copy()
        no_go_env["KAMN_GOVERNANCE_LIFECYCLE_ROLLBACK_SKIP_COMMANDS"] = "true"
        no_go_env["KAMN_GOVERNANCE_LIFECYCLE_FORCE_DOCS_CONTRACT_MISSING"] = "true"
        no_go_code, no_go_output = run_capture(
            ["bash", str(lane_script), "--output-file", str(no_go_report)],
            env=no_go_env,
            root_dir=root_dir,
        )
        if no_go_code != 0 or "final_decision=NO-GO" not in no_go_output:
            return fail("expected governance lifecycle/rollback forced docs drift path to produce NO-GO")
        if "reason_key=governance_lifecycle_rollback_reason_codes:NO-GO:v1" not in no_go_output:
            return fail("expected governance lifecycle/rollback NO-GO reason_key marker")

        no_go_policy_code, no_go_policy_output = run_capture(
            ["bash", str(policy_checker), "--report-file", str(no_go_report)],
            root_dir=root_dir,
        )
        if no_go_policy_code != 0 or "final_decision=NO-GO" not in no_go_policy_output:
            return fail("expected governance lifecycle/rollback policy checker NO-GO decision")

        tampered_report = temp_path / "governance-lifecycle-rollback-tampered.json"
        shutil.copyfile(no_go_report, tampered_report)
        payload = json.loads(tampered_report.read_text())
        payload["reason_key"] = "governance_lifecycle_rollback_reason_codes:GO:v1"
        tampered_report.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n")

        tampered_code, tampered_output = run_capture(
            ["bash", str(policy_checker), "--report-file", str(tampered_report)],
            root_dir=root_dir,
        )
        if tampered_code == 0:
            return fail("expected governance lifecycle/rollback reason_key tamper to fail policy checker")
        if "reason_key mismatch" not in tampered_output:
            return fail("expected explicit reason_key mismatch failure from governance lifecycle/rollback policy checker")

        output_file.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(go_report, output_file)

        runtime_seconds = int(time.time()) - start_epoch
        if runtime_seconds > max_runtime:
            return fail(
                "governance lifecycle/rollback contract lane exceeded runtime budget "
                f"({runtime_seconds}s > {max_runtime}s)"
            )

        print("status=ok")
        print(f"output_file={output_file}")
        print(f"final_decision={extract_value(go_output, 'final_decision')}")
        print(f"reason_key={extract_value(go_output, 'reason_key')}")
        print("governance lifecycle/rollback contract lane tests passed.")
        return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
