#!/usr/bin/env python3
"""Contract lane runner for Kolme runtime commit adapter replay/finality checks."""

from __future__ import annotations

import os
import subprocess
import sys
import time
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parents[3]
POLICY_CHECKER = ROOT_DIR / "scripts/kolme/check_runtime_commit_replay_policy.py"
GONOGO_DOC = ROOT_DIR / "docs/foundation/release-gonogo-checklist.md"
DEVNET_PLAN_DOC = ROOT_DIR / "docs/planning/kolme-devnet-ops.md"
MAX_SECONDS = 60


def command_output(command: list[str]) -> tuple[int, str]:
    result = subprocess.run(
        command,
        cwd=ROOT_DIR,
        check=False,
        capture_output=True,
        text=True,
    )
    return result.returncode, (result.stdout or "") + (result.stderr or "")


def main() -> int:
    if not os.access(POLICY_CHECKER, os.X_OK):
        print("expected runtime commit replay policy checker to be executable", file=sys.stderr)
        return 1

    if not GONOGO_DOC.is_file() or not DEVNET_PLAN_DOC.is_file():
        print("expected release go/no-go and Kolme devnet docs to exist", file=sys.stderr)
        return 1

    start_epoch = time.monotonic()
    tmp_report = Path(subprocess.check_output(["mktemp"], text=True).strip())
    try:
        subprocess.run(
            [
                "cargo",
                "test",
                "-p",
                "kamn-core",
                "--test",
                "kolme_runtime_commit_client",
                "functional_adapter_maps_transport_provider_and_finality_failures_to_typed_errors",
            ],
            cwd=ROOT_DIR,
            check=True,
        )
        subprocess.run(
            [
                "cargo",
                "test",
                "-p",
                "kamn-core",
                "--test",
                "kolme_runtime_commit_client",
                "integration_runtime_pipeline_accepts_adapter_backed_final_receipts",
            ],
            cwd=ROOT_DIR,
            check=True,
        )
        subprocess.run(
            [
                "cargo",
                "test",
                "-p",
                "kamn-core",
                "--test",
                "kolme_runtime_commit_client",
                "regression_adapter_path_keeps_receipt_provider_mismatch_fail_closed",
            ],
            cwd=ROOT_DIR,
            check=True,
        )

        provider_mismatch_code, provider_mismatch_output = command_output(
            [
                "python3",
                str(POLICY_CHECKER),
                "--operation-id",
                "op-adapter-no-go-provider-mismatch-001",
                "--idempotency-key",
                "kolme-runtime-commit:op-adapter-no-go-provider-mismatch-001:state:agent:7:12",
                "--receipt-provider",
                "kolme-remote",
                "--expected-receipt-provider",
                "kolme-local",
                "--receipt-commit-id",
                "kolme-commit:op-adapter-no-go-provider-mismatch-001:agent:7:12",
                "--expected-receipt-commit-id",
                "kolme-commit:op-adapter-no-go-provider-mismatch-001:agent:7:12",
                "--nonce-monotonic",
                "true",
                "--replay-detected",
                "false",
                "--payload-hash-match",
                "true",
                "--receipt-finality",
                "FINAL",
                "--ci-fast-gate",
                "PASS",
                "--output-json",
                str(tmp_report),
            ]
        )
        if provider_mismatch_code == 0:
            print("expected provider mismatch policy case to fail closed", file=sys.stderr)
            return 1
        if "final_decision=NO-GO" not in provider_mismatch_output:
            print("expected provider mismatch policy case to produce NO-GO", file=sys.stderr)
            return 1
        if "receipt_provider_mismatch" not in provider_mismatch_output:
            print(
                "expected provider mismatch policy case to emit receipt_provider_mismatch reason code",
                file=sys.stderr,
            )
            return 1

        non_final_code, non_final_output = command_output(
            [
                "python3",
                str(POLICY_CHECKER),
                "--operation-id",
                "op-adapter-no-go-non-final-001",
                "--idempotency-key",
                "kolme-runtime-commit:op-adapter-no-go-non-final-001:state:agent:8:12",
                "--receipt-provider",
                "kolme-local",
                "--expected-receipt-provider",
                "kolme-local",
                "--receipt-commit-id",
                "kolme-commit:op-adapter-no-go-non-final-001:agent:8:12",
                "--expected-receipt-commit-id",
                "kolme-commit:op-adapter-no-go-non-final-001:agent:8:12",
                "--nonce-monotonic",
                "true",
                "--replay-detected",
                "false",
                "--payload-hash-match",
                "true",
                "--receipt-finality",
                "PENDING",
                "--ci-fast-gate",
                "PASS",
                "--output-json",
                str(tmp_report),
            ]
        )
        if non_final_code == 0:
            print("expected non-final receipt policy case to fail closed", file=sys.stderr)
            return 1
        if "final_decision=NO-GO" not in non_final_output:
            print("expected non-final receipt policy case to produce NO-GO", file=sys.stderr)
            return 1
        if "receipt_not_final" not in non_final_output:
            print(
                "expected non-final receipt policy case to emit receipt_not_final reason code",
                file=sys.stderr,
            )
            return 1

        gonogo_doc_text = GONOGO_DOC.read_text(encoding="utf-8")
        devnet_doc_text = DEVNET_PLAN_DOC.read_text(encoding="utf-8")
        adapter_contract_lane_markers = [
            "run_runtime_commit_adapter_contract_lane.sh",
            "run_manifest_lane.sh --manifest scripts/framework/manifests/kolme_runtime_commit_adapter_contract_lane.json --phase contract",
        ]
        if not any(marker in gonogo_doc_text for marker in adapter_contract_lane_markers):
            print(
                "expected release go/no-go doc to reference adapter runtime commit contract lane command",
                file=sys.stderr,
            )
            return 1
        if not any(marker in devnet_doc_text for marker in adapter_contract_lane_markers):
            print(
                "expected Kolme devnet plan doc to reference adapter runtime commit contract lane command",
                file=sys.stderr,
            )
            return 1
        if "receipt_provider_mismatch" not in gonogo_doc_text:
            print(
                "expected release go/no-go doc to reference receipt_provider_mismatch reason code",
                file=sys.stderr,
            )
            return 1
        if "receipt_not_final" not in gonogo_doc_text:
            print(
                "expected release go/no-go doc to reference receipt_not_final reason code",
                file=sys.stderr,
            )
            return 1
        if "Regression: #980" not in gonogo_doc_text:
            print(
                "expected release go/no-go doc to include adapter replay/finality regression marker",
                file=sys.stderr,
            )
            return 1

        elapsed_seconds = int(time.monotonic() - start_epoch)
        if elapsed_seconds > MAX_SECONDS:
            print(
                f"Kolme runtime commit adapter contract lane exceeded runtime budget: {elapsed_seconds}s",
                file=sys.stderr,
            )
            return 1
    finally:
        tmp_report.unlink(missing_ok=True)

    print("Kolme runtime commit adapter contract lane tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
