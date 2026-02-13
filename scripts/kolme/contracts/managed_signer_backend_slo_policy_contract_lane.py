#!/usr/bin/env python3
"""Contract lane for managed-signer backend SLO policy checker behavior."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parents[3]
GENERATOR = ROOT_DIR / "scripts/kolme/generate_managed_signer_backend_slo_telemetry_bundle.sh"
POLICY_CHECKER = ROOT_DIR / "scripts/kolme/check_managed_signer_backend_slo_policy.py"
DOC_FILES = [
    ROOT_DIR / "docs/ci/ci-cost-and-lane-framework.md",
    ROOT_DIR / "docs/planning/kolme-devnet-ops.md",
    ROOT_DIR / "README.md",
]

DOC_MARKERS = [
    "check_managed_signer_backend_slo_policy.py",
    "run_managed_signer_backend_slo_policy_contract_lane.sh",
    "kamn.kolme.managed-signer-backend-slo-policy-report.v1",
    "kamn.kolme.managed-signer-backend-slo-policy-contract-report.v1",
    "managed_signer_backend_slo_within_threshold",
    "managed_signer_backend_no_action_required",
    "managed_signer_backend_timeout_rate_threshold_exceeded",
    "managed_signer_backend_unavailable_rate_threshold_exceeded",
    "managed_signer_backend_error_rate_threshold_exceeded",
    "managed_signer_backend_ci_fast_gate_failed",
    "managed_signer_backend_reduce_timeout_burst",
    "managed_signer_backend_failover_endpoint",
    "managed_signer_backend_enable_circuit_breaker",
    "managed_signer_backend_replay_ci_fast_gate",
]


def run_command(command: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        command,
        cwd=ROOT_DIR,
        check=False,
        capture_output=True,
        text=True,
    )


def ensure_markers_present(text: str, markers: list[str], source_name: str) -> list[str]:
    missing: list[str] = []
    for marker in markers:
        if marker not in text:
            missing.append(f"{source_name}_missing_marker:{marker}")
    return missing


def run_generator(bundle_path: Path, **kwargs: str) -> subprocess.CompletedProcess[str]:
    command = [
        "bash",
        str(GENERATOR),
        "--output-file",
        str(bundle_path),
    ]
    for key, value in kwargs.items():
        command.extend([f"--{key.replace('_', '-')}", value])
    return run_command(command)


def run_checker(bundle_path: Path, report_path: Path) -> subprocess.CompletedProcess[str]:
    command = [
        "python3",
        str(POLICY_CHECKER),
        "--telemetry-bundle",
        str(bundle_path),
        "--output-json",
        str(report_path),
    ]
    return run_command(command)


def _require_output_marker(output: str, marker: str, message: str) -> None:
    if marker not in output:
        raise RuntimeError(message)


def _read_json(path: Path) -> dict:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise RuntimeError(f"expected JSON object in {path}")
    return payload


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Run managed-signer backend SLO policy contract lane."
    )
    parser.add_argument(
        "--output-json",
        default="/tmp/managed-signer-backend-slo-policy-contract-report.json",
    )
    args = parser.parse_args()

    if not GENERATOR.is_file() or not GENERATOR.stat().st_mode & 0o111:
        print("expected managed-signer backend SLO telemetry generator to be executable", file=sys.stderr)
        return 1
    if not POLICY_CHECKER.is_file() or not POLICY_CHECKER.stat().st_mode & 0o111:
        print("expected managed-signer backend SLO policy checker to be executable", file=sys.stderr)
        return 1

    missing_doc_markers: list[str] = []
    for doc_file in DOC_FILES:
        if not doc_file.is_file():
            print(f"expected documentation file to exist: {doc_file}", file=sys.stderr)
            return 1
        missing_doc_markers.extend(
            ensure_markers_present(
                doc_file.read_text(encoding="utf-8"),
                DOC_MARKERS,
                str(doc_file.relative_to(ROOT_DIR)),
            )
        )
    if missing_doc_markers:
        print(",".join(missing_doc_markers), file=sys.stderr)
        return 1

    try:
        with tempfile.TemporaryDirectory(prefix="managed-signer-slo-policy-contract-") as temp_dir:
            temp_path = Path(temp_dir)
            go_bundle = temp_path / "go-bundle.json"
            no_go_bundle = temp_path / "no-go-bundle.json"
            ci_bundle = temp_path / "ci-bundle.json"
            go_report = temp_path / "go-policy-report.json"
            no_go_report = temp_path / "no-go-policy-report.json"
            ci_report = temp_path / "ci-policy-report.json"

            go_bundle_result = run_generator(
                go_bundle,
                window_start_utc="2026-02-13T00:00:00Z",
                window_end_utc="2026-02-13T00:15:00Z",
                backend_name="kolme-managed-signer-primary",
                signer_profile="ops-primary",
                signer_key_source="managed-external",
                sample_count="100",
                timeout_events="0",
                unavailable_events="0",
                error_events="1",
                max_timeout_rate_bps="100",
                max_unavailable_rate_bps="100",
                max_error_rate_bps="200",
                ci_fast_gate="PASS",
            )
            if go_bundle_result.returncode != 0:
                raise RuntimeError(
                    f"expected GO fixture generation to succeed: {go_bundle_result.stderr.strip()}"
                )

            go_policy_result = run_checker(go_bundle, go_report)
            if go_policy_result.returncode != 0:
                raise RuntimeError(
                    f"expected GO policy case to pass: {go_policy_result.stdout}{go_policy_result.stderr}"
                )
            _require_output_marker(
                go_policy_result.stdout,
                "final_decision=GO",
                "expected GO policy output final_decision=GO",
            )
            _require_output_marker(
                go_policy_result.stdout,
                "reason_codes=managed_signer_backend_slo_within_threshold",
                "expected deterministic GO reason code",
            )

            no_go_bundle_result = run_generator(
                no_go_bundle,
                window_start_utc="2026-02-13T00:15:00Z",
                window_end_utc="2026-02-13T00:30:00Z",
                backend_name="kolme-managed-signer-primary",
                signer_profile="ops-primary",
                signer_key_source="managed-external",
                sample_count="100",
                timeout_events="9",
                unavailable_events="8",
                error_events="9",
                max_timeout_rate_bps="500",
                max_unavailable_rate_bps="500",
                max_error_rate_bps="500",
                ci_fast_gate="PASS",
            )
            if no_go_bundle_result.returncode != 0:
                raise RuntimeError(
                    f"expected NO-GO threshold fixture generation to succeed: {no_go_bundle_result.stderr.strip()}"
                )

            no_go_policy_result = run_checker(no_go_bundle, no_go_report)
            if no_go_policy_result.returncode == 0:
                raise RuntimeError("expected threshold-breach policy case to fail closed")
            _require_output_marker(
                no_go_policy_result.stdout,
                "final_decision=NO-GO",
                "expected threshold-breach policy output final_decision=NO-GO",
            )
            for marker in [
                "managed_signer_backend_timeout_rate_threshold_exceeded",
                "managed_signer_backend_unavailable_rate_threshold_exceeded",
                "managed_signer_backend_error_rate_threshold_exceeded",
            ]:
                _require_output_marker(
                    no_go_policy_result.stdout,
                    marker,
                    f"expected deterministic threshold reason code marker: {marker}",
                )

            ci_bundle_result = run_generator(
                ci_bundle,
                window_start_utc="2026-02-13T00:30:00Z",
                window_end_utc="2026-02-13T00:45:00Z",
                backend_name="kolme-managed-signer-primary",
                signer_profile="ops-primary",
                signer_key_source="managed-external",
                sample_count="100",
                timeout_events="0",
                unavailable_events="0",
                error_events="0",
                max_timeout_rate_bps="100",
                max_unavailable_rate_bps="100",
                max_error_rate_bps="100",
                ci_fast_gate="FAIL",
            )
            if ci_bundle_result.returncode != 0:
                raise RuntimeError(
                    f"expected ci-fast-gate NO-GO fixture generation to succeed: {ci_bundle_result.stderr.strip()}"
                )

            ci_policy_result = run_checker(ci_bundle, ci_report)
            if ci_policy_result.returncode == 0:
                raise RuntimeError("expected ci-fast-gate policy case to fail closed")
            _require_output_marker(
                ci_policy_result.stdout,
                "managed_signer_backend_ci_fast_gate_failed",
                "expected ci-fast-gate NO-GO reason code marker",
            )
            _require_output_marker(
                ci_policy_result.stdout,
                "managed_signer_backend_replay_ci_fast_gate",
                "expected ci-fast-gate remediation marker",
            )

            go_payload = _read_json(go_report)
            no_go_payload = _read_json(no_go_report)
            ci_payload = _read_json(ci_report)

            if go_payload.get("schema_version") != "kamn.kolme.managed-signer-backend-slo-policy-report.v1":
                raise RuntimeError("unexpected GO policy report schema")
            if go_payload.get("final_decision") != "GO":
                raise RuntimeError("expected GO policy report final decision")
            if go_payload.get("reason_codes") != ["managed_signer_backend_slo_within_threshold"]:
                raise RuntimeError("expected deterministic GO policy reason code list")

            if no_go_payload.get("final_decision") != "NO-GO":
                raise RuntimeError("expected NO-GO threshold policy report final decision")
            no_go_reasons = set(no_go_payload.get("reason_codes", []))
            for marker in [
                "managed_signer_backend_timeout_rate_threshold_exceeded",
                "managed_signer_backend_unavailable_rate_threshold_exceeded",
                "managed_signer_backend_error_rate_threshold_exceeded",
            ]:
                if marker not in no_go_reasons:
                    raise RuntimeError(f"expected threshold-breach report marker: {marker}")
            no_go_remediation = set(no_go_payload.get("remediation_markers", []))
            for marker in [
                "managed_signer_backend_reduce_timeout_burst",
                "managed_signer_backend_failover_endpoint",
                "managed_signer_backend_enable_circuit_breaker",
            ]:
                if marker not in no_go_remediation:
                    raise RuntimeError(f"expected threshold remediation marker: {marker}")

            if ci_payload.get("final_decision") != "NO-GO":
                raise RuntimeError("expected ci-fast-gate policy report final decision")
            if "managed_signer_backend_ci_fast_gate_failed" not in set(ci_payload.get("reason_codes", [])):
                raise RuntimeError("expected ci-fast-gate reason code in report")
            if "managed_signer_backend_replay_ci_fast_gate" not in set(ci_payload.get("remediation_markers", [])):
                raise RuntimeError("expected ci-fast-gate remediation marker in report")

            contract_report = {
                "schema_version": "kamn.kolme.managed-signer-backend-slo-policy-contract-report.v1",
                "go_policy_report": str(go_report),
                "no_go_policy_report": str(no_go_report),
                "ci_fast_gate_no_go_policy_report": str(ci_report),
                "final_decision": "GO",
            }
            output_path = Path(args.output_json).resolve()
            output_path.parent.mkdir(parents=True, exist_ok=True)
            output_path.write_text(
                json.dumps(contract_report, sort_keys=True, indent=2) + "\n",
                encoding="utf-8",
            )
    except RuntimeError as exc:
        print(str(exc), file=sys.stderr)
        return 1

    print("managed-signer backend SLO policy contract lane tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
