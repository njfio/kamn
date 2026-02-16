#!/usr/bin/env python3
"""Contract lane runner for DID lifecycle chain-adapter checks."""

from __future__ import annotations

import json
import subprocess
import sys
import time
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parents[3]
DID_REGISTRATION_REASON_TAXONOMY_VERSION = "kamn.kolme.did-registration-reason-taxonomy.v1"
DID_REGISTRATION_REASON_CODES = [
    "did_registry_document_did_mismatch",
    "did_registry_submission_key_conflict",
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
        "did_registry_transactions",
        "--",
        "functional_lifecycle_chain_submission_through_kolme_adapter_returns_typed_outcome",
        "integration_lifecycle_chain_submission_allows_retry_without_reapplying_mutation",
        "regression_lifecycle_chain_submission_rejects_conflicting_same_nonce_payload",
        "regression_registration_chain_submission_rejects_malformed_document_payload",
        "regression_registration_chain_submission_rejects_duplicate_registration_payload_drift",
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
        print("did lifecycle chain adapter contract lane failed", file=sys.stderr)
        return 1

    if "5 passed; 0 failed" not in test_output:
        if test_output:
            print(test_output, file=sys.stderr, end="")
        print(
            "expected did lifecycle chain contract pass-count marker",
            file=sys.stderr,
        )
        return 1

    elapsed_seconds = int(time.monotonic() - start_epoch)
    if elapsed_seconds > max_seconds:
        print(
            "did lifecycle chain adapter contract lane exceeded runtime budget: "
            f"{elapsed_seconds}s",
            file=sys.stderr,
        )
        return 1

    payload = {
        "schema_version": "kamn.kolme.did-lifecycle-chain.contract.v1",
        "status": "pass",
        "final_decision": "GO",
        "lifecycle_chain_contract_status": "verified",
        "duplicate_retry_status": "verified",
        "conflict_fail_closed_status": "verified",
        "malformed_registration_payload_status": "verified",
        "duplicate_registration_payload_drift_status": "verified",
        "did_registration_reason_taxonomy_version": DID_REGISTRATION_REASON_TAXONOMY_VERSION,
        "did_registration_reason_codes_csv": ",".join(DID_REGISTRATION_REASON_CODES),
        "did_registration_reason_codes_value": "none",
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
    print("lifecycle_chain_contract_status=verified")
    print("duplicate_retry_status=verified")
    print("conflict_fail_closed_status=verified")
    print("malformed_registration_payload_status=verified")
    print("duplicate_registration_payload_drift_status=verified")
    print(
        "did_registration_reason_taxonomy_version="
        f"{DID_REGISTRATION_REASON_TAXONOMY_VERSION}"
    )
    print(
        "did_registration_reason_codes_csv="
        f"{','.join(DID_REGISTRATION_REASON_CODES)}"
    )
    print("did_registration_reason_codes_value=none")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
