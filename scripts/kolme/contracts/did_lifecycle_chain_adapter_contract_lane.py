#!/usr/bin/env python3
"""Contract lane runner for DID lifecycle chain-adapter checks."""

from __future__ import annotations

import json
import os
import subprocess
import sys
import time
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parents[3]
TEST_TARGET = "did_registry_transactions"
CARGO_TARGET_DIR = ROOT_DIR / "target" / "contract-lanes" / "did-lifecycle-chain-adapter"
DID_REGISTRATION_REASON_TAXONOMY_VERSION = "kamn.kolme.did-registration-reason-taxonomy.v1"
DID_REGISTRATION_REASON_CODES = ["did_registry_document_did_mismatch", "did_registry_submission_key_conflict"]
TEST_FILTERS = [
    "functional_lifecycle_chain_submission_through_kolme_adapter_returns_typed_outcome",
    "integration_lifecycle_chain_submission_allows_retry_without_reapplying_mutation",
    "regression_lifecycle_chain_submission_rejects_conflicting_same_nonce_payload",
    "regression_registration_chain_submission_rejects_malformed_document_payload",
    "regression_registration_chain_submission_rejects_duplicate_registration_payload_drift",
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


def cargo_target_env() -> dict[str, str]:
    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(CARGO_TARGET_DIR)
    return env


def artifact_executable(cargo_stdout: str) -> Path | None:
    executable = None
    for line in cargo_stdout.splitlines():
        if not line.strip().startswith("{"):
            continue
        try:
            artifact = json.loads(line)
        except json.JSONDecodeError:
            continue
        if artifact.get("reason") != "compiler-artifact":
            continue
        if artifact.get("target", {}).get("name") == TEST_TARGET and artifact.get("executable"):
            executable = Path(artifact["executable"])
    if executable is not None and executable.is_file():
        return executable
    return None


def prebuild_test_executable() -> tuple[int, Path | None]:
    compile_command = [
        "cargo", "test", "-p", "kamn-core", "--test", TEST_TARGET, "--no-run", "--message-format=json",
    ]
    result = subprocess.run(
        compile_command,
        cwd=ROOT_DIR,
        check=False,
        env=cargo_target_env(),
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        if result.stderr:
            print(result.stderr, file=sys.stderr, end="")
        return result.returncode, None
    executable = artifact_executable(result.stdout)
    if executable is None:
        print("expected Cargo to report DID lifecycle test executable", file=sys.stderr)
        return 1, None
    return 0, executable


def run_prebuilt_tests(executable: Path, max_seconds: int) -> tuple[int, str]:
    try:
        result = subprocess.run(
            [str(executable), *TEST_FILTERS, "--test-threads=1", "--nocapture"],
            cwd=ROOT_DIR,
            check=False,
            capture_output=True,
            text=True,
            timeout=max_seconds,
        )
    except subprocess.TimeoutExpired as error:
        test_output = (error.stdout or "") + (error.stderr or "")
        if test_output:
            print(test_output, file=sys.stderr, end="")
        print(
            f"did lifecycle chain adapter contract lane exceeded runtime budget: {max_seconds}s",
            file=sys.stderr,
        )
        return 1, test_output

    return result.returncode, (result.stdout or "") + (result.stderr or "")

def main() -> int:
    output_json, max_seconds = parse_args(sys.argv[1:])

    start_epoch = time.monotonic()
    compile_code, executable = prebuild_test_executable()
    if compile_code != 0 or executable is None:
        return compile_code or 1

    test_code, test_output = run_prebuilt_tests(executable, max_seconds)
    if test_code != 0:
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

    payload = dict(
        schema_version="kamn.kolme.did-lifecycle-chain.contract.v1",
        status="pass",
        final_decision="GO",
        lifecycle_chain_contract_status="verified",
        duplicate_retry_status="verified",
        conflict_fail_closed_status="verified",
        malformed_registration_payload_status="verified",
        duplicate_registration_payload_drift_status="verified",
        did_registration_reason_taxonomy_version=DID_REGISTRATION_REASON_TAXONOMY_VERSION,
        did_registration_reason_codes_csv=",".join(DID_REGISTRATION_REASON_CODES),
        did_registration_reason_codes_value="none",
        elapsed_seconds=elapsed_seconds,
    )

    if output_json:
        output_path = Path(output_json)
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    print("status=pass")
    print("final_decision=GO")
    print("lifecycle_chain_contract_status=verified")
    print("duplicate_retry_status=verified")
    print("conflict_fail_closed_status=verified")
    print("malformed_registration_payload_status=verified")
    print("duplicate_registration_payload_drift_status=verified")
    print(f"did_registration_reason_taxonomy_version={DID_REGISTRATION_REASON_TAXONOMY_VERSION}")
    print(f"did_registration_reason_codes_csv={','.join(DID_REGISTRATION_REASON_CODES)}")
    print("did_registration_reason_codes_value=none")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
