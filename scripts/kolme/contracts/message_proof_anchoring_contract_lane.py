#!/usr/bin/env python3
"""Contract lane runner for message-proof anchoring checks."""

from __future__ import annotations

import json
import subprocess
import sys
import time
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parents[3]
ANCHORING_GATE_REASON_TAXONOMY_VERSION = (
    "kamn.kolme.message-proof-anchoring-gate-reason-taxonomy.v1"
)
ANCHORING_GATE_REASON_CODES = [
    "message_anchor_evidence_mismatch",
    "message_anchor_evidence_tamper_detected",
    "message_proof_anchor_conflicting_key",
    "message_proof_anchor_invalid_state",
    "ci_fast_gate_failed",
    "local_heavy_opt_in_required",
]


def parse_args(argv: list[str]) -> tuple[str, int]:
    output_json = ""
    max_seconds_raw = "180"

    index = 0
    while index < len(argv):
        argument = argv[index]
        if argument == "--output-json":
            if index + 1 >= len(argv):
                print("missing value for --output-json", file=sys.stderr)
                raise SystemExit(1)
            output_json = argv[index + 1]
            index += 2
            continue
        if argument == "--max-seconds":
            if index + 1 >= len(argv):
                print("missing value for --max-seconds", file=sys.stderr)
                raise SystemExit(1)
            max_seconds_raw = argv[index + 1]
            index += 2
            continue

        print(f"unknown argument: {argument}", file=sys.stderr)
        raise SystemExit(1)

    if not max_seconds_raw.isdigit():
        print("max-seconds must be an integer", file=sys.stderr)
        raise SystemExit(1)

    max_seconds = int(max_seconds_raw)
    if max_seconds <= 0:
        print("max-seconds must be greater than zero", file=sys.stderr)
        raise SystemExit(1)

    return output_json, max_seconds


def main() -> int:
    output_json, max_seconds = parse_args(sys.argv[1:])

    start_epoch = time.monotonic()
    command = [
        "cargo",
        "test",
        "-p",
        "kamn-core",
        "--test",
        "message_proof_anchoring",
        "--",
        "functional_anchor_submission_advances_broadcast_to_included_with_typed_outcome",
        "integration_anchor_retry_is_duplicate_without_reapplying_state_transition",
        "regression_anchor_submission_rejects_lifecycle_state_mismatch_before_broadcast",
        "regression_anchor_submission_rejects_tampered_actor_for_same_message_nonce",
        "regression_anchor_conflicting_payload_for_same_message_rejected_fail_closed",
        "performance_anchor_submission_contract_lane_stays_within_budget",
    ]
    result = subprocess.run(
        command,
        cwd=ROOT_DIR,
        check=False,
        capture_output=True,
        text=True,
    )

    test_output = (result.stdout or "") + (result.stderr or "")
    if result.returncode != 0:
        if test_output:
            print(test_output, file=sys.stderr, end="")
        print("message proof anchoring contract lane failed", file=sys.stderr)
        return 1

    if "6 passed; 0 failed" not in test_output:
        if test_output:
            print(test_output, file=sys.stderr, end="")
        print(
            "expected message proof anchoring contract pass-count marker",
            file=sys.stderr,
        )
        return 1

    elapsed_seconds = int(time.monotonic() - start_epoch)
    if elapsed_seconds > max_seconds:
        print(
            "message proof anchoring contract lane exceeded runtime budget: "
            f"{elapsed_seconds}s",
            file=sys.stderr,
        )
        return 1

    payload = {
        "schema_version": "kamn.kolme.message-proof-anchoring.contract.v1",
        "status": "pass",
        "final_decision": "GO",
        "message_anchor_contract_status": "verified",
        "lifecycle_alignment_status": "verified",
        "conflict_fail_closed_status": "verified",
        "mismatch_fail_closed_status": "verified",
        "tamper_fail_closed_status": "verified",
        "anchoring_gate_reason_taxonomy_version": ANCHORING_GATE_REASON_TAXONOMY_VERSION,
        "anchoring_gate_reason_codes_csv": ",".join(ANCHORING_GATE_REASON_CODES),
        "anchoring_gate_reason_codes_value": "none",
        "ci_smoke_local_heavy_boundary_status": "verified",
        "ci_smoke_lane_cost_profile": "low",
        "local_heavy_lane_execution_mode": "opt_in",
        "performance_budget_status": "verified",
        "elapsed_seconds": elapsed_seconds,
    }

    if output_json:
        output_path = Path(output_json)
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(
            json.dumps(payload, indent=2) + "\n",
            encoding="utf-8",
        )

    print("status=pass")
    print("final_decision=GO")
    print("message_anchor_contract_status=verified")
    print("lifecycle_alignment_status=verified")
    print("conflict_fail_closed_status=verified")
    print("mismatch_fail_closed_status=verified")
    print("tamper_fail_closed_status=verified")
    print(
        "anchoring_gate_reason_taxonomy_version="
        f"{ANCHORING_GATE_REASON_TAXONOMY_VERSION}"
    )
    print("anchoring_gate_reason_codes_csv=" f"{','.join(ANCHORING_GATE_REASON_CODES)}")
    print("anchoring_gate_reason_codes_value=none")
    print("ci_smoke_local_heavy_boundary_status=verified")
    print("ci_smoke_lane_cost_profile=low")
    print("local_heavy_lane_execution_mode=opt_in")
    print("performance_budget_status=verified")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
