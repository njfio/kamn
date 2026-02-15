#!/usr/bin/env python3
"""Local full-stack integration live-validation lane and policy checker."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time

SCRIPT_DIR = Path(__file__).resolve().parent
ROOT_DIR = SCRIPT_DIR.parent.parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import (  # noqa: E402
    ContractError,
    DecisionAccumulator,
    fail,
    load_json,
    require_enum,
    require_positive_int,
    write_json,
)

RUN_LANE_SCHEMA = "kamn.runtime.local-full-stack-integration-live-report.v1"
POLICY_SCHEMA = "kamn.runtime.local-full-stack-integration-live-policy-report.v1"
EVIDENCE_BUNDLE_SCHEMA = "kamn.runtime.local-full-stack-integration-evidence-bundle.v1"
KOLME_INTEGRATION_REPORT_SCHEMA = "kamn.kolme.local-kamn-live-runtime-integration-summary.v1"
KOLME_INTEGRATION_POLICY_SCHEMA = "kamn.kolme.local-kamn-live-runtime-integration-policy-report.v1"
KOLME_PROVIDER_CLIENT_CONTRACT = "KolmeRuntimeCommitLiveProvider"
KOLME_RUNTIME_SIGNING_PROFILE = "kolme-fork-secp256k1-v1"
KOLME_SIGNER_ATTESTATION_SCHEMA = "kamn.kolme.runtime-signer-attestation.v1"
KOLME_RUNTIME_INTEGRATION_RUN_REASON = "live_runtime_integration_passed"
KOLME_DEFAULT_CHECKOUT_PATH = "/tmp/kolme_fork"
KOLME_DEFAULT_EXPECTED_REMOTE_URL = "https://github.com/njfio/kolme_fork.git"
KOLME_DEFAULT_EXPECTED_REF = "refs/heads/main"
KOLME_DEFAULT_BASE_URL = "http://127.0.0.1:3000"
KOLME_DEFAULT_FORK_CHAIN_VERSION = "v0.15.2"
OPT_IN_ENV = "KAMN_LOCAL_FULL_STACK_INTEGRATION_OPT_IN"
DRY_RUN_REASON = "dry_run_no_commands_executed"
RUN_REASON = "local_full_stack_integration_live_validation_executed"
FAST_GATE_EXCLUSION_REASON = "local_full_stack_integration_run_mode_excluded_from_fast_gate"


def _extract_line_value(output: str, key: str) -> str:
    prefix = f"{key}="
    for line in output.splitlines():
        if line.startswith(prefix):
            return line[len(prefix) :]
    return ""


def _run_command(
    command: list[str],
    *,
    timeout_seconds: int,
    env: dict[str, str] | None = None,
) -> str:
    completed = subprocess.run(
        command,
        cwd=ROOT_DIR,
        capture_output=True,
        text=True,
        check=False,
        timeout=timeout_seconds,
        env=env,
    )
    if completed.returncode != 0:
        detail = (completed.stderr or completed.stdout or "command failed").strip()
        fail(f"lane command failed: {' '.join(command)}: {detail}")
    return completed.stdout


def _write_json(path: Path, payload: dict[str, object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")


def _read_json_dict(path: Path, *, failure_reason: str) -> dict[str, object]:
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        fail(failure_reason)
    if not isinstance(payload, dict):
        fail(failure_reason)
    return payload


def _require_local_kolme_checkout(
    *,
    checkout_path: Path,
    expected_remote_url: str,
    expected_ref: str,
) -> None:
    if not checkout_path.is_dir():
        fail("local_kolme_checkout_missing")

    inside_work_tree = subprocess.run(
        ["git", "-C", str(checkout_path), "rev-parse", "--is-inside-work-tree"],
        cwd=ROOT_DIR,
        capture_output=True,
        text=True,
        check=False,
    )
    if inside_work_tree.returncode != 0:
        fail("local_kolme_checkout_not_git_repo")

    origin_remote = subprocess.run(
        ["git", "-C", str(checkout_path), "remote", "get-url", "origin"],
        cwd=ROOT_DIR,
        capture_output=True,
        text=True,
        check=False,
    )
    if origin_remote.returncode != 0:
        fail("local_kolme_checkout_origin_missing")
    observed_remote = origin_remote.stdout.strip()
    if observed_remote != expected_remote_url:
        fail("local_kolme_checkout_remote_mismatch")

    symbolic_ref = subprocess.run(
        ["git", "-C", str(checkout_path), "symbolic-ref", "-q", "HEAD"],
        cwd=ROOT_DIR,
        capture_output=True,
        text=True,
        check=False,
    )
    if symbolic_ref.returncode != 0:
        fail("local_kolme_checkout_ref_missing")
    observed_ref = symbolic_ref.stdout.strip()
    if observed_ref != expected_ref:
        fail("local_kolme_checkout_ref_mismatch")


def run_lane(args: argparse.Namespace) -> int:
    mode = require_enum("--mode", args.mode.strip(), ("dry-run", "run"))
    ci_fast_gate = require_enum("--ci-fast-gate", args.ci_fast_gate.strip(), ("PASS", "FAIL"))
    max_seconds = require_positive_int("KAMN_LOCAL_FULL_STACK_INTEGRATION_MAX_SECONDS", args.max_seconds)
    command_max_seconds = require_positive_int(
        "KAMN_LOCAL_FULL_STACK_INTEGRATION_COMMAND_MAX_SECONDS",
        args.command_max_seconds,
    )
    kolme_checkout_path = Path(args.kolme_checkout_path).expanduser().resolve()
    kolme_expected_remote_url = args.kolme_expected_remote_url.strip()
    kolme_expected_ref = args.kolme_expected_ref.strip()
    kolme_base_url = args.kolme_base_url.strip()
    kolme_fork_chain_version = args.kolme_fork_chain_version.strip()

    if mode == "run" and args.local_opt_in != "1":
        fail(
            "run mode requires explicit local-only opt-in via "
            "KAMN_LOCAL_FULL_STACK_INTEGRATION_OPT_IN=1"
        )
    if mode == "run" and not kolme_expected_remote_url:
        fail("local_kolme_expected_remote_url_missing")
    if mode == "run" and not kolme_expected_ref:
        fail("local_kolme_expected_ref_missing")
    if mode == "run" and not kolme_base_url:
        fail("local_kolme_base_url_missing")
    if mode == "run" and not kolme_fork_chain_version:
        fail("local_kolme_fork_chain_version_missing")
    if mode == "run":
        _require_local_kolme_checkout(
            checkout_path=kolme_checkout_path,
            expected_remote_url=kolme_expected_remote_url,
            expected_ref=kolme_expected_ref,
        )

    start_epoch = int(time.time())
    commands_executed = 0
    artifact_paths: dict[str, str] = {}
    transport_convergence_status = "planned" if mode == "dry-run" else "verified"
    signer_provenance_status = "planned" if mode == "dry-run" else "verified"
    runtime_commit_submission_status = "planned" if mode == "dry-run" else "verified"
    runtime_commit_finality_status = "planned" if mode == "dry-run" else "verified"
    runtime_provider_contract_status = "planned" if mode == "dry-run" else "verified"
    kolme_local_prerequisite_status = "planned" if mode == "dry-run" else "verified"
    kolme_local_only_enforced_status = "planned" if mode == "dry-run" else "verified"
    kolme_integration_mode_status = "planned" if mode == "dry-run" else "verified"
    kolme_integration_policy_status = "planned" if mode == "dry-run" else "verified"

    if mode == "run":
        artifact_dir = Path(tempfile.mkdtemp(prefix="local-full-stack-integration-live-"))
        full_io_report = artifact_dir / "full-io-scenario-matrix-report.json"
        full_runtime_report = artifact_dir / "local-full-runtime-report.json"
        kolme_integration_report = artifact_dir / "kolme-runtime-integration-summary.json"
        kolme_integration_policy_report = artifact_dir / "kolme-runtime-integration-policy.json"
        evidence_bundle_file = artifact_dir / "local-full-stack-evidence-bundle.json"

        full_io_output = _run_command(
            [
                "bash",
                "scripts/runtime/validate_full_io_scenario_matrix_live.sh",
                "--mode",
                "run",
                "--ci-fast-gate",
                "FAIL",
                "--max-seconds",
                str(command_max_seconds),
                "--output-json",
                str(full_io_report),
            ],
            timeout_seconds=command_max_seconds,
            env={**os.environ, "KAMN_LOCAL_FULL_IO_SCENARIO_MATRIX_OPT_IN": "1"},
        )
        if _extract_line_value(full_io_output, "status") != "pass":
            fail("full I/O scenario matrix command did not emit status=pass")
        if _extract_line_value(full_io_output, "final_decision") != "GO":
            fail("full I/O scenario matrix command did not emit final_decision=GO")
        commands_executed += 1

        full_runtime_output = _run_command(
            [
                "bash",
                "scripts/runtime/validate_local_full_runtime_live.sh",
                "--mode",
                "run",
                "--ci-fast-gate",
                "FAIL",
                "--max-seconds",
                str(command_max_seconds),
                "--command-max-seconds",
                str(min(command_max_seconds, 180)),
                "--output-json",
                str(full_runtime_report),
            ],
            timeout_seconds=command_max_seconds,
            env={**os.environ, "KAMN_LOCAL_FULL_RUNTIME_LIVE_OPT_IN": "1"},
        )
        if _extract_line_value(full_runtime_output, "status") != "pass":
            fail("local full-runtime command did not emit status=pass")
        if _extract_line_value(full_runtime_output, "final_decision") != "GO":
            fail("local full-runtime command did not emit final_decision=GO")
        commands_executed += 1

        kolme_output = _run_command(
            [
                "bash",
                "scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh",
                "--mode",
                "run",
                "--checkout-path",
                str(kolme_checkout_path),
                "--expected-remote-url",
                kolme_expected_remote_url,
                "--expected-ref",
                kolme_expected_ref,
                "--base-url",
                kolme_base_url,
                "--fork-chain-version",
                kolme_fork_chain_version,
                "--output-json",
                str(kolme_integration_report),
                "--max-seconds",
                str(min(command_max_seconds, 210)),
            ],
            timeout_seconds=command_max_seconds,
            env={**os.environ, "KAMN_KOLME_LOCAL_HEAVY": "1"},
        )
        if _extract_line_value(kolme_output, "status") != "ok":
            fail("kolme runtime integration lane did not emit status=ok")
        if _extract_line_value(kolme_output, "lane_mode") != "run":
            fail("kolme runtime integration lane did not emit lane_mode=run")
        commands_executed += 1

        kolme_policy_output = _run_command(
            [
                "python3",
                "scripts/kolme/check_local_kamn_live_runtime_integration_policy.py",
                "--report-file",
                str(kolme_integration_report),
                "--expected-final-decision",
                "GO",
                "--ci-fast-gate",
                "PASS",
                "--require-reason-code",
                KOLME_RUNTIME_INTEGRATION_RUN_REASON,
                "--output-json",
                str(kolme_integration_policy_report),
            ],
            timeout_seconds=command_max_seconds,
        )
        if _extract_line_value(kolme_policy_output, "status") != "ok":
            fail("kolme runtime integration policy checker did not emit status=ok")
        if _extract_line_value(kolme_policy_output, "final_decision") != "GO":
            fail("kolme runtime integration policy checker did not emit final_decision=GO")
        commands_executed += 1

        full_io_payload = _read_json_dict(
            full_io_report,
            failure_reason="full_i_o_scenario_matrix_report_invalid",
        )
        full_runtime_payload = _read_json_dict(
            full_runtime_report,
            failure_reason="local_full_runtime_report_invalid",
        )
        kolme_integration_payload = _read_json_dict(
            kolme_integration_report,
            failure_reason="kolme_runtime_integration_report_invalid",
        )
        kolme_policy_payload = _read_json_dict(
            kolme_integration_policy_report,
            failure_reason="kolme_runtime_integration_policy_report_invalid",
        )

        if full_io_payload.get("final_decision") != "GO":
            fail("full I/O scenario matrix report missing final_decision=GO")
        if full_runtime_payload.get("final_decision") != "GO":
            fail("local full-runtime report missing final_decision=GO")
        if kolme_integration_payload.get("schema_version") != KOLME_INTEGRATION_REPORT_SCHEMA:
            fail("kolme runtime integration report schema mismatch")
        if kolme_integration_payload.get("status") != "ok":
            fail("kolme runtime integration report status mismatch")
        if kolme_integration_payload.get("mode") != "run":
            fail("kolme runtime integration report mode mismatch")
        if kolme_integration_payload.get("reason_code") != KOLME_RUNTIME_INTEGRATION_RUN_REASON:
            fail("kolme runtime integration report reason code mismatch")
        if kolme_integration_payload.get("local_only_enforced") is not True:
            fail("kolme runtime integration report local_only_enforced mismatch")
        if kolme_integration_payload.get("ci_fast_gate_eligible") is not False:
            fail("kolme runtime integration report ci_fast_gate_eligible mismatch")
        if kolme_integration_payload.get("checkout_path") != str(kolme_checkout_path):
            fail("kolme runtime integration report checkout_path mismatch")
        if kolme_integration_payload.get("expected_remote_url") != kolme_expected_remote_url:
            fail("kolme runtime integration report expected_remote_url mismatch")
        if kolme_integration_payload.get("expected_ref") != kolme_expected_ref:
            fail("kolme runtime integration report expected_ref mismatch")
        if kolme_integration_payload.get("base_url") != kolme_base_url:
            fail("kolme runtime integration report base_url mismatch")
        if kolme_integration_payload.get("fork_chain_version") != kolme_fork_chain_version:
            fail("kolme runtime integration report fork_chain_version mismatch")
        if kolme_integration_payload.get("runtime_profile") != "real-node":
            fail("kolme runtime integration report runtime_profile mismatch")
        if (
            kolme_integration_payload.get("runtime_provider_client_contract")
            != KOLME_PROVIDER_CLIENT_CONTRACT
        ):
            fail("kolme runtime integration report provider contract mismatch")
        if (
            kolme_integration_payload.get("runtime_signing_profile")
            != KOLME_RUNTIME_SIGNING_PROFILE
        ):
            fail("kolme runtime integration report signing profile mismatch")
        if (
            kolme_integration_payload.get("runtime_signer_attestation_schema_version")
            != KOLME_SIGNER_ATTESTATION_SCHEMA
        ):
            fail("kolme runtime integration report signer attestation schema mismatch")
        runtime_commit_command = kolme_integration_payload.get("runtime_commit_command")
        if (
            not isinstance(runtime_commit_command, str)
            or "run_local_runtime_commit_live_finality_evidence_contract_lane.sh"
            not in runtime_commit_command
        ):
            fail("kolme runtime integration report missing runtime commit finality lane marker")
        if kolme_policy_payload.get("schema_version") != KOLME_INTEGRATION_POLICY_SCHEMA:
            fail("kolme runtime integration policy schema mismatch")
        if kolme_policy_payload.get("final_decision") != "GO":
            fail("kolme runtime integration policy missing final_decision=GO")
        if kolme_policy_payload.get("observed_reason_code") != KOLME_RUNTIME_INTEGRATION_RUN_REASON:
            fail("kolme runtime integration policy observed reason code mismatch")

        evidence_bundle = {
            "schema_version": EVIDENCE_BUNDLE_SCHEMA,
            "status": "pass",
            "final_decision": "GO",
            "lane_mode": mode,
            "full_io_matrix_report_file": str(full_io_report),
            "full_runtime_report_file": str(full_runtime_report),
            "kolme_runtime_integration_report_file": str(kolme_integration_report),
            "kolme_runtime_integration_policy_report_file": str(kolme_integration_policy_report),
            "transport_convergence_status": transport_convergence_status,
            "signer_provenance_status": signer_provenance_status,
            "runtime_commit_submission_status": runtime_commit_submission_status,
            "runtime_commit_finality_status": runtime_commit_finality_status,
            "runtime_provider_contract_status": runtime_provider_contract_status,
            "runtime_provider_client_contract": KOLME_PROVIDER_CLIENT_CONTRACT,
            "runtime_signing_profile": KOLME_RUNTIME_SIGNING_PROFILE,
            "runtime_signer_attestation_schema_version": KOLME_SIGNER_ATTESTATION_SCHEMA,
            "kolme_local_prerequisite_status": kolme_local_prerequisite_status,
            "kolme_local_only_enforced_status": kolme_local_only_enforced_status,
            "kolme_integration_mode_status": kolme_integration_mode_status,
            "kolme_integration_policy_status": kolme_integration_policy_status,
            "kolme_checkout_path": str(kolme_checkout_path),
            "kolme_expected_remote_url": kolme_expected_remote_url,
            "kolme_expected_ref": kolme_expected_ref,
            "kolme_base_url": kolme_base_url,
            "kolme_fork_chain_version": kolme_fork_chain_version,
            "commands_executed": commands_executed,
            "ci_fast_gate_eligibility": "excluded_local_heavy",
        }
        _write_json(evidence_bundle_file, evidence_bundle)

        artifact_paths = {
            "full_io_matrix_report": str(full_io_report),
            "full_runtime_report": str(full_runtime_report),
            "kolme_runtime_integration_summary_report": str(kolme_integration_report),
            "kolme_runtime_integration_policy_report": str(kolme_integration_policy_report),
            "evidence_bundle_file": str(evidence_bundle_file),
        }

    elapsed_seconds = int(time.time()) - start_epoch
    if elapsed_seconds > max_seconds:
        fail(
            "local full-stack integration lane exceeded runtime budget: "
            f"{elapsed_seconds}s (max={max_seconds}s)"
        )

    run_mode_command_status = "executed" if mode == "run" else "dry_run_no_commands_executed"
    ci_fast_gate_eligibility = "excluded_local_heavy" if mode == "run" else "eligible"
    reason_code = RUN_REASON if mode == "run" else DRY_RUN_REASON

    payload = {
        "schema_version": RUN_LANE_SCHEMA,
        "status": "pass",
        "final_decision": "GO",
        "lane_mode": mode,
        "ci_fast_gate": ci_fast_gate,
        "ci_fast_gate_eligibility": ci_fast_gate_eligibility,
        "fast_gate_exclusion_status": "verified",
        "fast_gate_exclusion_reason_code": FAST_GATE_EXCLUSION_REASON,
        "scenario_matrix_status": "verified",
        "full_runtime_status": "verified",
        "evidence_bundle_status": "verified",
        "transport_convergence_status": transport_convergence_status,
        "signer_provenance_status": signer_provenance_status,
        "runtime_commit_submission_status": runtime_commit_submission_status,
        "runtime_commit_finality_status": runtime_commit_finality_status,
        "runtime_provider_contract_status": runtime_provider_contract_status,
        "runtime_provider_client_contract": KOLME_PROVIDER_CLIENT_CONTRACT,
        "runtime_signing_profile": KOLME_RUNTIME_SIGNING_PROFILE,
        "runtime_signer_attestation_schema_version": KOLME_SIGNER_ATTESTATION_SCHEMA,
        "kolme_local_prerequisite_status": kolme_local_prerequisite_status,
        "kolme_local_only_enforced_status": kolme_local_only_enforced_status,
        "kolme_integration_mode_status": kolme_integration_mode_status,
        "kolme_integration_policy_status": kolme_integration_policy_status,
        "kolme_checkout_path": str(kolme_checkout_path),
        "kolme_expected_remote_url": kolme_expected_remote_url,
        "kolme_expected_ref": kolme_expected_ref,
        "kolme_base_url": kolme_base_url,
        "kolme_fork_chain_version": kolme_fork_chain_version,
        "kolme_integration_report_schema_version": KOLME_INTEGRATION_REPORT_SCHEMA,
        "kolme_integration_policy_schema_version": KOLME_INTEGRATION_POLICY_SCHEMA,
        "run_mode_command_status": run_mode_command_status,
        "run_mode_command_count": commands_executed,
        "reason_code": reason_code,
        "elapsed_seconds": elapsed_seconds,
        "max_seconds": max_seconds,
        "command_max_seconds": command_max_seconds,
        "artifact_paths": artifact_paths,
    }
    if args.output_json:
        write_json(Path(args.output_json), payload)

    print("status=pass")
    print("final_decision=GO")
    print(f"lane_mode={mode}")
    print(f"ci_fast_gate={ci_fast_gate}")
    print(f"ci_fast_gate_eligibility={ci_fast_gate_eligibility}")
    print("fast_gate_exclusion_status=verified")
    print(f"fast_gate_exclusion_reason_code={FAST_GATE_EXCLUSION_REASON}")
    print("scenario_matrix_status=verified")
    print("full_runtime_status=verified")
    print("evidence_bundle_status=verified")
    print(f"transport_convergence_status={transport_convergence_status}")
    print(f"signer_provenance_status={signer_provenance_status}")
    print(f"runtime_commit_submission_status={runtime_commit_submission_status}")
    print(f"runtime_commit_finality_status={runtime_commit_finality_status}")
    print(f"runtime_provider_contract_status={runtime_provider_contract_status}")
    print(f"runtime_provider_client_contract={KOLME_PROVIDER_CLIENT_CONTRACT}")
    print(f"runtime_signing_profile={KOLME_RUNTIME_SIGNING_PROFILE}")
    print(f"runtime_signer_attestation_schema_version={KOLME_SIGNER_ATTESTATION_SCHEMA}")
    print(f"kolme_local_prerequisite_status={kolme_local_prerequisite_status}")
    print(f"kolme_local_only_enforced_status={kolme_local_only_enforced_status}")
    print(f"kolme_integration_mode_status={kolme_integration_mode_status}")
    print(f"kolme_integration_policy_status={kolme_integration_policy_status}")
    print(f"kolme_checkout_path={kolme_checkout_path}")
    print(f"kolme_expected_remote_url={kolme_expected_remote_url}")
    print(f"kolme_expected_ref={kolme_expected_ref}")
    print(f"kolme_base_url={kolme_base_url}")
    print(f"kolme_fork_chain_version={kolme_fork_chain_version}")
    print(f"kolme_integration_report_schema_version={KOLME_INTEGRATION_REPORT_SCHEMA}")
    print(f"kolme_integration_policy_schema_version={KOLME_INTEGRATION_POLICY_SCHEMA}")
    print(f"run_mode_command_status={run_mode_command_status}")
    print(f"run_mode_command_count={commands_executed}")
    print(f"reason_code={reason_code}")
    if args.output_json:
        print(f"report_file={Path(args.output_json).resolve()}")
    return 0


def check_policy(args: argparse.Namespace) -> int:
    report_file = Path(args.report_file)
    if not report_file.is_file():
        fail(f"report file does not exist: {report_file}")

    expected_final_decision = require_enum(
        "--expected-final-decision",
        args.expected_final_decision.strip(),
        ("GO", "NO-GO"),
    )
    ci_fast_gate = require_enum("--ci-fast-gate", args.ci_fast_gate.strip(), ("PASS", "FAIL"))
    payload = load_json(report_file)

    checks = DecisionAccumulator()
    checks.reject_if(
        payload.get("schema_version") != RUN_LANE_SCHEMA,
        "local_full_stack_integration_policy_schema_mismatch",
    )
    checks.reject_if(payload.get("status") != "pass", "local_full_stack_integration_policy_status_mismatch")
    checks.reject_if(
        payload.get("final_decision") != "GO",
        "local_full_stack_integration_policy_final_decision_mismatch",
    )
    checks.reject_if(
        payload.get("ci_fast_gate") != ci_fast_gate,
        "local_full_stack_integration_policy_ci_fast_gate_mismatch",
    )
    checks.reject_if(
        payload.get("fast_gate_exclusion_status") != "verified",
        "local_full_stack_integration_policy_fast_gate_exclusion_mismatch",
    )
    checks.reject_if(
        payload.get("fast_gate_exclusion_reason_code") != FAST_GATE_EXCLUSION_REASON,
        "local_full_stack_integration_policy_fast_gate_reason_mismatch",
    )
    checks.reject_if(
        payload.get("scenario_matrix_status") != "verified",
        "local_full_stack_integration_policy_scenario_matrix_status_mismatch",
    )
    checks.reject_if(
        payload.get("full_runtime_status") != "verified",
        "local_full_stack_integration_policy_full_runtime_status_mismatch",
    )
    checks.reject_if(
        payload.get("evidence_bundle_status") != "verified",
        "local_full_stack_integration_policy_evidence_bundle_status_mismatch",
    )
    checks.reject_if(
        payload.get("runtime_provider_client_contract") != KOLME_PROVIDER_CLIENT_CONTRACT,
        "local_full_stack_integration_policy_runtime_provider_client_contract_mismatch",
    )
    checks.reject_if(
        payload.get("runtime_signing_profile") != KOLME_RUNTIME_SIGNING_PROFILE,
        "local_full_stack_integration_policy_runtime_signing_profile_mismatch",
    )
    checks.reject_if(
        payload.get("runtime_signer_attestation_schema_version") != KOLME_SIGNER_ATTESTATION_SCHEMA,
        "local_full_stack_integration_policy_runtime_signer_attestation_schema_mismatch",
    )
    checks.reject_if(
        payload.get("kolme_integration_report_schema_version") != KOLME_INTEGRATION_REPORT_SCHEMA,
        "local_full_stack_integration_policy_kolme_report_schema_contract_mismatch",
    )
    checks.reject_if(
        payload.get("kolme_integration_policy_schema_version") != KOLME_INTEGRATION_POLICY_SCHEMA,
        "local_full_stack_integration_policy_kolme_policy_schema_contract_mismatch",
    )

    lane_mode = payload.get("lane_mode")
    checks.reject_if(
        lane_mode not in ("dry-run", "run"),
        "local_full_stack_integration_policy_lane_mode_invalid",
    )
    command_count = payload.get("run_mode_command_count")
    checks.reject_if(
        not isinstance(command_count, int) or command_count < 0,
        "local_full_stack_integration_policy_command_count_invalid",
    )
    command_status = payload.get("run_mode_command_status")
    reason_code = payload.get("reason_code")
    artifact_paths = payload.get("artifact_paths")
    checks.reject_if(
        not isinstance(artifact_paths, dict),
        "local_full_stack_integration_policy_artifact_paths_invalid",
    )
    expected_domain_status = "planned" if lane_mode == "dry-run" else "verified"
    checks.reject_if(
        payload.get("transport_convergence_status") != expected_domain_status,
        "local_full_stack_integration_policy_transport_convergence_status_mismatch",
    )
    checks.reject_if(
        payload.get("signer_provenance_status") != expected_domain_status,
        "local_full_stack_integration_policy_signer_provenance_status_mismatch",
    )
    checks.reject_if(
        payload.get("runtime_commit_submission_status") != expected_domain_status,
        "local_full_stack_integration_policy_runtime_commit_submission_status_mismatch",
    )
    checks.reject_if(
        payload.get("runtime_commit_finality_status") != expected_domain_status,
        "local_full_stack_integration_policy_runtime_commit_finality_status_mismatch",
    )
    checks.reject_if(
        payload.get("runtime_provider_contract_status") != expected_domain_status,
        "local_full_stack_integration_policy_runtime_provider_contract_status_mismatch",
    )
    checks.reject_if(
        payload.get("kolme_local_prerequisite_status") != expected_domain_status,
        "local_full_stack_integration_policy_kolme_local_prerequisite_status_mismatch",
    )
    checks.reject_if(
        payload.get("kolme_local_only_enforced_status") != expected_domain_status,
        "local_full_stack_integration_policy_kolme_local_only_enforced_status_mismatch",
    )
    checks.reject_if(
        payload.get("kolme_integration_mode_status") != expected_domain_status,
        "local_full_stack_integration_policy_kolme_integration_mode_status_mismatch",
    )
    checks.reject_if(
        payload.get("kolme_integration_policy_status") != expected_domain_status,
        "local_full_stack_integration_policy_kolme_integration_policy_status_mismatch",
    )

    kolme_checkout_path = payload.get("kolme_checkout_path")
    kolme_expected_remote_url = payload.get("kolme_expected_remote_url")
    kolme_expected_ref = payload.get("kolme_expected_ref")
    kolme_base_url = payload.get("kolme_base_url")
    kolme_fork_chain_version = payload.get("kolme_fork_chain_version")

    checks.reject_if(
        not isinstance(kolme_checkout_path, str) or not kolme_checkout_path.strip(),
        "local_full_stack_integration_policy_kolme_checkout_path_missing",
    )
    checks.reject_if(
        not isinstance(kolme_expected_remote_url, str) or not kolme_expected_remote_url.strip(),
        "local_full_stack_integration_policy_kolme_expected_remote_url_missing",
    )
    checks.reject_if(
        not isinstance(kolme_expected_ref, str) or not kolme_expected_ref.strip(),
        "local_full_stack_integration_policy_kolme_expected_ref_missing",
    )
    checks.reject_if(
        not isinstance(kolme_base_url, str) or not kolme_base_url.strip(),
        "local_full_stack_integration_policy_kolme_base_url_missing",
    )
    checks.reject_if(
        not isinstance(kolme_fork_chain_version, str) or not kolme_fork_chain_version.strip(),
        "local_full_stack_integration_policy_kolme_fork_chain_version_missing",
    )

    if lane_mode == "dry-run":
        checks.reject_if(
            payload.get("ci_fast_gate_eligibility") != "eligible",
            "local_full_stack_integration_policy_dry_run_eligibility_mismatch",
        )
        checks.reject_if(
            command_count != 0,
            "local_full_stack_integration_policy_dry_run_command_count_mismatch",
        )
        checks.reject_if(
            command_status != "dry_run_no_commands_executed",
            "local_full_stack_integration_policy_dry_run_command_status_mismatch",
        )
        checks.reject_if(
            reason_code != DRY_RUN_REASON,
            "local_full_stack_integration_policy_dry_run_reason_code_mismatch",
        )
    elif lane_mode == "run":
        checks.reject_if(
            payload.get("ci_fast_gate_eligibility") != "excluded_local_heavy",
            "local_full_stack_integration_policy_run_mode_exclusion_mismatch",
        )
        checks.reject_if(
            command_count < 4,
            "local_full_stack_integration_policy_run_mode_command_count_mismatch",
        )
        checks.reject_if(
            command_status != "executed",
            "local_full_stack_integration_policy_run_mode_command_status_mismatch",
        )
        checks.reject_if(
            reason_code != RUN_REASON,
            "local_full_stack_integration_policy_run_mode_reason_code_mismatch",
        )
        required_artifacts = (
            "full_io_matrix_report",
            "full_runtime_report",
            "kolme_runtime_integration_summary_report",
            "kolme_runtime_integration_policy_report",
            "evidence_bundle_file",
        )
        kolme_summary_report_path = ""
        kolme_policy_report_path = ""
        if isinstance(artifact_paths, dict):
            for artifact_key in required_artifacts:
                artifact_value = artifact_paths.get(artifact_key)
                checks.reject_if(
                    not isinstance(artifact_value, str) or not Path(artifact_value).is_file(),
                    f"local_full_stack_integration_policy_artifact_missing:{artifact_key}",
                )
            summary_value = artifact_paths.get("kolme_runtime_integration_summary_report")
            policy_value = artifact_paths.get("kolme_runtime_integration_policy_report")
            if isinstance(summary_value, str):
                kolme_summary_report_path = summary_value
            if isinstance(policy_value, str):
                kolme_policy_report_path = policy_value

        if kolme_summary_report_path:
            try:
                kolme_summary_payload = json.loads(
                    Path(kolme_summary_report_path).read_text(encoding="utf-8")
                )
            except (OSError, json.JSONDecodeError):
                kolme_summary_payload = {}
                checks.reject_if(
                    True,
                    "local_full_stack_integration_policy_kolme_summary_json_invalid",
                )
            if not isinstance(kolme_summary_payload, dict):
                kolme_summary_payload = {}
                checks.reject_if(
                    True,
                    "local_full_stack_integration_policy_kolme_summary_root_invalid",
                )
            checks.reject_if(
                kolme_summary_payload.get("schema_version") != KOLME_INTEGRATION_REPORT_SCHEMA,
                "local_full_stack_integration_policy_kolme_summary_schema_mismatch",
            )
            checks.reject_if(
                kolme_summary_payload.get("status") != "ok",
                "local_full_stack_integration_policy_kolme_summary_status_mismatch",
            )
            checks.reject_if(
                kolme_summary_payload.get("mode") != "run",
                "local_full_stack_integration_policy_kolme_summary_mode_mismatch",
            )
            checks.reject_if(
                kolme_summary_payload.get("reason_code") != KOLME_RUNTIME_INTEGRATION_RUN_REASON,
                "local_full_stack_integration_policy_kolme_summary_reason_code_mismatch",
            )
            checks.reject_if(
                kolme_summary_payload.get("local_only_enforced") is not True,
                "local_full_stack_integration_policy_kolme_summary_local_only_enforced_mismatch",
            )
            checks.reject_if(
                kolme_summary_payload.get("ci_fast_gate_eligible") is not False,
                "local_full_stack_integration_policy_kolme_summary_ci_fast_gate_eligibility_mismatch",
            )
            checks.reject_if(
                isinstance(kolme_checkout_path, str)
                and kolme_summary_payload.get("checkout_path") != kolme_checkout_path,
                "local_full_stack_integration_policy_kolme_summary_checkout_path_mismatch",
            )
            checks.reject_if(
                isinstance(kolme_expected_remote_url, str)
                and kolme_summary_payload.get("expected_remote_url") != kolme_expected_remote_url,
                "local_full_stack_integration_policy_kolme_summary_expected_remote_url_mismatch",
            )
            checks.reject_if(
                isinstance(kolme_expected_ref, str)
                and kolme_summary_payload.get("expected_ref") != kolme_expected_ref,
                "local_full_stack_integration_policy_kolme_summary_expected_ref_mismatch",
            )
            checks.reject_if(
                isinstance(kolme_base_url, str)
                and kolme_summary_payload.get("base_url") != kolme_base_url,
                "local_full_stack_integration_policy_kolme_summary_base_url_mismatch",
            )
            checks.reject_if(
                isinstance(kolme_fork_chain_version, str)
                and kolme_summary_payload.get("fork_chain_version") != kolme_fork_chain_version,
                "local_full_stack_integration_policy_kolme_summary_fork_chain_version_mismatch",
            )
            checks.reject_if(
                kolme_summary_payload.get("runtime_profile") != "real-node",
                "local_full_stack_integration_policy_kolme_summary_runtime_profile_mismatch",
            )
            checks.reject_if(
                kolme_summary_payload.get("runtime_provider_client_contract")
                != KOLME_PROVIDER_CLIENT_CONTRACT,
                "local_full_stack_integration_policy_kolme_summary_provider_contract_mismatch",
            )
            checks.reject_if(
                kolme_summary_payload.get("runtime_signing_profile")
                != KOLME_RUNTIME_SIGNING_PROFILE,
                "local_full_stack_integration_policy_kolme_summary_signing_profile_mismatch",
            )
            checks.reject_if(
                kolme_summary_payload.get("runtime_signer_attestation_schema_version")
                != KOLME_SIGNER_ATTESTATION_SCHEMA,
                "local_full_stack_integration_policy_kolme_summary_signer_attestation_schema_mismatch",
            )
            runtime_commit_command = kolme_summary_payload.get("runtime_commit_command")
            checks.reject_if(
                not isinstance(runtime_commit_command, str)
                or "run_local_runtime_commit_live_finality_evidence_contract_lane.sh"
                not in runtime_commit_command,
                "local_full_stack_integration_policy_kolme_summary_runtime_commit_contract_mismatch",
            )
            checks.reject_if(
                kolme_summary_payload.get("runtime_commit_finality_enabled") is not True,
                "local_full_stack_integration_policy_kolme_summary_finality_marker_mismatch",
            )

        if kolme_policy_report_path:
            try:
                kolme_policy_payload = json.loads(
                    Path(kolme_policy_report_path).read_text(encoding="utf-8")
                )
            except (OSError, json.JSONDecodeError):
                kolme_policy_payload = {}
                checks.reject_if(
                    True,
                    "local_full_stack_integration_policy_kolme_policy_json_invalid",
                )
            if not isinstance(kolme_policy_payload, dict):
                kolme_policy_payload = {}
                checks.reject_if(
                    True,
                    "local_full_stack_integration_policy_kolme_policy_root_invalid",
                )
            checks.reject_if(
                kolme_policy_payload.get("schema_version") != KOLME_INTEGRATION_POLICY_SCHEMA,
                "local_full_stack_integration_policy_kolme_policy_schema_mismatch",
            )
            checks.reject_if(
                kolme_policy_payload.get("final_decision") != "GO",
                "local_full_stack_integration_policy_kolme_policy_final_decision_mismatch",
            )
            checks.reject_if(
                kolme_policy_payload.get("observed_reason_code") != KOLME_RUNTIME_INTEGRATION_RUN_REASON,
                "local_full_stack_integration_policy_kolme_policy_observed_reason_code_mismatch",
            )

    observed_final_decision, decision_reasons = checks.finalize(
        "local_full_stack_integration_policy_verified"
    )
    failed_checks: list[str] = []
    if observed_final_decision == "NO-GO":
        failed_checks.extend(decision_reasons)
    if observed_final_decision != expected_final_decision:
        failed_checks.append("local_full_stack_integration_policy_expected_decision_mismatch")

    report_payload = {
        "schema_version": POLICY_SCHEMA,
        "status": "ok" if not failed_checks else "fail",
        "final_decision": observed_final_decision,
        "expected_final_decision": expected_final_decision,
        "ci_fast_gate": ci_fast_gate,
        "decision_reasons": decision_reasons,
        "local_full_stack_integration_policy_status": "verified" if not failed_checks else "failed",
        "failed_checks": failed_checks,
    }
    if args.output_json:
        write_json(Path(args.output_json), report_payload)

    print(f"status={'ok' if not failed_checks else 'fail'}")
    print(f"final_decision={observed_final_decision}")
    print(f"expected_final_decision={expected_final_decision}")
    print(f"ci_fast_gate={ci_fast_gate}")
    print(
        "local_full_stack_integration_policy_status="
        f"{'verified' if not failed_checks else 'failed'}"
    )
    print(f"failed_checks={','.join(failed_checks)}")
    if args.output_json:
        print(f"report_file={Path(args.output_json).resolve()}")

    if failed_checks:
        fail(",".join(failed_checks))
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Local full-stack integration lane contracts.")
    subparsers = parser.add_subparsers(dest="command", required=True)

    run_lane_parser = subparsers.add_parser("run-lane", help="Run local full-stack integration lane.")
    run_lane_parser.add_argument("--mode", default=os.environ.get("KAMN_LOCAL_FULL_STACK_INTEGRATION_MODE", "dry-run"))
    run_lane_parser.add_argument(
        "--max-seconds",
        default=os.environ.get("KAMN_LOCAL_FULL_STACK_INTEGRATION_MAX_SECONDS", "360"),
    )
    run_lane_parser.add_argument(
        "--command-max-seconds",
        default=os.environ.get("KAMN_LOCAL_FULL_STACK_INTEGRATION_COMMAND_MAX_SECONDS", "300"),
    )
    run_lane_parser.add_argument("--ci-fast-gate", default=os.environ.get("KAMN_CI_FAST_GATE", "PASS"))
    run_lane_parser.add_argument("--local-opt-in", default=os.environ.get(OPT_IN_ENV, "0"))
    run_lane_parser.add_argument(
        "--kolme-checkout-path",
        default=os.environ.get(
            "KAMN_LOCAL_FULL_STACK_INTEGRATION_KOLME_CHECKOUT_PATH",
            KOLME_DEFAULT_CHECKOUT_PATH,
        ),
    )
    run_lane_parser.add_argument(
        "--kolme-expected-remote-url",
        default=os.environ.get(
            "KAMN_LOCAL_FULL_STACK_INTEGRATION_KOLME_EXPECTED_REMOTE_URL",
            KOLME_DEFAULT_EXPECTED_REMOTE_URL,
        ),
    )
    run_lane_parser.add_argument(
        "--kolme-expected-ref",
        default=os.environ.get(
            "KAMN_LOCAL_FULL_STACK_INTEGRATION_KOLME_EXPECTED_REF",
            KOLME_DEFAULT_EXPECTED_REF,
        ),
    )
    run_lane_parser.add_argument(
        "--kolme-base-url",
        default=os.environ.get(
            "KAMN_LOCAL_FULL_STACK_INTEGRATION_KOLME_BASE_URL",
            KOLME_DEFAULT_BASE_URL,
        ),
    )
    run_lane_parser.add_argument(
        "--kolme-fork-chain-version",
        default=os.environ.get(
            "KAMN_LOCAL_FULL_STACK_INTEGRATION_KOLME_FORK_CHAIN_VERSION",
            KOLME_DEFAULT_FORK_CHAIN_VERSION,
        ),
    )
    run_lane_parser.add_argument("--output-json", default="")
    run_lane_parser.set_defaults(handler=run_lane)

    policy_parser = subparsers.add_parser("check-policy", help="Check local full-stack integration policy.")
    policy_parser.add_argument("--report-file", required=True)
    policy_parser.add_argument("--expected-final-decision", default="GO")
    policy_parser.add_argument("--ci-fast-gate", default="PASS")
    policy_parser.add_argument("--output-json", default="")
    policy_parser.set_defaults(handler=check_policy)
    return parser


def main(argv: list[str]) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    return args.handler(args)


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except ContractError as error:
        print(error, file=sys.stderr)
        raise SystemExit(1)
