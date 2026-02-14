#!/usr/bin/env python3
"""Contract lane for managed-signer startup live validation behavior."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
import sys
import tempfile
import time
from pathlib import Path
from typing import Any

ROOT_DIR = Path(__file__).resolve().parents[3]
RUNNER = ROOT_DIR / "scripts/kolme/run_local_kolme_live_deployment_preflight_lane.sh"
CHECKER = ROOT_DIR / "scripts/kolme/check_local_kolme_live_deployment_preflight_policy.py"
UPGRADE_ROLLBACK_RUNBOOK = ROOT_DIR / "docs/foundation/upgrade-rollback-runbook.md"
ROADMAP_DOC = ROOT_DIR / "docs/plans/2026-02-08-production-service-roadmap.md"
DOC_FILES = [
    ROOT_DIR / "docs/planning/kolme-devnet-ops.md",
    ROOT_DIR / "docs/ci/ci-cost-and-lane-framework.md",
    ROADMAP_DOC,
    ROOT_DIR / "README.md",
]
DOC_MARKERS = [
    "run_managed_signer_startup_live_validation_contract_lane.sh",
    "kamn.kolme.managed-signer-startup-live-validation-contract-report.v1",
    "deployment_preflight_passed",
    "checkpoint_failed_signer_profile_contract",
    "checkpoint_failed_signer_provenance_contract",
    "checkpoint_failed_signer_rotation_freshness_contract",
    "signer_key_source_production_managed_external_required",
    "signer_profile_mismatch",
    "signer_rotation_epoch_stale",
    "execution_scope=local-scheduled",
]
PROFILE_MATRIX_DOC_FILES = [
    ROADMAP_DOC,
    UPGRADE_ROLLBACK_RUNBOOK,
]
PROFILE_MATRIX_DOC_MARKERS = [
    "signer_key_source_profile_matrix_status=verified",
    "signer_key_source_production_reject_status=verified",
    "signer_key_source_local_override_allow_status=verified",
    "production_signer_key_source_env_local_forbidden",
    "KAMN_KOLME_LIVE_ALLOW_LOCAL_SIGNER_TESTING=true",
]

PRIMARY_TEST_PRIVATE_KEY_HEX = "1" * 64
SECONDARY_TEST_PRIVATE_KEY_HEX = "2" * 64


def run_command(command: list[str], *, env: dict[str, str] | None = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        command,
        cwd=ROOT_DIR,
        check=False,
        capture_output=True,
        text=True,
        env=env,
    )


def ensure_markers_present(text: str, markers: list[str], source_name: str) -> list[str]:
    missing: list[str] = []
    for marker in markers:
        if marker not in text:
            missing.append(f"{source_name}_missing_marker:{marker}")
    return missing


def _read_json(path: Path) -> dict[str, Any]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise RuntimeError(f"expected JSON object in {path}")
    return payload


def _write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")


def _write_evidence_bundle(case_dir: Path) -> tuple[Path, Path, Path]:
    custody_path = case_dir / "custody.json"
    provenance_path = case_dir / "provenance.json"
    quorum_path = case_dir / "quorum.json"

    custody_payload = {
        "schema_version": "kamn.kolme.runtime-signer-custody.v1",
        "attestation": "ops-primary custody epoch-3",
    }
    _write_json(custody_path, custody_payload)
    custody_sha256 = hashlib.sha256(custody_path.read_bytes()).hexdigest()

    provenance_payload = {
        "schema_version": "kamn.kolme.runtime-signer-provenance.v1",
        "source": "managed-external",
        "signer_profile": "ops-primary",
    }
    _write_json(provenance_path, provenance_payload)

    quorum_payload = {
        "schema_version": "kamn.kolme.runtime-signer-attestation.v1",
        "required_approvals": 2,
        "received_approvals": 2,
        "approved_signers": ["ops-primary", "ops-secondary"],
        "signer_roles": {"ops-primary": "primary", "ops-secondary": "secondary"},
        "signer_rotation_epochs": {"ops-primary": 3, "ops-secondary": 2},
        "custody_evidence_sha256": custody_sha256,
    }
    _write_json(quorum_path, quorum_payload)
    return custody_path, provenance_path, quorum_path


def _assert_marker(output: str, marker: str, message: str) -> None:
    if marker not in output:
        raise RuntimeError(message)


def run_preflight_scenario(
    *,
    temp_root: Path,
    scenario_id: str,
    signer_profile: str,
    signer_key_source: str,
    signer_rotation_epoch: int,
    signer_previous_rotation_epoch: int,
    expected_runner_status: str,
    expected_reason_code: str,
    expected_final_decision: str,
    expected_policy_reason_code: str | None,
) -> dict[str, Any]:
    case_dir = temp_root / scenario_id
    case_dir.mkdir(parents=True, exist_ok=True)
    custody_path, provenance_path, quorum_path = _write_evidence_bundle(case_dir)
    summary_path = case_dir / "summary.json"
    policy_path = case_dir / "policy.json"

    env = os.environ.copy()
    env["KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX"] = PRIMARY_TEST_PRIVATE_KEY_HEX
    env["KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY"] = SECONDARY_TEST_PRIVATE_KEY_HEX
    env.pop("KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK", None)

    runner_result = run_command(
        [
            "bash",
            str(RUNNER),
            "--mode",
            "run",
            "--runtime-mode",
            "kolme-live",
            "--signer-profile",
            signer_profile,
            "--required-approvals",
            "2",
            "--received-approvals",
            "2",
            "--quorum-evidence-file",
            str(quorum_path),
            "--custody-evidence-file",
            str(custody_path),
            "--signer-provenance-file",
            str(provenance_path),
            "--signer-key-source-contract-version",
            "v1",
            "--signer-key-source",
            signer_key_source,
            "--signer-rotation-epoch",
            str(signer_rotation_epoch),
            "--signer-previous-rotation-epoch",
            str(signer_previous_rotation_epoch),
            "--signer-rotation-freshness-max-delta",
            "2",
            "--max-seconds",
            "20",
            "--output-json",
            str(summary_path),
        ],
        env=env,
    )

    runner_output = f"{runner_result.stdout}{runner_result.stderr}"
    _assert_marker(
        runner_output,
        f"status={expected_runner_status}",
        f"expected runner status marker for scenario {scenario_id}: status={expected_runner_status}",
    )
    _assert_marker(
        runner_output,
        f"reason_code={expected_reason_code}",
        f"expected reason-code marker for scenario {scenario_id}: reason_code={expected_reason_code}",
    )

    if expected_runner_status == "ok" and runner_result.returncode != 0:
        raise RuntimeError(
            f"expected scenario {scenario_id} to succeed but runner failed: {runner_output.strip()}"
        )
    if expected_runner_status == "fail" and runner_result.returncode == 0:
        raise RuntimeError(f"expected scenario {scenario_id} to fail closed")

    summary_payload = _read_json(summary_path)
    if summary_payload.get("reason_code") != expected_reason_code:
        raise RuntimeError(f"scenario {scenario_id} summary reason code mismatch")

    checker_result = run_command(
        [
            "python3",
            str(CHECKER),
            "--report-file",
            str(summary_path),
            "--expected-final-decision",
            expected_final_decision,
            "--ci-fast-gate",
            "PASS",
            "--require-reason-code",
            expected_reason_code,
            "--output-json",
            str(policy_path),
        ]
    )
    checker_output = f"{checker_result.stdout}{checker_result.stderr}"
    _assert_marker(
        checker_output,
        f"final_decision={expected_final_decision}",
        f"expected checker decision marker for scenario {scenario_id}: final_decision={expected_final_decision}",
    )

    if expected_final_decision == "GO" and checker_result.returncode != 0:
        raise RuntimeError(
            f"expected scenario {scenario_id} policy checker GO path to pass: {checker_output.strip()}"
        )
    if expected_final_decision == "NO-GO" and checker_result.returncode == 0:
        raise RuntimeError(f"expected scenario {scenario_id} policy checker to fail closed")

    policy_payload = _read_json(policy_path)
    if policy_payload.get("final_decision") != expected_final_decision:
        raise RuntimeError(f"scenario {scenario_id} policy report final decision mismatch")

    if expected_policy_reason_code is not None:
        reason_codes = policy_payload.get("reason_codes")
        if not isinstance(reason_codes, list) or expected_policy_reason_code not in reason_codes:
            raise RuntimeError(
                f"scenario {scenario_id} missing expected policy reason code: {expected_policy_reason_code}"
            )

    return {
        "scenario_id": scenario_id,
        "summary_report": str(summary_path),
        "policy_report": str(policy_path),
        "expected_reason_code": expected_reason_code,
        "expected_policy_reason_code": expected_policy_reason_code,
        "final_decision": expected_final_decision,
    }


def run_key_source_policy_matrix_test(
    *,
    scenario_id: str,
    test_name: str,
    expected_policy_outcome: str,
    expected_reason_code: str,
) -> dict[str, Any]:
    command = [
        "cargo",
        "test",
        "-p",
        "kamn-node",
        test_name,
        "--",
        "--exact",
    ]
    result = run_command(command)
    output = f"{result.stdout}{result.stderr}"
    if result.returncode != 0:
        raise RuntimeError(
            f"signer key-source matrix scenario {scenario_id} failed: {output.strip()}"
        )
    return {
        "scenario_id": scenario_id,
        "command": " ".join(command),
        "expected_policy_outcome": expected_policy_outcome,
        "expected_reason_code": expected_reason_code,
        "status": "pass",
    }


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Run managed-signer startup live validation contract lane."
    )
    parser.add_argument(
        "--output-json",
        default="/tmp/managed-signer-startup-live-validation-contract-report.json",
    )
    parser.add_argument("--max-seconds", type=int, default=120)
    args = parser.parse_args()

    if args.max_seconds <= 0:
        print("max-seconds must be a positive integer", file=sys.stderr)
        return 1

    if not RUNNER.is_file() or not RUNNER.stat().st_mode & 0o111:
        print("expected deployment preflight runner to be executable", file=sys.stderr)
        return 1
    if not CHECKER.is_file() or not CHECKER.stat().st_mode & 0o111:
        print("expected deployment preflight policy checker to be executable", file=sys.stderr)
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
    for doc_file in PROFILE_MATRIX_DOC_FILES:
        missing_doc_markers.extend(
            ensure_markers_present(
                doc_file.read_text(encoding="utf-8"),
                PROFILE_MATRIX_DOC_MARKERS,
                str(doc_file.relative_to(ROOT_DIR)),
            )
        )
    if missing_doc_markers:
        print(",".join(missing_doc_markers), file=sys.stderr)
        return 1

    start_time = time.time()
    try:
        with tempfile.TemporaryDirectory(prefix="managed-signer-startup-live-contract-") as temp_dir:
            temp_root = Path(temp_dir)
            scenario_reports = [
                run_preflight_scenario(
                    temp_root=temp_root,
                    scenario_id="go_baseline",
                    signer_profile="ops-primary",
                    signer_key_source="managed-external",
                    signer_rotation_epoch=3,
                    signer_previous_rotation_epoch=1,
                    expected_runner_status="ok",
                    expected_reason_code="deployment_preflight_passed",
                    expected_final_decision="GO",
                    expected_policy_reason_code=None,
                ),
                run_preflight_scenario(
                    temp_root=temp_root,
                    scenario_id="no_go_missing_key_source",
                    signer_profile="ops-primary",
                    signer_key_source="env-local",
                    signer_rotation_epoch=3,
                    signer_previous_rotation_epoch=1,
                    expected_runner_status="fail",
                    expected_reason_code="checkpoint_failed_signer_provenance_contract",
                    expected_final_decision="NO-GO",
                    expected_policy_reason_code="signer_key_source_production_managed_external_required",
                ),
                run_preflight_scenario(
                    temp_root=temp_root,
                    scenario_id="no_go_invalid_signer_profile",
                    signer_profile="ops-tertiary",
                    signer_key_source="managed-external",
                    signer_rotation_epoch=3,
                    signer_previous_rotation_epoch=1,
                    expected_runner_status="fail",
                    expected_reason_code="checkpoint_failed_signer_profile_contract",
                    expected_final_decision="NO-GO",
                    expected_policy_reason_code="signer_profile_mismatch",
                ),
                run_preflight_scenario(
                    temp_root=temp_root,
                    scenario_id="no_go_stale_rotation_metadata",
                    signer_profile="ops-primary",
                    signer_key_source="managed-external",
                    signer_rotation_epoch=5,
                    signer_previous_rotation_epoch=1,
                    expected_runner_status="fail",
                    expected_reason_code="checkpoint_failed_signer_rotation_freshness_contract",
                    expected_final_decision="NO-GO",
                    expected_policy_reason_code="signer_rotation_epoch_stale",
                ),
            ]
            key_source_matrix_reports = [
                run_key_source_policy_matrix_test(
                    scenario_id="production_strict_env_local_rejected",
                    test_name="main_tests::core_behavior_tests::functional_kolme_live_strict_env_local_key_source_rejects_with_reason_code",
                    expected_policy_outcome="NO-GO",
                    expected_reason_code="production_signer_key_source_env_local_forbidden",
                ),
                run_key_source_policy_matrix_test(
                    scenario_id="local_override_env_local_allowed",
                    test_name="main_tests::core_behavior_tests::functional_kolme_live_strict_env_local_key_source_allows_with_local_override",
                    expected_policy_outcome="GO",
                    expected_reason_code="local_override_enabled",
                ),
                run_key_source_policy_matrix_test(
                    scenario_id="production_strict_managed_external_allowed",
                    test_name="main_tests::core_behavior_tests::integration_kolme_live_strict_managed_external_key_source_policy_passes",
                    expected_policy_outcome="GO",
                    expected_reason_code="managed_external_required",
                ),
            ]

            elapsed_seconds = int(time.time() - start_time)
            if elapsed_seconds > args.max_seconds:
                raise RuntimeError(
                    f"managed-signer startup live validation exceeded runtime budget: "
                    f"{elapsed_seconds}s > {args.max_seconds}s"
                )

            report_payload = {
                "schema_version": "kamn.kolme.managed-signer-startup-live-validation-contract-report.v1",
                "status": "pass",
                "final_decision": "GO",
                "execution_scope": "local-scheduled",
                "ci_fast_gate_eligible": False,
                "max_seconds": args.max_seconds,
                "elapsed_seconds": elapsed_seconds,
                "managed_signer_profile_status": "verified",
                "managed_signer_missing_key_source_fail_closed_status": "verified",
                "managed_signer_invalid_profile_fail_closed_status": "verified",
                "managed_signer_stale_rotation_fail_closed_status": "verified",
                "managed_signer_reason_code_status": "verified",
                "signer_key_source_profile_matrix_status": "verified",
                "signer_key_source_production_reject_status": "verified",
                "signer_key_source_local_override_allow_status": "verified",
                "signer_key_source_managed_external_allow_status": "verified",
                "performance_budget_status": "verified",
                "scenario_reports": scenario_reports,
                "signer_key_source_matrix_reports": key_source_matrix_reports,
            }

            output_path = Path(args.output_json).resolve()
            output_path.parent.mkdir(parents=True, exist_ok=True)
            output_path.write_text(
                json.dumps(report_payload, sort_keys=True, indent=2) + "\n",
                encoding="utf-8",
            )
    except RuntimeError as exc:
        print(str(exc), file=sys.stderr)
        return 1

    print("status=pass")
    print("final_decision=GO")
    print("managed_signer_profile_status=verified")
    print("managed_signer_missing_key_source_fail_closed_status=verified")
    print("managed_signer_invalid_profile_fail_closed_status=verified")
    print("managed_signer_stale_rotation_fail_closed_status=verified")
    print("managed_signer_reason_code_status=verified")
    print("signer_key_source_profile_matrix_status=verified")
    print("signer_key_source_production_reject_status=verified")
    print("signer_key_source_local_override_allow_status=verified")
    print("signer_key_source_managed_external_allow_status=verified")
    print("execution_scope=local-scheduled")
    print("performance_budget_status=verified")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
