#!/usr/bin/env python3
"""Classification/redaction compliance lane runner."""

from __future__ import annotations

from datetime import datetime, timezone
import json
import os
from pathlib import Path
import subprocess
import sys
import time


def usage() -> None:
    """Print CLI usage."""
    print(
        "Usage:\n"
        "  bash scripts/compliance/run_classification_redaction_lane.sh \\\n"
        "    [--output-file <path>]"
    )


def fail(message: str) -> int:
    """Emit stable error and return non-zero."""
    print(message, file=sys.stderr)
    return 1


def parse_args(argv: list[str], root_dir: Path) -> tuple[int, Path | None]:
    """Parse CLI args and return exit code/output path."""
    output_file: Path = root_dir / "classification-redaction-report.json"
    index = 0
    while index < len(argv):
        argument = argv[index]
        if argument == "--output-file":
            if index + 1 >= len(argv):
                return fail("unknown argument: --output-file"), None
            output_file = Path(argv[index + 1])
            index += 2
            continue
        if argument in {"--help", "-h"}:
            usage()
            return 0, None
        return fail(f"unknown argument: {argument}"), None
    return 200, output_file


def main(argv: list[str]) -> int:
    """Run lane orchestration and emit compliance report."""
    root_dir = Path(__file__).resolve().parents[2]

    parse_status, output_path = parse_args(argv, root_dir)
    if parse_status != 200:
        return parse_status

    classification_contract = (
        root_dir / "scripts/compliance/run_dsar_legal_hold_contract_lane.sh"
    )
    redaction_contract = (
        root_dir / "scripts/channel/run_channel_retention_redaction_contract_lane.sh"
    )
    classification_doc = root_dir / "docs/foundation/data-classification-tagging.md"
    redaction_doc = root_dir / "docs/foundation/redaction-tombstones.md"

    max_runtime_seconds_raw = os.getenv("KAMN_CLASSIFICATION_REDACTION_MAX_SECONDS", "180")
    if not max_runtime_seconds_raw.isdigit():
        return fail("KAMN_CLASSIFICATION_REDACTION_MAX_SECONDS must be an integer >= 0")
    max_runtime_seconds = int(max_runtime_seconds_raw)

    skip_commands = os.getenv("KAMN_CLASSIFICATION_REDACTION_SKIP_COMMANDS", "false")
    if skip_commands not in {"true", "false"}:
        return fail("KAMN_CLASSIFICATION_REDACTION_SKIP_COMMANDS must be true or false")

    lane_failed = False
    classification_contract_present = (
        classification_contract.is_file() and os.access(classification_contract, os.X_OK)
    )
    redaction_contract_present = (
        redaction_contract.is_file() and os.access(redaction_contract, os.X_OK)
    )
    docs_contract_present = True
    commands: list[str] = []

    if os.getenv("KAMN_CLASSIFICATION_REDACTION_FORCE_CLASSIFICATION_MISSING", "false") == "true":
        classification_contract_present = False
    if os.getenv("KAMN_CLASSIFICATION_REDACTION_FORCE_REDACTION_MISSING", "false") == "true":
        redaction_contract_present = False

    start_epoch = int(time.time())

    if skip_commands != "true":
        if classification_contract_present:
            commands.append("bash scripts/compliance/run_dsar_legal_hold_contract_lane.sh")
            result = subprocess.run(
                ["bash", str(classification_contract)],
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                check=False,
            )
            if result.returncode != 0:
                lane_failed = True

        if redaction_contract_present:
            commands.append(
                "bash scripts/channel/run_channel_retention_redaction_contract_lane.sh --skip-tests"
            )
            result = subprocess.run(
                ["bash", str(redaction_contract), "--skip-tests"],
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
                check=False,
            )
            if result.returncode != 0:
                lane_failed = True

    if os.getenv("KAMN_CLASSIFICATION_REDACTION_FORCE_LANE_FAILURE", "false") == "true":
        lane_failed = True

    required_doc_markers = (
        "run_classification_redaction_lane.sh",
        "classification_redaction_lane_contract.py",
        "run_classification_redaction_contract_lane.sh",
        "check_classification_redaction_policy.sh",
        "classification_redaction_policy_contract.py",
        "kamn.compliance.classification-redaction-report.v1",
        "classification_redaction_reason_codes:GO:v1",
        "classification_redaction_reason_codes:NO-GO:v1",
        "classification/redaction contract drift must fail closed (`Regression: #914`).",
        "Regression: #1222",
        "Regression: #1226",
    )

    for marker in required_doc_markers:
        if marker not in classification_doc.read_text(encoding="utf-8"):
            docs_contract_present = False
        if marker not in redaction_doc.read_text(encoding="utf-8"):
            docs_contract_present = False

    if os.getenv("KAMN_CLASSIFICATION_REDACTION_FORCE_DOCS_CONTRACT_MISSING", "false") == "true":
        docs_contract_present = False

    runtime_seconds = int(time.time()) - start_epoch
    runtime_budget_ok = runtime_seconds <= max_runtime_seconds

    decision_reasons: list[str] = []
    if lane_failed:
        decision_reasons.append("classification_redaction_lane_failed")
    if not classification_contract_present:
        decision_reasons.append("classification_contract_missing")
    if not redaction_contract_present:
        decision_reasons.append("redaction_contract_missing")
    if not docs_contract_present:
        decision_reasons.append("docs_contract_missing")
    if not runtime_budget_ok:
        decision_reasons.append("runtime_budget_exceeded")

    final_decision = "GO" if not decision_reasons else "NO-GO"
    reason_key = f"classification_redaction_reason_codes:{final_decision}:v1"

    output_path.parent.mkdir(parents=True, exist_ok=True)

    payload = {
        "schema_version": "kamn.compliance.classification-redaction-report.v1",
        "generated_at": datetime.now(tz=timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "max_runtime_seconds": max_runtime_seconds,
        "runtime_seconds": runtime_seconds,
        "checks": {
            "lane_failed": lane_failed,
            "classification_contract_present": classification_contract_present,
            "redaction_contract_present": redaction_contract_present,
            "docs_contract_present": docs_contract_present,
            "runtime_budget_ok": runtime_budget_ok,
        },
        "commands": commands,
        "decision_reasons": decision_reasons,
        "final_decision": final_decision,
        "reason_key": reason_key,
    }
    output_path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")

    print("status=ok")
    print(f"output_file={output_path}")
    print(f"final_decision={final_decision}")
    print(f"reason_key={reason_key}")
    print(f"runtime_seconds={runtime_seconds}")
    print(f"max_runtime_seconds={max_runtime_seconds}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
