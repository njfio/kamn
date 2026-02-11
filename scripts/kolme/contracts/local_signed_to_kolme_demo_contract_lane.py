#!/usr/bin/env python3
"""Contract lane runner for unified local signed-to-Kolme demo checks."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import time
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parents[3]
SIGNED_DEMO = ROOT_DIR / "scripts/sdk/run_localhost_signed_demo_contract_lane.sh"
SIGNED_INTEGRATION = ROOT_DIR / "scripts/sdk/run_localhost_signed_integration_contract_lane.sh"
RUNTIME_INTEGRATION = ROOT_DIR / "scripts/kolme/run_local_kamn_live_runtime_integration_contract_lane.sh"
CHECKER = ROOT_DIR / "scripts/kolme/check_local_signed_to_kolme_demo_policy.py"
DOC_FILE = ROOT_DIR / "docs/planning/kolme-devnet-ops.md"
README_FILE = ROOT_DIR / "README.md"


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Run unified local signed-to-Kolme demo contract lane checks."
    )
    parser.add_argument(
        "--mode",
        default="run",
        choices=("dry-run", "run"),
        help="Emit planned checks or run the demo checkpoint sequence.",
    )
    parser.add_argument(
        "--output-json",
        default="/tmp/kolme-local-signed-to-kolme-demo-summary.json",
        help="Signed-to-Kolme demo summary output.",
    )
    parser.add_argument(
        "--policy-output-json",
        default="/tmp/kolme-local-signed-to-kolme-demo-policy.json",
        help="Signed-to-Kolme policy report output.",
    )
    parser.add_argument(
        "--max-seconds",
        default="420",
        help="Total runtime budget in seconds.",
    )
    return parser


def main() -> int:
    args = build_parser().parse_args()
    if not args.max_seconds.isdigit() or int(args.max_seconds) <= 0:
        print("max-seconds must be a positive integer", file=sys.stderr)
        return 1
    max_seconds = int(args.max_seconds)

    for script in (SIGNED_DEMO, SIGNED_INTEGRATION, RUNTIME_INTEGRATION, CHECKER):
        if not script.is_file() or not script.stat().st_mode & 0o111:
            print(f"expected executable dependency: {script}", file=sys.stderr)
            return 1

    if not DOC_FILE.is_file() or not README_FILE.is_file():
        print("expected docs to exist", file=sys.stderr)
        return 1

    doc_text = DOC_FILE.read_text(encoding="utf-8")
    readme_text = README_FILE.read_text(encoding="utf-8")
    for marker in (
        "run_local_signed_to_kolme_demo_contract_lane.sh",
        "check_local_signed_to_kolme_demo_policy.py",
        "Regression: #1640",
    ):
        if marker not in doc_text:
            print(f"expected Kolme devnet ops doc marker: {marker}", file=sys.stderr)
            return 1
    if "run_local_signed_to_kolme_demo_contract_lane.sh" not in readme_text:
        print("expected README to reference signed-to-Kolme contract lane", file=sys.stderr)
        return 1

    start_epoch = time.monotonic()
    checks: list[dict[str, str]] = []
    status = "ok"
    reason_code = "dry_run_no_commands_executed"
    budget_status = "not_run"

    checkpoint_commands = [
        ("localhost_signed_demo_contract", ["bash", str(SIGNED_DEMO)]),
        ("localhost_signed_integration_contract", ["bash", str(SIGNED_INTEGRATION)]),
        (
            "local_kamn_runtime_integration_contract",
            ["bash", str(RUNTIME_INTEGRATION)],
        ),
    ]

    if args.mode == "dry-run":
        for check_id, command in checkpoint_commands:
            checks.append(
                {
                    "id": check_id,
                    "command": " ".join(command),
                    "status": "planned",
                    "reason_code": "not_run",
                }
            )
    else:
        budget_status = "within_budget"
        reason_code = "signed_to_kolme_demo_passed"
        for check_id, command in checkpoint_commands:
            result = subprocess.run(
                command,
                cwd=ROOT_DIR,
                check=False,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            if result.returncode == 0:
                checks.append(
                    {
                        "id": check_id,
                        "command": " ".join(command),
                        "status": "pass",
                        "reason_code": f"{check_id}_passed",
                    }
                )
                continue
            checks.append(
                {
                    "id": check_id,
                    "command": " ".join(command),
                    "status": "fail",
                    "reason_code": f"{check_id}_failed",
                }
            )
            status = "fail"
            reason_code = f"checkpoint_failed_{check_id}"
            break

        if status == "fail":
            remaining = [entry for entry in checkpoint_commands if entry[0] not in {c["id"] for c in checks}]
            for check_id, command in remaining:
                checks.append(
                    {
                        "id": check_id,
                        "command": " ".join(command),
                        "status": "skipped",
                        "reason_code": "skipped_due_prior_failure",
                    }
                )

    elapsed_seconds = int(time.monotonic() - start_epoch)
    if elapsed_seconds > max_seconds:
        budget_status = "exceeded_budget"
        if status == "ok":
            status = "fail"
            reason_code = "demo_budget_exceeded"

    summary = {
        "schema_version": "kamn.kolme.local-signed-to-kolme-demo-summary.v1",
        "mode": args.mode,
        "status": status,
        "reason_code": reason_code,
        "local_only_enforced": True,
        "elapsed_seconds": elapsed_seconds,
        "max_seconds": max_seconds,
        "budget_status": budget_status,
        "checks": checks,
        "artifact_paths": [
            "/tmp/localhost-signed-demo-contract-report.json",
            "/tmp/localhost-signed-integration-contract-report.json",
            "/tmp/kolme-local-kamn-live-runtime-integration-summary.json",
            "/tmp/kolme-local-kamn-live-runtime-integration-policy.json",
        ],
    }

    output_path = Path(args.output_json).resolve()
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(summary, sort_keys=True, indent=2) + "\n", encoding="utf-8")

    expected_final_decision = "GO" if status == "ok" else "NO-GO"
    subprocess.run(
        [
            "python3",
            str(CHECKER),
            "--report-file",
            str(output_path),
            "--expected-final-decision",
            expected_final_decision,
            "--ci-fast-gate",
            "PASS",
            "--require-reason-code",
            reason_code,
            "--output-json",
            args.policy_output_json,
        ],
        cwd=ROOT_DIR,
        check=True,
        stdout=subprocess.DEVNULL,
    )

    print("unified local signed-to-Kolme demo contract lane tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
