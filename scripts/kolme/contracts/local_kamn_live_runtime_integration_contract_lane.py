#!/usr/bin/env python3
"""Contract lane runner for local KAMN live runtime integration checks."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parents[3]
RUNNER = ROOT_DIR / "scripts/kolme/run_local_kamn_live_runtime_integration_lane.sh"
CHECKER = ROOT_DIR / "scripts/kolme/check_local_kamn_live_runtime_integration_policy.py"
DOC_FILE = ROOT_DIR / "docs/planning/kolme-devnet-ops.md"
README_FILE = ROOT_DIR / "README.md"
FALLBACK_SIGNER_PRIVATE_KEY_ENV = "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK"


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Run local KAMN live runtime integration contract lane checks."
    )
    parser.add_argument(
        "--output-json",
        default="/tmp/kolme-local-kamn-live-runtime-integration-summary.json",
        help="Runtime integration summary output.",
    )
    parser.add_argument(
        "--policy-output-json",
        default="/tmp/kolme-local-kamn-live-runtime-integration-policy.json",
        help="Policy checker report output.",
    )
    parser.add_argument(
        "--max-seconds",
        default="210",
        help="Total runtime budget in seconds.",
    )
    parser.add_argument(
        "--fork-chain-version",
        default="v0.15.2",
        help="Required fork-info chain_version query value.",
    )
    return parser


def main() -> int:
    args = build_parser().parse_args()

    if not args.max_seconds.isdigit() or int(args.max_seconds) <= 0:
        print("max-seconds must be a positive integer", file=sys.stderr)
        return 1
    max_seconds = int(args.max_seconds)

    if not RUNNER.is_file() or not RUNNER.stat().st_mode & 0o111:
        print("expected local KAMN live runtime integration runner to be executable", file=sys.stderr)
        return 1
    if not CHECKER.is_file() or not CHECKER.stat().st_mode & 0o111:
        print("expected local KAMN live runtime integration policy checker to be executable", file=sys.stderr)
        return 1
    if not DOC_FILE.is_file():
        print("expected Kolme devnet ops documentation to exist", file=sys.stderr)
        return 1
    if not README_FILE.is_file():
        print("expected README to exist", file=sys.stderr)
        return 1

    start_epoch = time.monotonic()

    with tempfile.TemporaryDirectory(prefix="kolme-live-runtime-integration-") as temp_dir:
        temp_path = Path(temp_dir)
        checkout_path = temp_path / "kolme_fork"
        runtime_commit_output_file = temp_path / "runtime_commit_endpoint.log"
        checkout_path.mkdir(parents=True, exist_ok=True)

        subprocess.run(["git", "-C", str(checkout_path), "init", "-q"], check=True)
        subprocess.run(["git", "-C", str(checkout_path), "checkout", "-q", "-b", "main"], check=True)
        subprocess.run(["git", "-C", str(checkout_path), "config", "user.email", "ci@example.com"], check=True)
        subprocess.run(["git", "-C", str(checkout_path), "config", "user.name", "CI Runner"], check=True)
        (checkout_path / "README.md").write_text(
            "local KAMN live runtime integration fixture\n",
            encoding="utf-8",
        )
        subprocess.run(["git", "-C", str(checkout_path), "add", "README.md"], check=True)
        subprocess.run(
            [
                "git",
                "-C",
                str(checkout_path),
                "commit",
                "-q",
                "-m",
                "init runtime integration fixture",
            ],
            check=True,
        )
        subprocess.run(
            [
                "git",
                "-C",
                str(checkout_path),
                "remote",
                "add",
                "origin",
                "https://github.com/njfio/kolme_fork.git",
            ],
            check=True,
        )

        subprocess.run(
            [
                "bash",
                str(RUNNER),
                "--mode",
                "dry-run",
                "--checkout-path",
                str(checkout_path),
                "--expected-remote-url",
                "https://github.com/njfio/kolme_fork.git",
                "--expected-ref",
                "refs/heads/main",
                "--base-url",
                "http://127.0.0.1:3000",
                "--fork-chain-version",
                args.fork_chain_version,
                "--max-seconds",
                str(max_seconds),
                "--localhost-signed-max-seconds",
                "45",
                "--runtime-commit-output-file",
                str(runtime_commit_output_file),
                "--runtime-provider-client-contract",
                "KolmeRuntimeCommitLiveProvider",
                "--output-json",
                args.output_json,
            ],
            cwd=ROOT_DIR,
            check=True,
            stdout=subprocess.DEVNULL,
        )

        subprocess.run(
            [
                "python3",
                str(CHECKER),
                "--report-file",
                args.output_json,
                "--expected-final-decision",
                "GO",
                "--ci-fast-gate",
                "PASS",
                "--require-reason-code",
                "dry_run_no_commands_executed",
                "--output-json",
                args.policy_output_json,
            ],
            cwd=ROOT_DIR,
            check=True,
            stdout=subprocess.DEVNULL,
        )

        summary_payload = json.loads(Path(args.output_json).read_text(encoding="utf-8"))
        if summary_payload.get("runtime_commit_failure_taxonomy_version") != "v1":
            print("expected runtime commit failure taxonomy version marker v1 in summary", file=sys.stderr)
            return 1
        if summary_payload.get("runtime_commit_failure_taxonomy") != "none":
            print("expected dry-run summary to classify runtime commit failure taxonomy as none", file=sys.stderr)
            return 1
        if summary_payload.get("runtime_commit_nested_reason_code") != "not_run":
            print("expected dry-run summary to classify nested runtime reason as not_run", file=sys.stderr)
            return 1
        diagnostic_hint = summary_payload.get("runtime_commit_failure_diagnostic_hint")
        if not isinstance(diagnostic_hint, str) or not diagnostic_hint.strip():
            print("expected runtime commit failure diagnostic hint marker in summary", file=sys.stderr)
            return 1
        if summary_payload.get("runtime_signer_fallback_private_key_env") != FALLBACK_SIGNER_PRIVATE_KEY_ENV:
            print("expected fallback signer private key env marker in summary", file=sys.stderr)
            return 1
        if summary_payload.get("runtime_signer_fallback_private_key_present") is not False:
            print("expected fallback signer private key presence marker false in dry-run summary", file=sys.stderr)
            return 1
        contracts_payload = summary_payload.get("contracts")
        if not isinstance(contracts_payload, dict):
            print("expected contracts object in dry-run summary", file=sys.stderr)
            return 1
        if contracts_payload.get("runtime_signer_fallback_private_key_env") != FALLBACK_SIGNER_PRIVATE_KEY_ENV:
            print("expected contracts fallback signer private key env marker in summary", file=sys.stderr)
            return 1
        if contracts_payload.get("runtime_signer_fallback_private_key_allowed") is not False:
            print("expected contracts fallback signer private key allowed=false marker in summary", file=sys.stderr)
            return 1
        checks_payload = summary_payload.get("checks")
        if not isinstance(checks_payload, list):
            print("expected checks list in dry-run summary", file=sys.stderr)
            return 1
        fallback_checks = [
            check
            for check in checks_payload
            if isinstance(check, dict)
            and check.get("id") == "runtime_signer_fallback_private_key_contract"
        ]
        if len(fallback_checks) != 1:
            print("expected one runtime_signer_fallback_private_key_contract check in summary", file=sys.stderr)
            return 1
        if fallback_checks[0].get("status") != "planned":
            print("expected fallback signer check planned in dry-run summary", file=sys.stderr)
            return 1

        # Regression: #2296
        failure_payload = dict(summary_payload)
        failure_payload["mode"] = "run"
        failure_payload["status"] = "fail"
        failure_payload["reason_code"] = "runtime_commit_endpoint_failed"
        failure_payload["budget_status"] = "within_budget"
        failure_payload["runtime_commit_reason_code"] = "runtime_commit_endpoint_failed"
        failure_payload["runtime_commit_policy_reason_code"] = "runtime_commit_endpoint_failed"
        failure_payload["runtime_commit_nested_reason_code"] = "live_finality_command_timeout"
        failure_payload["runtime_commit_failure_taxonomy"] = "finality.timeout"
        failure_payload["runtime_commit_failure_diagnostic_hint"] = (
            "Inspect runtime finality command output and verify notifications/block fallback endpoint contracts."
        )
        failure_report = temp_path / "runtime_failure_summary.json"
        failure_policy_output = temp_path / "runtime_failure_policy.json"
        failure_report.write_text(json.dumps(failure_payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")

        subprocess.run(
            [
                "python3",
                str(CHECKER),
                "--report-file",
                str(failure_report),
                "--expected-final-decision",
                "NO-GO",
                "--ci-fast-gate",
                "PASS",
                "--require-reason-code",
                "runtime_commit_endpoint_failed",
                "--output-json",
                str(failure_policy_output),
            ],
            cwd=ROOT_DIR,
            check=True,
            stdout=subprocess.DEVNULL,
        )

        taxonomy_drift_payload = dict(failure_payload)
        taxonomy_drift_payload["runtime_commit_failure_taxonomy"] = "transport.submit.failed"
        taxonomy_drift_report = temp_path / "runtime_failure_taxonomy_drift_summary.json"
        taxonomy_drift_report.write_text(
            json.dumps(taxonomy_drift_payload, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )

        taxonomy_drift_run = subprocess.run(
            [
                "python3",
                str(CHECKER),
                "--report-file",
                str(taxonomy_drift_report),
                "--expected-final-decision",
                "NO-GO",
                "--ci-fast-gate",
                "PASS",
                "--require-reason-code",
                "runtime_commit_endpoint_failed",
                "--output-json",
                str(failure_policy_output),
            ],
            cwd=ROOT_DIR,
            check=False,
            capture_output=True,
            text=True,
        )
        if taxonomy_drift_run.returncode == 0:
            print("expected checker to fail when runtime commit failure taxonomy drifts", file=sys.stderr)
            return 1
        drift_output = f"{taxonomy_drift_run.stdout}\n{taxonomy_drift_run.stderr}"
        if "runtime_commit_failure_taxonomy_mismatch:finality.timeout" not in drift_output:
            print(
                "expected runtime commit taxonomy mismatch reason for drifted failure taxonomy",
                file=sys.stderr,
            )
            return 1

        # Regression: #2298
        simulated_profile_payload = dict(failure_payload)
        simulated_profile_payload["runtime_profile"] = "standard"
        contracts = simulated_profile_payload.get("contracts")
        if isinstance(contracts, dict):
            contracts["runtime_profile"] = "standard"
        simulated_profile_report = temp_path / "runtime_failure_simulated_profile_summary.json"
        simulated_profile_report.write_text(
            json.dumps(simulated_profile_payload, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )

        simulated_profile_run = subprocess.run(
            [
                "python3",
                str(CHECKER),
                "--report-file",
                str(simulated_profile_report),
                "--expected-final-decision",
                "NO-GO",
                "--ci-fast-gate",
                "PASS",
                "--require-reason-code",
                "runtime_commit_endpoint_failed",
                "--output-json",
                str(failure_policy_output),
            ],
            cwd=ROOT_DIR,
            check=False,
            capture_output=True,
            text=True,
        )
        if simulated_profile_run.returncode == 0:
            print("expected checker to fail when run-mode uses simulated standard runtime profile", file=sys.stderr)
            return 1
        simulated_profile_output = f"{simulated_profile_run.stdout}\n{simulated_profile_run.stderr}"
        if "runtime_profile_run_mode_mismatch" not in simulated_profile_output:
            print(
                "expected run-mode simulated profile mismatch reason for policy failure",
                file=sys.stderr,
            )
            return 1

        # Regression: #2302
        fallback_violation_payload = dict(summary_payload)
        fallback_violation_payload["mode"] = "run"
        fallback_violation_payload["status"] = "fail"
        fallback_violation_payload["reason_code"] = "runtime_signer_fallback_private_key_present_violation"
        fallback_violation_payload["runtime_signer_fallback_private_key_present"] = True
        fallback_violation_payload["bootstrap_reason_code"] = "fallback_signer_secret_present_violation"
        fallback_violation_payload["localhost_signed_reason_code"] = "fallback_signer_secret_present_violation"
        fallback_violation_payload["conformance_reason_code"] = "fallback_signer_secret_present_violation"
        fallback_violation_payload["runtime_commit_reason_code"] = "fallback_signer_secret_present_violation"
        fallback_violation_payload["runtime_commit_policy_reason_code"] = "fallback_signer_secret_present_violation"
        fallback_violation_checks = [
            {
                "id": "bootstrap_readiness",
                "command": "bash scripts/kolme/run_local_kolme_fork_bootstrap_readiness_lane.sh --mode run",
                "status": "skipped",
                "reason_code": "fallback_signer_secret_present_violation",
            },
            {
                "id": "localhost_signed_integration",
                "command": "bash scripts/sdk/run_localhost_signed_integration_contract_lane.sh --output-json /tmp/localhost-signed.json",
                "status": "skipped",
                "reason_code": "fallback_signer_secret_present_violation",
            },
            {
                "id": "live_api_conformance",
                "command": "bash scripts/kolme/run_local_kolme_live_api_conformance_harness.sh --mode run",
                "status": "skipped",
                "reason_code": "fallback_signer_secret_present_violation",
            },
            {
                "id": "runtime_signer_fallback_private_key_contract",
                "command": "fallback signer secret env must remain unset for real-node runtime profile",
                "status": "fail",
                "reason_code": "fallback_signer_secret_present_violation",
            },
            {
                "id": "runtime_commit_endpoint",
                "command": summary_payload.get("runtime_commit_command", ""),
                "status": "skipped",
                "reason_code": "fallback_signer_secret_present_violation",
            },
            {
                "id": "runtime_commit_policy",
                "command": "python3 scripts/kolme/check_local_runtime_commit_live_evidence_policy.py --report-file /tmp/runtime-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/runtime-policy.json",
                "status": "skipped",
                "reason_code": "fallback_signer_secret_present_violation",
            },
        ]
        fallback_violation_payload["checks"] = fallback_violation_checks
        fallback_violation_report = temp_path / "runtime_fallback_violation_summary.json"
        fallback_violation_report.write_text(
            json.dumps(fallback_violation_payload, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )

        fallback_violation_run = subprocess.run(
            [
                "python3",
                str(CHECKER),
                "--report-file",
                str(fallback_violation_report),
                "--expected-final-decision",
                "NO-GO",
                "--ci-fast-gate",
                "PASS",
                "--require-reason-code",
                "runtime_signer_fallback_private_key_present_violation",
                "--output-json",
                str(failure_policy_output),
            ],
            cwd=ROOT_DIR,
            check=False,
            capture_output=True,
            text=True,
        )
        if fallback_violation_run.returncode == 0:
            print("expected checker to fail when fallback signer private key marker is present", file=sys.stderr)
            return 1
        fallback_violation_output = f"{fallback_violation_run.stdout}\n{fallback_violation_run.stderr}"
        if "runtime_signer_fallback_private_key_present_violation" not in fallback_violation_output:
            print(
                "expected fallback signer private key violation reason for policy failure",
                file=sys.stderr,
            )
            return 1

    doc_text = DOC_FILE.read_text(encoding="utf-8")
    readme_text = README_FILE.read_text(encoding="utf-8")
    if "run_local_kamn_live_runtime_integration_lane.sh" not in doc_text:
        print("expected Kolme devnet ops doc to reference local KAMN live runtime integration runner", file=sys.stderr)
        return 1
    if "check_local_kamn_live_runtime_integration_policy.py" not in doc_text:
        print("expected Kolme devnet ops doc to reference local KAMN live runtime integration policy checker", file=sys.stderr)
        return 1
    if "run_local_kamn_live_runtime_integration_contract_lane.sh" not in doc_text:
        print("expected Kolme devnet ops doc to reference local KAMN live runtime integration contract lane", file=sys.stderr)
        return 1
    if "run_local_runtime_commit_live_finality_evidence_contract_lane.sh" not in doc_text:
        print(
            "expected Kolme devnet ops doc to reference runtime finality evidence contract lane composition",
            file=sys.stderr,
        )
        return 1
    # Regression: #1971
    if "--runtime-commit-finality-command" not in doc_text:
        print("expected Kolme devnet ops doc to document runtime finality pass-through command option", file=sys.stderr)
        return 1
    if "--runtime-commit-live-policy-report" not in doc_text:
        print(
            "expected Kolme devnet ops doc to document runtime finality policy report composition option",
            file=sys.stderr,
        )
        return 1
    # Regression: #2112
    if "--runtime-provider-client-contract" not in doc_text:
        print(
            "expected Kolme devnet ops doc to document runtime provider contract option",
            file=sys.stderr,
        )
        return 1
    # Regression: #2113
    if "ci_fast_gate_eligible" not in doc_text:
        print(
            "expected Kolme devnet ops doc to document local-only fast-gate eligibility marker",
            file=sys.stderr,
        )
        return 1
    # Regression: #2114
    if "Live Provider Operator Runbook (Issue #2114)" not in doc_text:
        print(
            "expected Kolme devnet ops doc to include live provider operator runbook section marker",
            file=sys.stderr,
        )
        return 1
    if "run_localhost_signed_integration_contract_lane.sh" not in doc_text:
        print("expected Kolme devnet ops doc to reference localhost signed integration prerequisite lane", file=sys.stderr)
        return 1
    if "Regression: #1489" not in doc_text:
        print("expected Kolme devnet ops doc to include local KAMN live runtime integration regression marker", file=sys.stderr)
        return 1
    if "Regression: #1971" not in doc_text:
        print("expected Kolme devnet ops doc to include runtime finality pass-through regression marker", file=sys.stderr)
        return 1
    # Regression: #2101
    if "Regression: #2101" not in doc_text:
        print("expected Kolme devnet ops doc to include runtime finality contract composition regression marker", file=sys.stderr)
        return 1
    # Regression: #2302
    if "runtime_signer_fallback_private_key_env=KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK" not in doc_text:
        print("expected Kolme devnet ops doc to include fallback signer private key env marker", file=sys.stderr)
        return 1
    if "runtime_signer_fallback_private_key_present=false" not in doc_text:
        print("expected Kolme devnet ops doc to include fallback signer private key presence marker", file=sys.stderr)
        return 1
    if "runtime_signer_fallback_private_key_present_violation" not in doc_text:
        print("expected Kolme devnet ops doc to include fallback signer private key violation marker", file=sys.stderr)
        return 1
    if "Regression: #2302" not in doc_text:
        print("expected Kolme devnet ops doc to include fallback signer runtime regression marker", file=sys.stderr)
        return 1
    if "run_local_kamn_live_runtime_integration_contract_lane.sh" not in readme_text:
        print("expected README to reference local KAMN live runtime integration contract lane", file=sys.stderr)
        return 1
    if "run_local_runtime_commit_live_finality_evidence_contract_lane.sh" not in readme_text:
        print(
            "expected README to reference runtime finality evidence contract lane composition",
            file=sys.stderr,
        )
        return 1
    if "--runtime-commit-finality-command" not in readme_text:
        print("expected README to document runtime finality pass-through command option", file=sys.stderr)
        return 1
    if "--runtime-commit-live-policy-report" not in readme_text:
        print(
            "expected README to document runtime finality policy report composition option",
            file=sys.stderr,
        )
        return 1
    if "--runtime-provider-client-contract" not in readme_text:
        print(
            "expected README to document runtime provider contract option",
            file=sys.stderr,
        )
        return 1
    if "ci_fast_gate_eligible" not in readme_text:
        print(
            "expected README to document local-only fast-gate eligibility marker",
            file=sys.stderr,
        )
        return 1
    if "Live Provider Operator Runbook (Issue #2114)" not in readme_text:
        print(
            "expected README to reference live provider operator runbook section marker",
            file=sys.stderr,
        )
        return 1
    if "runtime_signer_fallback_private_key_env=KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK" not in readme_text:
        print("expected README to include fallback signer private key env marker", file=sys.stderr)
        return 1
    if "runtime_signer_fallback_private_key_present=false" not in readme_text:
        print("expected README to include fallback signer private key presence marker", file=sys.stderr)
        return 1
    if "runtime_signer_fallback_private_key_present_violation" not in readme_text:
        print("expected README to include fallback signer private key violation marker", file=sys.stderr)
        return 1
    if "Regression: #2302" not in readme_text:
        print("expected README to include fallback signer runtime regression marker", file=sys.stderr)
        return 1

    elapsed_seconds = int(time.monotonic() - start_epoch)
    if elapsed_seconds > max_seconds:
        print(
            f"local KAMN live runtime integration contract lane exceeded runtime budget: {elapsed_seconds}s",
            file=sys.stderr,
        )
        return 1

    print("local KAMN live runtime integration contract lane tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
