#!/usr/bin/env python3
"""Contract lane runner for local KAMN live runtime real-node profile checks."""

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
CHECKER = ROOT_DIR / "scripts/kolme/check_local_kamn_live_runtime_real_node_profile_policy.py"
DOC_FILE = ROOT_DIR / "docs/planning/kolme-devnet-ops.md"
CI_DOC_FILE = ROOT_DIR / "docs/ci/strategy.md"
README_FILE = ROOT_DIR / "README.md"
SIGNER_PRIVATE_KEY_ENV_BY_PROFILE = {
    "ops-primary": "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX",
    "ops-secondary": "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY",
}
SIGNER_KEY_REF_ENV_BY_PROFILE = {
    "ops-primary": "KAMN_KOLME_LIVE_SIGNER_KEY_REF",
    "ops-secondary": "KAMN_KOLME_LIVE_SIGNER_KEY_REF_SECONDARY",
}
SIGNER_PUBLIC_KEY_ENV_BY_PROFILE = {
    "ops-primary": "KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX",
    "ops-secondary": "KAMN_KOLME_LIVE_SIGNER_PUBLIC_KEY_HEX_SECONDARY",
}
FALLBACK_SIGNER_GUARD_CONTRACT_VERSION = "v2"
FALLBACK_SIGNER_GUARD_MODE = "reject_if_present"


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Run local KAMN live runtime real-node profile contract lane checks."
    )
    parser.add_argument(
        "--output-json",
        default="/tmp/kolme-local-kamn-live-runtime-integration-summary.json",
        help="Runtime integration summary output.",
    )
    parser.add_argument(
        "--policy-output-json",
        default="/tmp/kolme-local-kamn-live-runtime-real-node-policy.json",
        help="Real-node profile policy checker output.",
    )
    parser.add_argument(
        "--max-seconds",
        default="180",
        help="Total runtime budget in seconds.",
    )
    parser.add_argument(
        "--fork-chain-version",
        default="v0.15.2",
        help="Required fork-info chain_version query value.",
    )
    parser.add_argument(
        "--runtime-signer-profile",
        default="ops-primary",
        choices=sorted(SIGNER_PRIVATE_KEY_ENV_BY_PROFILE.keys()),
        help="Signer profile marker expected from the real-node runtime lane.",
    )
    return parser


def ensure_markers_present(text: str, markers: list[str], source_name: str) -> list[str]:
    missing: list[str] = []
    for marker in markers:
        if marker not in text:
            missing.append(f"{source_name}_missing_marker:{marker}")
    return missing


def run_real_node_policy_check(
    report_file: Path, output_json: Path, expected_final_decision: str
) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [
            "python3",
            str(CHECKER),
            "--report-file",
            str(report_file),
            "--expected-final-decision",
            expected_final_decision,
            "--ci-fast-gate",
            "PASS",
            "--require-non-synthetic-run-evidence",
            "--output-json",
            str(output_json),
        ],
        cwd=ROOT_DIR,
        check=False,
        capture_output=True,
        text=True,
    )


def main() -> int:
    args = build_parser().parse_args()

    if not args.max_seconds.isdigit() or int(args.max_seconds) <= 0:
        print("max-seconds must be a positive integer", file=sys.stderr)
        return 1
    max_seconds = int(args.max_seconds)
    expected_signer_profile = args.runtime_signer_profile
    expected_signer_key_source_contract_version = "v1"
    expected_signer_key_source = "env-local"
    expected_signer_private_key_env = SIGNER_PRIVATE_KEY_ENV_BY_PROFILE[expected_signer_profile]
    expected_signer_key_reference_env = SIGNER_KEY_REF_ENV_BY_PROFILE[expected_signer_profile]
    expected_signer_public_key_env = SIGNER_PUBLIC_KEY_ENV_BY_PROFILE[expected_signer_profile]
    expected_managed_external_raw_private_key_remediation = (
        f"unset {expected_signer_private_key_env}; set {expected_signer_key_reference_env}"
    )
    alternate_signer_profile = "ops-secondary" if expected_signer_profile == "ops-primary" else "ops-primary"
    alternate_signer_private_key_env = SIGNER_PRIVATE_KEY_ENV_BY_PROFILE[alternate_signer_profile]

    if not RUNNER.is_file() or not RUNNER.stat().st_mode & 0o111:
        print("expected local KAMN live runtime integration runner to be executable", file=sys.stderr)
        return 1
    if not CHECKER.is_file() or not CHECKER.stat().st_mode & 0o111:
        print("expected local KAMN live runtime real-node profile policy checker to be executable", file=sys.stderr)
        return 1
    if not DOC_FILE.is_file():
        print("expected Kolme devnet ops documentation to exist", file=sys.stderr)
        return 1
    if not CI_DOC_FILE.is_file():
        print("expected CI strategy documentation to exist", file=sys.stderr)
        return 1
    if not README_FILE.is_file():
        print("expected README to exist", file=sys.stderr)
        return 1

    start_epoch = time.monotonic()

    with tempfile.TemporaryDirectory(prefix="kolme-runtime-real-node-contract-") as temp_dir:
        temp_path = Path(temp_dir)
        checkout_path = temp_path / "kolme_fork"
        runtime_commit_live_summary = temp_path / "runtime_commit_live_summary.json"
        runtime_commit_live_policy = temp_path / "runtime_commit_live_policy.json"
        checkout_path.mkdir(parents=True, exist_ok=True)

        subprocess.run(["git", "-C", str(checkout_path), "init", "-q"], check=True)
        subprocess.run(["git", "-C", str(checkout_path), "checkout", "-q", "-b", "main"], check=True)
        subprocess.run(["git", "-C", str(checkout_path), "config", "user.email", "ci@example.com"], check=True)
        subprocess.run(["git", "-C", str(checkout_path), "config", "user.name", "CI Runner"], check=True)
        (checkout_path / "README.md").write_text(
            "local KAMN real-node profile contract fixture\n",
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
                "init runtime real-node profile fixture",
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
                "--runtime-profile",
                "real-node",
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
                "--runtime-provider-client-contract",
                "KolmeRuntimeCommitLiveProvider",
                "--runtime-signer-profile",
                expected_signer_profile,
                "--runtime-commit-live-summary",
                str(runtime_commit_live_summary),
                "--runtime-commit-live-policy-report",
                str(runtime_commit_live_policy),
                "--output-json",
                args.output_json,
            ],
            cwd=ROOT_DIR,
            check=True,
            stdout=subprocess.DEVNULL,
        )

        policy_result = run_real_node_policy_check(
            report_file=Path(args.output_json),
            output_json=Path(args.policy_output_json),
            expected_final_decision="GO",
        )
        if policy_result.returncode != 0:
            print("expected real-node policy checker GO path to pass", file=sys.stderr)
            stderr = policy_result.stderr.strip()
            if stderr:
                print(stderr, file=sys.stderr)
            return 1

    summary = json.loads(Path(args.output_json).read_text(encoding="utf-8"))
    policy = json.loads(Path(args.policy_output_json).read_text(encoding="utf-8"))
    if summary.get("schema_version") != "kamn.kolme.local-kamn-live-runtime-integration-summary.v1":
        print("unexpected runtime integration summary schema for real-node profile contract lane", file=sys.stderr)
        return 1
    if summary.get("status") != "ok":
        print("expected runtime integration summary status ok for real-node profile contract lane", file=sys.stderr)
        return 1
    if summary.get("reason_code") != "dry_run_no_commands_executed":
        print("expected dry-run reason code for real-node profile contract lane summary", file=sys.stderr)
        return 1
    if summary.get("runtime_profile") != "real-node":
        print("expected runtime_profile=real-node in contract-lane summary", file=sys.stderr)
        return 1
    runtime_commit_command = summary.get("runtime_commit_command")
    if not isinstance(runtime_commit_command, str):
        print("expected runtime_commit_command in contract-lane summary", file=sys.stderr)
        return 1
    if "--require-non-synthetic-run-evidence" not in runtime_commit_command:
        print("expected strict non-synthetic runtime marker in contract-lane summary command", file=sys.stderr)
        return 1
    if "KAMN_KOLME_LIVE_SIGNING_PROFILE=kolme-fork-secp256k1-v1" not in runtime_commit_command:
        print("expected real signing profile marker in contract-lane summary command", file=sys.stderr)
        return 1
    expected_signer_command_marker = f"KAMN_KOLME_LIVE_SIGNER_PROFILE={expected_signer_profile}"
    if expected_signer_command_marker not in runtime_commit_command:
        print("expected signer profile marker in contract-lane summary command", file=sys.stderr)
        return 1
    expected_signer_key_source_command_marker = (
        f"KAMN_KOLME_LIVE_SIGNER_KEY_SOURCE={expected_signer_key_source}"
    )
    if expected_signer_key_source_command_marker not in runtime_commit_command:
        print("expected signer key-source marker in contract-lane summary command", file=sys.stderr)
        return 1
    if "InMemoryKolmeRuntimeCommitClient" in runtime_commit_command:
        print("expected live-provider-only runtime command composition in real-node contract-lane summary", file=sys.stderr)
        return 1
    if summary.get("runtime_commit_command_profile") != "real-node-non-synthetic-v1":
        print("expected deterministic runtime commit command profile marker in contract-lane summary", file=sys.stderr)
        return 1
    if summary.get("runtime_commit_policy_command_profile") != "real-node-non-synthetic-v1":
        print("expected deterministic runtime commit policy command profile marker in contract-lane summary", file=sys.stderr)
        return 1
    if summary.get("runtime_commit_command_profile_version") != "v1":
        print("expected runtime commit command profile marker version in contract-lane summary", file=sys.stderr)
        return 1
    if summary.get("runtime_signer_profile_selector_env") != "KAMN_KOLME_LIVE_SIGNER_PROFILE":
        print("expected signer profile selector env marker in contract-lane summary", file=sys.stderr)
        return 1
    if summary.get("runtime_signer_profile") != expected_signer_profile:
        print("expected signer profile marker in contract-lane summary", file=sys.stderr)
        return 1
    if summary.get("runtime_signer_previous_profile") != expected_signer_profile:
        print("expected signer previous profile marker in contract-lane summary", file=sys.stderr)
        return 1
    if summary.get("runtime_signer_failover_active") is not False:
        print("expected signer failover marker false in contract-lane summary", file=sys.stderr)
        return 1
    if summary.get("runtime_signer_rotation_epoch") != 1:
        print("expected signer rotation epoch marker in contract-lane summary", file=sys.stderr)
        return 1
    if summary.get("runtime_signer_previous_rotation_epoch") != 1:
        print("expected signer previous rotation epoch marker in contract-lane summary", file=sys.stderr)
        return 1
    if summary.get("runtime_signer_key_source_contract_version") != expected_signer_key_source_contract_version:
        print("expected signer key-source contract version marker in contract-lane summary", file=sys.stderr)
        return 1
    if summary.get("runtime_signer_key_source") != expected_signer_key_source:
        print("expected signer key-source marker in contract-lane summary", file=sys.stderr)
        return 1
    if summary.get("runtime_signing_profile") != "kolme-fork-secp256k1-v1":
        print("expected runtime signing profile marker in contract-lane summary", file=sys.stderr)
        return 1
    if summary.get("runtime_signer_private_key_env") != expected_signer_private_key_env:
        print("expected signer private key env marker in contract-lane summary", file=sys.stderr)
        return 1
    if summary.get("runtime_signer_key_reference_env") != expected_signer_key_reference_env:
        print("expected signer key reference env marker in contract-lane summary", file=sys.stderr)
        return 1
    if (
        summary.get("runtime_signer_fallback_guard_contract_version")
        != FALLBACK_SIGNER_GUARD_CONTRACT_VERSION
    ):
        print(
            "expected fallback signer guard contract version marker in contract-lane summary",
            file=sys.stderr,
        )
        return 1
    if summary.get("runtime_signer_fallback_guard_mode") != FALLBACK_SIGNER_GUARD_MODE:
        print("expected fallback signer guard mode marker in contract-lane summary", file=sys.stderr)
        return 1
    if (
        summary.get("runtime_signer_managed_external_raw_private_key_remediation")
        != expected_managed_external_raw_private_key_remediation
    ):
        print(
            "expected managed-external signer raw private key remediation marker in contract-lane summary",
            file=sys.stderr,
        )
        return 1
    if summary.get("runtime_signer_fallback_private_key_present") is not False:
        print("expected fallback signer private key presence marker false in contract-lane summary", file=sys.stderr)
        return 1
    if summary.get("runtime_signer_raw_private_key_present") is not False:
        print("expected runtime signer raw private key presence marker false in contract-lane summary", file=sys.stderr)
        return 1
    if summary.get("runtime_signer_private_key_env_zeroized") is not True:
        print("expected runtime signer private key env zeroization marker true in contract-lane summary", file=sys.stderr)
        return 1
    if summary.get("runtime_signer_private_key_bytes_zeroized") is not True:
        print("expected runtime signer private key bytes zeroization marker true in contract-lane summary", file=sys.stderr)
        return 1
    if summary.get("runtime_signer_attestation_schema_version") != "kamn.kolme.runtime-signer-attestation.v1":
        print("expected runtime signer attestation schema marker in contract-lane summary", file=sys.stderr)
        return 1
    runtime_signer_attestation_bundle = summary.get("runtime_signer_attestation_bundle")
    if not isinstance(runtime_signer_attestation_bundle, dict):
        print("expected runtime signer attestation bundle in contract-lane summary", file=sys.stderr)
        return 1
    if runtime_signer_attestation_bundle.get("schema_version") != "kamn.kolme.runtime-signer-attestation.v1":
        print("expected runtime signer attestation bundle schema marker in contract-lane summary", file=sys.stderr)
        return 1
    if runtime_signer_attestation_bundle.get("required_approvals") != 1:
        print("expected runtime signer attestation required approvals marker in contract-lane summary", file=sys.stderr)
        return 1
    expected_attestation_signers = [expected_signer_profile]
    if runtime_signer_attestation_bundle.get("approved_signers") != expected_attestation_signers:
        print("expected runtime signer attestation approved signers marker in contract-lane summary", file=sys.stderr)
        return 1
    if summary.get("runtime_signer_quorum_linkage_contract_version") != "v1":
        print("expected runtime signer quorum linkage contract version marker in contract-lane summary", file=sys.stderr)
        return 1
    if summary.get("runtime_signer_quorum_required_approvals") != 1:
        print("expected runtime signer quorum required approvals marker in contract-lane summary", file=sys.stderr)
        return 1
    if summary.get("runtime_signer_quorum_approved_signers_count") != 1:
        print("expected runtime signer quorum approved signers count marker in contract-lane summary", file=sys.stderr)
        return 1
    if summary.get("runtime_signer_quorum_profile_linked") is not True:
        print("expected runtime signer quorum profile linked marker true in contract-lane summary", file=sys.stderr)
        return 1
    if summary.get("runtime_signer_quorum_satisfied") is not True:
        print("expected runtime signer quorum satisfied marker true in contract-lane summary", file=sys.stderr)
        return 1
    if summary.get("runtime_signer_quorum_linked") is not True:
        print("expected runtime signer quorum linked marker true in contract-lane summary", file=sys.stderr)
        return 1
    checks = summary.get("checks")
    if not isinstance(checks, list):
        print("expected checks list in real-node profile contract-lane summary", file=sys.stderr)
        return 1
    fallback_signer_checks = [
        check
        for check in checks
        if isinstance(check, dict) and check.get("id") == "runtime_signer_fallback_private_key_contract"
    ]
    if len(fallback_signer_checks) != 1:
        print(
            "expected one runtime_signer_fallback_private_key_contract check in contract-lane summary",
            file=sys.stderr,
        )
        return 1
    if fallback_signer_checks[0].get("status") != "planned":
        print(
            "expected runtime_signer_fallback_private_key_contract planned status in dry-run contract-lane summary",
            file=sys.stderr,
        )
        return 1
    contracts = summary.get("contracts", {})
    if not isinstance(contracts, dict):
        print("expected contracts object in real-node profile contract-lane summary", file=sys.stderr)
        return 1
    if contracts.get("runtime_profile") != "real-node":
        print("expected contracts.runtime_profile=real-node in contract-lane summary", file=sys.stderr)
        return 1
    if contracts.get("runtime_signer_profile_selector_env") != "KAMN_KOLME_LIVE_SIGNER_PROFILE":
        print("expected contracts signer profile selector env marker in contract-lane summary", file=sys.stderr)
        return 1
    if contracts.get("runtime_signer_profile") != expected_signer_profile:
        print("expected contracts signer profile marker in contract-lane summary", file=sys.stderr)
        return 1
    if contracts.get("runtime_signer_failover_requires_profile_change") is not True:
        print("expected contracts failover profile-change guard marker in contract-lane summary", file=sys.stderr)
        return 1
    if contracts.get("runtime_signer_rotation_epoch_must_increase_on_failover") is not True:
        print("expected contracts signer rotation epoch guard marker in contract-lane summary", file=sys.stderr)
        return 1
    if contracts.get("runtime_signer_key_source_contract_version") != expected_signer_key_source_contract_version:
        print("expected contracts signer key-source contract version marker in contract-lane summary", file=sys.stderr)
        return 1
    if contracts.get("runtime_signer_key_source") != expected_signer_key_source:
        print("expected contracts signer key-source marker in contract-lane summary", file=sys.stderr)
        return 1
    if contracts.get("runtime_signing_profile") != "kolme-fork-secp256k1-v1":
        print("expected contracts runtime signing profile marker in contract-lane summary", file=sys.stderr)
        return 1
    if contracts.get("runtime_signer_private_key_env") != expected_signer_private_key_env:
        print("expected contracts signer private key env marker in contract-lane summary", file=sys.stderr)
        return 1
    if contracts.get("runtime_signer_key_reference_env") != expected_signer_key_reference_env:
        print("expected contracts signer key reference env marker in contract-lane summary", file=sys.stderr)
        return 1
    if (
        contracts.get("runtime_signer_fallback_guard_contract_version")
        != FALLBACK_SIGNER_GUARD_CONTRACT_VERSION
    ):
        print(
            "expected contracts fallback signer guard contract version marker in contract-lane summary",
            file=sys.stderr,
        )
        return 1
    if contracts.get("runtime_signer_fallback_guard_mode") != FALLBACK_SIGNER_GUARD_MODE:
        print("expected contracts fallback signer guard mode marker in contract-lane summary", file=sys.stderr)
        return 1
    if (
        contracts.get("runtime_signer_managed_external_raw_private_key_remediation")
        != expected_managed_external_raw_private_key_remediation
    ):
        print(
            "expected contracts managed-external signer raw private key remediation marker in contract-lane summary",
            file=sys.stderr,
        )
        return 1
    if contracts.get("runtime_signer_fallback_private_key_allowed") is not False:
        print("expected contracts fallback signer private key allowed=false marker in contract-lane summary", file=sys.stderr)
        return 1
    if contracts.get("runtime_signer_fallback_private_key_command_marker_allowed") is not False:
        print(
            "expected contracts fallback signer private key command marker allowed=false marker in contract-lane summary",
            file=sys.stderr,
        )
        return 1
    if contracts.get("runtime_signer_managed_external_raw_private_key_allowed") is not False:
        print("expected contracts managed-external raw private key allowed=false marker in contract-lane summary", file=sys.stderr)
        return 1
    if contracts.get("runtime_signer_private_key_env_zeroization_required") is not True:
        print("expected contracts signer private key env zeroization required marker in contract-lane summary", file=sys.stderr)
        return 1
    if contracts.get("runtime_signer_private_key_bytes_zeroization_required") is not True:
        print("expected contracts signer private key bytes zeroization required marker in contract-lane summary", file=sys.stderr)
        return 1
    if contracts.get("runtime_signer_attestation_schema_version") != "kamn.kolme.runtime-signer-attestation.v1":
        print("expected contracts runtime signer attestation schema marker in contract-lane summary", file=sys.stderr)
        return 1
    if contracts.get("runtime_signer_attestation_signer_uniqueness_required") is not True:
        print("expected contracts runtime signer attestation signer uniqueness marker in contract-lane summary", file=sys.stderr)
        return 1
    if contracts.get("runtime_signer_attestation_threshold_required") is not True:
        print("expected contracts runtime signer attestation threshold marker in contract-lane summary", file=sys.stderr)
        return 1
    if contracts.get("runtime_signer_attestation_profile_membership_required") is not True:
        print("expected contracts runtime signer attestation profile membership marker in contract-lane summary", file=sys.stderr)
        return 1
    if contracts.get("runtime_signer_attestation_required_approvals") != 1:
        print("expected contracts runtime signer attestation required approvals marker in contract-lane summary", file=sys.stderr)
        return 1
    if contracts.get("runtime_signer_quorum_linkage_contract_version") != "v1":
        print("expected contracts runtime signer quorum linkage contract version marker in contract-lane summary", file=sys.stderr)
        return 1
    if contracts.get("runtime_signer_quorum_required_approvals") != 1:
        print("expected contracts runtime signer quorum required approvals marker in contract-lane summary", file=sys.stderr)
        return 1
    if contracts.get("runtime_signer_quorum_linked_required") is not True:
        print("expected contracts runtime signer quorum linked-required marker in contract-lane summary", file=sys.stderr)
        return 1
    if contracts.get("runtime_signer_quorum_threshold_required") is not True:
        print("expected contracts runtime signer quorum threshold-required marker in contract-lane summary", file=sys.stderr)
        return 1
    if contracts.get("runtime_signer_quorum_profile_membership_required") is not True:
        print("expected contracts runtime signer quorum profile-membership marker in contract-lane summary", file=sys.stderr)
        return 1
    if contracts.get("runtime_signer_quorum_linked") is not True:
        print("expected contracts runtime signer quorum linked marker in contract-lane summary", file=sys.stderr)
        return 1
    if contracts.get("runtime_signer_failover_attestation_min_required_approvals") != 2:
        print(
            "expected contracts runtime signer failover attestation minimum approvals marker in contract-lane summary",
            file=sys.stderr,
        )
        return 1
    if contracts.get("runtime_signer_failover_attestation_previous_profile_membership_required") is not True:
        print(
            "expected contracts runtime signer failover previous-profile attestation membership marker in contract-lane summary",
            file=sys.stderr,
        )
        return 1
    if policy.get("schema_version") != "kamn.kolme.local-kamn-live-runtime-real-node-policy-report.v1":
        print("unexpected real-node profile policy schema in contract-lane output", file=sys.stderr)
        return 1
    if policy.get("final_decision") != "GO":
        print("expected real-node profile policy final_decision GO", file=sys.stderr)
        return 1
    if policy.get("observed_reason_code") != "dry_run_no_commands_executed":
        print("expected dry-run observed reason code in real-node profile policy output", file=sys.stderr)
        return 1

    with tempfile.TemporaryDirectory(prefix="kolme-runtime-real-node-negative-") as temp_dir:
        negative_path = Path(temp_dir)

        marker_drift_summary_file = negative_path / "marker_drift_summary.json"
        marker_drift_policy_file = negative_path / "marker_drift_policy.json"
        marker_drift_summary = dict(summary)
        marker_drift_summary["runtime_commit_command_profile"] = "standard-default-v1"
        marker_drift_summary_file.write_text(
            json.dumps(marker_drift_summary, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )

        marker_drift_result = run_real_node_policy_check(
            report_file=marker_drift_summary_file,
            output_json=marker_drift_policy_file,
            expected_final_decision="NO-GO",
        )
        if marker_drift_result.returncode == 0:
            print("expected marker drift negative proof to fail closed", file=sys.stderr)
            return 1
        marker_drift_policy = json.loads(marker_drift_policy_file.read_text(encoding="utf-8"))
        marker_drift_reason_codes = marker_drift_policy.get("reason_codes")
        if not isinstance(marker_drift_reason_codes, list):
            print("expected reason_codes list in marker drift policy output", file=sys.stderr)
            return 1
        if "runtime_commit_command_profile_mismatch" not in marker_drift_reason_codes:
            print("expected runtime_commit_command_profile_mismatch in marker drift policy output", file=sys.stderr)
            return 1
        if marker_drift_policy.get("final_decision") != "NO-GO":
            print("expected NO-GO final decision for marker drift policy output", file=sys.stderr)
            return 1

        forced_failover_go_summary_file = negative_path / "forced_failover_go_summary.json"
        forced_failover_go_policy_file = negative_path / "forced_failover_go_policy.json"
        forced_failover_go_summary = dict(summary)
        forced_failover_go_summary["runtime_signer_profile"] = alternate_signer_profile
        forced_failover_go_summary["runtime_signer_previous_profile"] = expected_signer_profile
        forced_failover_go_summary["runtime_signer_failover_active"] = True
        forced_failover_go_summary["runtime_signer_rotation_epoch"] = 2
        forced_failover_go_summary["runtime_signer_previous_rotation_epoch"] = 1
        forced_failover_go_summary["runtime_signer_private_key_env"] = SIGNER_PRIVATE_KEY_ENV_BY_PROFILE[
            alternate_signer_profile
        ]
        forced_failover_go_summary["runtime_signer_key_reference_env"] = SIGNER_KEY_REF_ENV_BY_PROFILE[
            alternate_signer_profile
        ]
        forced_failover_go_summary["runtime_signer_managed_external_raw_private_key_remediation"] = (
            "unset "
            f"{SIGNER_PRIVATE_KEY_ENV_BY_PROFILE[alternate_signer_profile]}; "
            "set "
            f"{SIGNER_KEY_REF_ENV_BY_PROFILE[alternate_signer_profile]}"
        )
        forced_failover_go_summary["runtime_commit_command"] = runtime_commit_command.replace(
            expected_signer_command_marker,
            f"KAMN_KOLME_LIVE_SIGNER_PROFILE={alternate_signer_profile}",
            1,
        )
        forced_failover_go_attestation_bundle = dict(runtime_signer_attestation_bundle)
        forced_failover_go_attestation_bundle["approved_signers"] = [
            expected_signer_profile,
            alternate_signer_profile,
        ]
        forced_failover_go_attestation_bundle["required_approvals"] = 2
        forced_failover_go_attestation_bundle["signer_profile"] = alternate_signer_profile
        forced_failover_go_summary["runtime_signer_attestation_bundle"] = (
            forced_failover_go_attestation_bundle
        )
        forced_failover_go_summary["runtime_signer_quorum_required_approvals"] = 2
        forced_failover_go_summary["runtime_signer_quorum_approved_signers_count"] = 2
        forced_failover_go_summary["runtime_signer_quorum_profile_linked"] = True
        forced_failover_go_summary["runtime_signer_quorum_satisfied"] = True
        forced_failover_go_summary["runtime_signer_quorum_linked"] = True
        forced_failover_go_contracts = dict(summary.get("contracts", {}))
        forced_failover_go_contracts["runtime_signer_profile"] = alternate_signer_profile
        forced_failover_go_contracts["runtime_signer_private_key_env"] = (
            SIGNER_PRIVATE_KEY_ENV_BY_PROFILE[alternate_signer_profile]
        )
        forced_failover_go_contracts["runtime_signer_key_reference_env"] = (
            SIGNER_KEY_REF_ENV_BY_PROFILE[alternate_signer_profile]
        )
        forced_failover_go_contracts["runtime_signer_managed_external_raw_private_key_remediation"] = (
            "unset "
            f"{SIGNER_PRIVATE_KEY_ENV_BY_PROFILE[alternate_signer_profile]}; "
            "set "
            f"{SIGNER_KEY_REF_ENV_BY_PROFILE[alternate_signer_profile]}"
        )
        forced_failover_go_contracts["runtime_signer_attestation_required_approvals"] = 2
        forced_failover_go_contracts["runtime_signer_quorum_required_approvals"] = 2
        forced_failover_go_contracts["runtime_signer_quorum_linked"] = True
        forced_failover_go_summary["contracts"] = forced_failover_go_contracts
        forced_failover_go_summary_file.write_text(
            json.dumps(forced_failover_go_summary, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )

        forced_failover_go_result = run_real_node_policy_check(
            report_file=forced_failover_go_summary_file,
            output_json=forced_failover_go_policy_file,
            expected_final_decision="GO",
        )
        if forced_failover_go_result.returncode != 0:
            print("expected forced failover GO scenario matrix proof to pass", file=sys.stderr)
            stderr = forced_failover_go_result.stderr.strip()
            if stderr:
                print(stderr, file=sys.stderr)
            return 1
        forced_failover_go_policy = json.loads(
            forced_failover_go_policy_file.read_text(encoding="utf-8")
        )
        if forced_failover_go_policy.get("final_decision") != "GO":
            print("expected GO final decision for forced failover scenario matrix policy output", file=sys.stderr)
            return 1
        forced_failover_go_reason_codes = forced_failover_go_policy.get("reason_codes")
        if forced_failover_go_reason_codes != []:
            print("expected no reason codes in forced failover scenario matrix policy output", file=sys.stderr)
            return 1

        split_brain_summary_file = negative_path / "split_brain_negative_summary.json"
        split_brain_policy_file = negative_path / "split_brain_negative_policy.json"
        split_brain_summary = dict(forced_failover_go_summary)
        forced_failover_command = split_brain_summary.get("runtime_commit_command")
        if not isinstance(forced_failover_command, str):
            print(
                "expected runtime_commit_command marker in split-brain negative summary fixture",
                file=sys.stderr,
            )
            return 1
        split_brain_summary["runtime_commit_command"] = forced_failover_command.replace(
            f"KAMN_KOLME_LIVE_SIGNER_PROFILE={alternate_signer_profile}",
            (
                f"KAMN_KOLME_LIVE_SIGNER_PROFILE={alternate_signer_profile} "
                f"KAMN_KOLME_LIVE_SIGNER_PROFILE={expected_signer_profile}"
            ),
            1,
        )
        split_brain_summary_file.write_text(
            json.dumps(split_brain_summary, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )

        split_brain_result = run_real_node_policy_check(
            report_file=split_brain_summary_file,
            output_json=split_brain_policy_file,
            expected_final_decision="NO-GO",
        )
        if split_brain_result.returncode == 0:
            print("expected split-brain signer-profile negative proof to fail closed", file=sys.stderr)
            return 1
        split_brain_policy = json.loads(split_brain_policy_file.read_text(encoding="utf-8"))
        split_brain_reason_codes = split_brain_policy.get("reason_codes")
        if not isinstance(split_brain_reason_codes, list):
            print("expected reason_codes list in split-brain policy output", file=sys.stderr)
            return 1
        if "runtime_commit_signer_profile_split_brain_detected" not in split_brain_reason_codes:
            print(
                "expected runtime_commit_signer_profile_split_brain_detected in split-brain policy output",
                file=sys.stderr,
            )
            return 1
        if split_brain_policy.get("final_decision") != "NO-GO":
            print("expected NO-GO final decision for split-brain policy output", file=sys.stderr)
            return 1

        failover_stale_summary_file = negative_path / "failover_stale_summary.json"
        failover_stale_policy_file = negative_path / "failover_stale_policy.json"
        failover_stale_summary = dict(summary)
        failover_stale_summary["runtime_signer_profile"] = expected_signer_profile
        failover_stale_summary["runtime_signer_previous_profile"] = expected_signer_profile
        failover_stale_summary["runtime_signer_failover_active"] = True
        failover_stale_summary["runtime_signer_rotation_epoch"] = 3
        failover_stale_summary["runtime_signer_previous_rotation_epoch"] = 3
        failover_stale_summary_file.write_text(
            json.dumps(failover_stale_summary, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )

        failover_stale_result = run_real_node_policy_check(
            report_file=failover_stale_summary_file,
            output_json=failover_stale_policy_file,
            expected_final_decision="NO-GO",
        )
        if failover_stale_result.returncode == 0:
            print("expected failover stale negative proof to fail closed", file=sys.stderr)
            return 1
        failover_stale_policy = json.loads(failover_stale_policy_file.read_text(encoding="utf-8"))
        failover_stale_reason_codes = failover_stale_policy.get("reason_codes")
        if not isinstance(failover_stale_reason_codes, list):
            print("expected reason_codes list in failover stale policy output", file=sys.stderr)
            return 1
        if "runtime_signer_failover_profile_unchanged" not in failover_stale_reason_codes:
            print("expected runtime_signer_failover_profile_unchanged in failover stale policy output", file=sys.stderr)
            return 1
        if "runtime_signer_rotation_epoch_stale" not in failover_stale_reason_codes:
            print("expected runtime_signer_rotation_epoch_stale in failover stale policy output", file=sys.stderr)
            return 1
        if failover_stale_policy.get("final_decision") != "NO-GO":
            print("expected NO-GO final decision for failover stale policy output", file=sys.stderr)
            return 1

        signer_key_env_drift_summary_file = negative_path / "signer_key_env_drift_summary.json"
        signer_key_env_drift_policy_file = negative_path / "signer_key_env_drift_policy.json"
        signer_key_env_drift_summary = dict(summary)
        signer_key_env_drift_summary["runtime_signer_profile"] = expected_signer_profile
        signer_key_env_drift_summary["runtime_signer_previous_profile"] = expected_signer_profile
        signer_key_env_drift_summary["runtime_signer_private_key_env"] = alternate_signer_private_key_env
        signer_key_env_drift_contracts = dict(summary.get("contracts", {}))
        signer_key_env_drift_contracts["runtime_signer_profile"] = expected_signer_profile
        signer_key_env_drift_contracts["runtime_signer_private_key_env"] = alternate_signer_private_key_env
        signer_key_env_drift_summary["contracts"] = signer_key_env_drift_contracts
        signer_key_env_drift_summary_file.write_text(
            json.dumps(signer_key_env_drift_summary, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )

        signer_key_env_drift_result = run_real_node_policy_check(
            report_file=signer_key_env_drift_summary_file,
            output_json=signer_key_env_drift_policy_file,
            expected_final_decision="NO-GO",
        )
        if signer_key_env_drift_result.returncode == 0:
            print("expected signer key env drift negative proof to fail closed", file=sys.stderr)
            return 1
        signer_key_env_drift_policy = json.loads(
            signer_key_env_drift_policy_file.read_text(encoding="utf-8")
        )
        signer_key_env_drift_reason_codes = signer_key_env_drift_policy.get("reason_codes")
        if not isinstance(signer_key_env_drift_reason_codes, list):
            print("expected reason_codes list in signer key env drift policy output", file=sys.stderr)
            return 1
        if "runtime_signer_private_key_env_mismatch" not in signer_key_env_drift_reason_codes:
            print(
                "expected runtime_signer_private_key_env_mismatch in signer key env drift policy output",
                file=sys.stderr,
            )
            return 1
        if signer_key_env_drift_policy.get("final_decision") != "NO-GO":
            print("expected NO-GO final decision for signer key env drift policy output", file=sys.stderr)
            return 1

        signing_profile_drift_summary_file = negative_path / "signing_profile_drift_summary.json"
        signing_profile_drift_policy_file = negative_path / "signing_profile_drift_policy.json"
        signing_profile_drift_summary = dict(summary)
        signing_profile_drift_summary["runtime_signing_profile"] = "simulated-signing-v0"
        signing_profile_drift_summary_file.write_text(
            json.dumps(signing_profile_drift_summary, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )

        signing_profile_drift_result = run_real_node_policy_check(
            report_file=signing_profile_drift_summary_file,
            output_json=signing_profile_drift_policy_file,
            expected_final_decision="NO-GO",
        )
        if signing_profile_drift_result.returncode == 0:
            print("expected runtime signing profile drift negative proof to fail closed", file=sys.stderr)
            return 1
        signing_profile_drift_policy = json.loads(
            signing_profile_drift_policy_file.read_text(encoding="utf-8")
        )
        signing_profile_drift_reason_codes = signing_profile_drift_policy.get("reason_codes")
        if not isinstance(signing_profile_drift_reason_codes, list):
            print("expected reason_codes list in runtime signing profile drift policy output", file=sys.stderr)
            return 1
        if "runtime_signing_profile_mismatch" not in signing_profile_drift_reason_codes:
            print(
                "expected runtime_signing_profile_mismatch in runtime signing profile drift policy output",
                file=sys.stderr,
            )
            return 1
        if signing_profile_drift_policy.get("final_decision") != "NO-GO":
            print("expected NO-GO final decision for runtime signing profile drift policy output", file=sys.stderr)
            return 1

        signing_profile_contract_drift_summary_file = negative_path / "signing_profile_contract_drift_summary.json"
        signing_profile_contract_drift_policy_file = negative_path / "signing_profile_contract_drift_policy.json"
        signing_profile_contract_drift_summary = dict(summary)
        signing_profile_contract_drift_contracts = dict(summary.get("contracts", {}))
        signing_profile_contract_drift_contracts["runtime_signing_profile"] = "simulated-signing-v0"
        signing_profile_contract_drift_summary["contracts"] = signing_profile_contract_drift_contracts
        signing_profile_contract_drift_summary_file.write_text(
            json.dumps(signing_profile_contract_drift_summary, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )

        signing_profile_contract_drift_result = run_real_node_policy_check(
            report_file=signing_profile_contract_drift_summary_file,
            output_json=signing_profile_contract_drift_policy_file,
            expected_final_decision="NO-GO",
        )
        if signing_profile_contract_drift_result.returncode == 0:
            print(
                "expected runtime signing profile contract drift negative proof to fail closed",
                file=sys.stderr,
            )
            return 1
        signing_profile_contract_drift_policy = json.loads(
            signing_profile_contract_drift_policy_file.read_text(encoding="utf-8")
        )
        signing_profile_contract_drift_reason_codes = signing_profile_contract_drift_policy.get(
            "reason_codes"
        )
        if not isinstance(signing_profile_contract_drift_reason_codes, list):
            print(
                "expected reason_codes list in runtime signing profile contract drift policy output",
                file=sys.stderr,
            )
            return 1
        if (
            "runtime_signing_profile_contract_mismatch"
            not in signing_profile_contract_drift_reason_codes
        ):
            print(
                "expected runtime_signing_profile_contract_mismatch in runtime signing profile contract drift policy output",
                file=sys.stderr,
            )
            return 1
        if signing_profile_contract_drift_policy.get("final_decision") != "NO-GO":
            print(
                "expected NO-GO final decision for runtime signing profile contract drift policy output",
                file=sys.stderr,
            )
            return 1

        # Regression: #2302
        fallback_signer_violation_summary_file = negative_path / "fallback_signer_violation_summary.json"
        fallback_signer_violation_policy_file = negative_path / "fallback_signer_violation_policy.json"
        fallback_signer_violation_summary = dict(summary)
        fallback_signer_violation_summary["mode"] = "run"
        fallback_signer_violation_summary["status"] = "fail"
        fallback_signer_violation_summary["reason_code"] = "runtime_signer_fallback_private_key_present_violation"
        fallback_signer_violation_summary["runtime_signer_fallback_private_key_present"] = True
        fallback_signer_violation_summary["bootstrap_reason_code"] = "fallback_signer_secret_present_violation"
        fallback_signer_violation_summary["localhost_signed_reason_code"] = "fallback_signer_secret_present_violation"
        fallback_signer_violation_summary["conformance_reason_code"] = "fallback_signer_secret_present_violation"
        fallback_signer_violation_summary["runtime_commit_reason_code"] = "fallback_signer_secret_present_violation"
        fallback_signer_violation_summary["runtime_commit_policy_reason_code"] = "fallback_signer_secret_present_violation"
        fallback_signer_violation_summary["checks"] = [
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
                "command": runtime_commit_command,
                "status": "skipped",
                "reason_code": "fallback_signer_secret_present_violation",
            },
            {
                "id": "runtime_commit_policy",
                "command": "python3 scripts/kolme/check_local_runtime_commit_live_evidence_policy.py --report-file /tmp/runtime-summary.json --expected-final-decision GO --ci-fast-gate PASS --require-non-synthetic-run-evidence --require-native-payload-evidence --output-json /tmp/runtime-policy.json",
                "status": "skipped",
                "reason_code": "fallback_signer_secret_present_violation",
            },
        ]
        fallback_signer_violation_summary_file.write_text(
            json.dumps(fallback_signer_violation_summary, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )

        fallback_signer_violation_result = run_real_node_policy_check(
            report_file=fallback_signer_violation_summary_file,
            output_json=fallback_signer_violation_policy_file,
            expected_final_decision="NO-GO",
        )
        if fallback_signer_violation_result.returncode == 0:
            print("expected fallback signer violation negative proof to fail closed", file=sys.stderr)
            return 1
        fallback_signer_violation_policy = json.loads(
            fallback_signer_violation_policy_file.read_text(encoding="utf-8")
        )
        fallback_signer_violation_reason_codes = fallback_signer_violation_policy.get("reason_codes")
        if not isinstance(fallback_signer_violation_reason_codes, list):
            print("expected reason_codes list in fallback signer violation policy output", file=sys.stderr)
            return 1
        if "runtime_signer_fallback_private_key_present_violation" not in fallback_signer_violation_reason_codes:
            print(
                "expected runtime_signer_fallback_private_key_present_violation in fallback signer policy output",
                file=sys.stderr,
            )
            return 1
        if fallback_signer_violation_policy.get("final_decision") != "NO-GO":
            print("expected NO-GO final decision for fallback signer violation policy output", file=sys.stderr)
            return 1

        # Regression: #2324
        managed_external_raw_key_violation_summary_file = (
            negative_path / "managed_external_raw_key_violation_summary.json"
        )
        managed_external_raw_key_violation_policy_file = (
            negative_path / "managed_external_raw_key_violation_policy.json"
        )
        managed_external_raw_key_violation_summary = dict(summary)
        managed_external_raw_key_violation_summary["mode"] = "run"
        managed_external_raw_key_violation_summary["status"] = "fail"
        managed_external_raw_key_violation_summary["reason_code"] = (
            "runtime_signer_managed_external_raw_private_key_present_violation"
        )
        managed_external_raw_key_violation_summary["runtime_signer_key_source"] = "managed-external"
        managed_external_raw_key_violation_summary["runtime_signer_raw_private_key_present"] = True
        managed_external_raw_key_violation_summary["bootstrap_reason_code"] = (
            "managed_signer_raw_private_key_present_violation"
        )
        managed_external_raw_key_violation_summary["localhost_signed_reason_code"] = (
            "managed_signer_raw_private_key_present_violation"
        )
        managed_external_raw_key_violation_summary["conformance_reason_code"] = (
            "managed_signer_raw_private_key_present_violation"
        )
        managed_external_raw_key_violation_summary["runtime_commit_reason_code"] = (
            "managed_signer_raw_private_key_present_violation"
        )
        managed_external_raw_key_violation_summary["runtime_commit_policy_reason_code"] = (
            "managed_signer_raw_private_key_present_violation"
        )
        managed_external_raw_key_violation_summary["checks"] = [
            {
                "id": "bootstrap_readiness",
                "command": "bash scripts/kolme/run_local_kolme_fork_bootstrap_readiness_lane.sh --mode run",
                "status": "skipped",
                "reason_code": "managed_signer_raw_private_key_present_violation",
            },
            {
                "id": "localhost_signed_integration",
                "command": "bash scripts/sdk/run_localhost_signed_integration_contract_lane.sh --output-json /tmp/localhost-signed.json",
                "status": "skipped",
                "reason_code": "managed_signer_raw_private_key_present_violation",
            },
            {
                "id": "live_api_conformance",
                "command": "bash scripts/kolme/run_local_kolme_live_api_conformance_harness.sh --mode run",
                "status": "skipped",
                "reason_code": "managed_signer_raw_private_key_present_violation",
            },
            {
                "id": "runtime_signer_fallback_private_key_contract",
                "command": "fallback signer secret env must remain unset for real-node runtime profile",
                "status": "pass",
                "reason_code": "fallback_signer_secret_absent",
            },
            {
                "id": "runtime_signer_managed_external_raw_private_key_contract",
                "command": "managed-external signer profile must reject raw private key env markers for selected profile",
                "status": "fail",
                "reason_code": "managed_signer_raw_private_key_present_violation",
            },
            {
                "id": "runtime_commit_endpoint",
                "command": runtime_commit_command,
                "status": "skipped",
                "reason_code": "managed_signer_raw_private_key_present_violation",
            },
            {
                "id": "runtime_commit_policy",
                "command": "python3 scripts/kolme/check_local_runtime_commit_live_evidence_policy.py --report-file /tmp/runtime-summary.json --expected-final-decision GO --ci-fast-gate PASS --require-non-synthetic-run-evidence --require-native-payload-evidence --output-json /tmp/runtime-policy.json",
                "status": "skipped",
                "reason_code": "managed_signer_raw_private_key_present_violation",
            },
        ]
        managed_external_raw_key_violation_contracts = dict(
            managed_external_raw_key_violation_summary.get("contracts", {})
        )
        managed_external_raw_key_violation_contracts["runtime_signer_key_source"] = "managed-external"
        managed_external_raw_key_violation_summary["contracts"] = (
            managed_external_raw_key_violation_contracts
        )
        managed_external_raw_key_violation_summary_file.write_text(
            json.dumps(managed_external_raw_key_violation_summary, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )

        managed_external_raw_key_violation_result = run_real_node_policy_check(
            report_file=managed_external_raw_key_violation_summary_file,
            output_json=managed_external_raw_key_violation_policy_file,
            expected_final_decision="NO-GO",
        )
        if managed_external_raw_key_violation_result.returncode == 0:
            print("expected managed-external raw signer key violation proof to fail closed", file=sys.stderr)
            return 1
        managed_external_raw_key_violation_policy = json.loads(
            managed_external_raw_key_violation_policy_file.read_text(encoding="utf-8")
        )
        managed_external_raw_key_violation_reason_codes = (
            managed_external_raw_key_violation_policy.get("reason_codes")
        )
        if not isinstance(managed_external_raw_key_violation_reason_codes, list):
            print(
                "expected reason_codes list in managed-external raw signer key violation policy output",
                file=sys.stderr,
            )
            return 1
        if (
            "runtime_signer_managed_external_raw_private_key_present_violation"
            not in managed_external_raw_key_violation_reason_codes
        ):
            print(
                "expected runtime_signer_managed_external_raw_private_key_present_violation in policy output",
                file=sys.stderr,
            )
            return 1
        if managed_external_raw_key_violation_policy.get("final_decision") != "NO-GO":
            print(
                "expected NO-GO final decision for managed-external raw signer key violation policy output",
                file=sys.stderr,
            )
            return 1

        zeroization_env_violation_summary_file = negative_path / "zeroization_env_violation_summary.json"
        zeroization_env_violation_policy_file = negative_path / "zeroization_env_violation_policy.json"
        zeroization_env_violation_summary = dict(summary)
        zeroization_env_violation_summary["mode"] = "run"
        zeroization_env_violation_summary["status"] = "fail"
        zeroization_env_violation_summary["reason_code"] = "runtime_signer_private_key_env_zeroization_violation"
        zeroization_env_violation_summary["runtime_signer_private_key_env_zeroized"] = False
        zeroization_env_violation_summary_file.write_text(
            json.dumps(zeroization_env_violation_summary, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )
        zeroization_env_violation_result = run_real_node_policy_check(
            report_file=zeroization_env_violation_summary_file,
            output_json=zeroization_env_violation_policy_file,
            expected_final_decision="NO-GO",
        )
        if zeroization_env_violation_result.returncode == 0:
            print("expected signer private key env zeroization violation proof to fail closed", file=sys.stderr)
            return 1
        zeroization_env_violation_policy = json.loads(
            zeroization_env_violation_policy_file.read_text(encoding="utf-8")
        )
        zeroization_env_violation_reason_codes = zeroization_env_violation_policy.get("reason_codes")
        if not isinstance(zeroization_env_violation_reason_codes, list):
            print(
                "expected reason_codes list in signer private key env zeroization violation policy output",
                file=sys.stderr,
            )
            return 1
        if "runtime_signer_private_key_env_zeroization_violation" not in zeroization_env_violation_reason_codes:
            print(
                "expected runtime_signer_private_key_env_zeroization_violation in signer private key env zeroization policy output",
                file=sys.stderr,
            )
            return 1
        if zeroization_env_violation_policy.get("final_decision") != "NO-GO":
            print(
                "expected NO-GO final decision for signer private key env zeroization violation policy output",
                file=sys.stderr,
            )
            return 1

        zeroization_bytes_violation_summary_file = negative_path / "zeroization_bytes_violation_summary.json"
        zeroization_bytes_violation_policy_file = negative_path / "zeroization_bytes_violation_policy.json"
        zeroization_bytes_violation_summary = dict(summary)
        zeroization_bytes_violation_summary["mode"] = "run"
        zeroization_bytes_violation_summary["status"] = "fail"
        zeroization_bytes_violation_summary["reason_code"] = "runtime_signer_private_key_bytes_zeroization_violation"
        zeroization_bytes_violation_summary["runtime_signer_private_key_bytes_zeroized"] = False
        zeroization_bytes_violation_summary_file.write_text(
            json.dumps(zeroization_bytes_violation_summary, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )
        zeroization_bytes_violation_result = run_real_node_policy_check(
            report_file=zeroization_bytes_violation_summary_file,
            output_json=zeroization_bytes_violation_policy_file,
            expected_final_decision="NO-GO",
        )
        if zeroization_bytes_violation_result.returncode == 0:
            print("expected signer private key bytes zeroization violation proof to fail closed", file=sys.stderr)
            return 1
        zeroization_bytes_violation_policy = json.loads(
            zeroization_bytes_violation_policy_file.read_text(encoding="utf-8")
        )
        zeroization_bytes_violation_reason_codes = zeroization_bytes_violation_policy.get("reason_codes")
        if not isinstance(zeroization_bytes_violation_reason_codes, list):
            print(
                "expected reason_codes list in signer private key bytes zeroization violation policy output",
                file=sys.stderr,
            )
            return 1
        if "runtime_signer_private_key_bytes_zeroization_violation" not in zeroization_bytes_violation_reason_codes:
            print(
                "expected runtime_signer_private_key_bytes_zeroization_violation in signer private key bytes zeroization policy output",
                file=sys.stderr,
            )
            return 1
        if zeroization_bytes_violation_policy.get("final_decision") != "NO-GO":
            print(
                "expected NO-GO final decision for signer private key bytes zeroization violation policy output",
                file=sys.stderr,
            )
            return 1

        attestation_duplicate_signers_summary_file = negative_path / "attestation_duplicate_signers_summary.json"
        attestation_duplicate_signers_policy_file = negative_path / "attestation_duplicate_signers_policy.json"
        attestation_duplicate_signers_summary = dict(summary)
        attestation_duplicate_signers_bundle = dict(
            attestation_duplicate_signers_summary.get("runtime_signer_attestation_bundle", {})
        )
        attestation_duplicate_signers_bundle["approved_signers"] = [
            expected_signer_profile,
            expected_signer_profile,
        ]
        attestation_duplicate_signers_summary["runtime_signer_attestation_bundle"] = (
            attestation_duplicate_signers_bundle
        )
        attestation_duplicate_signers_summary_file.write_text(
            json.dumps(attestation_duplicate_signers_summary, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )

        attestation_duplicate_signers_result = run_real_node_policy_check(
            report_file=attestation_duplicate_signers_summary_file,
            output_json=attestation_duplicate_signers_policy_file,
            expected_final_decision="NO-GO",
        )
        if attestation_duplicate_signers_result.returncode == 0:
            print("expected duplicate signer attestation proof to fail closed", file=sys.stderr)
            return 1
        attestation_duplicate_signers_policy = json.loads(
            attestation_duplicate_signers_policy_file.read_text(encoding="utf-8")
        )
        attestation_duplicate_signers_reason_codes = attestation_duplicate_signers_policy.get(
            "reason_codes"
        )
        if not isinstance(attestation_duplicate_signers_reason_codes, list):
            print(
                "expected reason_codes list in duplicate signer attestation policy output",
                file=sys.stderr,
            )
            return 1
        if "runtime_signer_attestation_approved_signers_not_unique" not in attestation_duplicate_signers_reason_codes:
            print(
                "expected runtime_signer_attestation_approved_signers_not_unique in duplicate signer attestation policy output",
                file=sys.stderr,
            )
            return 1
        if attestation_duplicate_signers_policy.get("final_decision") != "NO-GO":
            print(
                "expected NO-GO final decision for duplicate signer attestation policy output",
                file=sys.stderr,
            )
            return 1

        attestation_quorum_shortfall_summary_file = negative_path / "attestation_quorum_shortfall_summary.json"
        attestation_quorum_shortfall_policy_file = negative_path / "attestation_quorum_shortfall_policy.json"
        attestation_quorum_shortfall_summary = dict(summary)
        attestation_quorum_shortfall_bundle = dict(
            attestation_quorum_shortfall_summary.get("runtime_signer_attestation_bundle", {})
        )
        attestation_quorum_shortfall_bundle["required_approvals"] = 2
        attestation_quorum_shortfall_bundle["approved_signers"] = [expected_signer_profile]
        attestation_quorum_shortfall_summary["runtime_signer_attestation_bundle"] = (
            attestation_quorum_shortfall_bundle
        )
        attestation_quorum_shortfall_summary["runtime_signer_quorum_required_approvals"] = 2
        attestation_quorum_shortfall_summary["runtime_signer_quorum_approved_signers_count"] = 1
        attestation_quorum_shortfall_summary["runtime_signer_quorum_profile_linked"] = True
        attestation_quorum_shortfall_summary["runtime_signer_quorum_satisfied"] = False
        attestation_quorum_shortfall_summary["runtime_signer_quorum_linked"] = False
        attestation_quorum_shortfall_contracts = dict(
            attestation_quorum_shortfall_summary.get("contracts", {})
        )
        attestation_quorum_shortfall_contracts["runtime_signer_quorum_required_approvals"] = 2
        attestation_quorum_shortfall_contracts["runtime_signer_quorum_linked"] = False
        attestation_quorum_shortfall_summary["contracts"] = (
            attestation_quorum_shortfall_contracts
        )
        attestation_quorum_shortfall_summary_file.write_text(
            json.dumps(attestation_quorum_shortfall_summary, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )

        attestation_quorum_shortfall_result = run_real_node_policy_check(
            report_file=attestation_quorum_shortfall_summary_file,
            output_json=attestation_quorum_shortfall_policy_file,
            expected_final_decision="NO-GO",
        )
        if attestation_quorum_shortfall_result.returncode == 0:
            print("expected attestation quorum shortfall proof to fail closed", file=sys.stderr)
            return 1
        attestation_quorum_shortfall_policy = json.loads(
            attestation_quorum_shortfall_policy_file.read_text(encoding="utf-8")
        )
        attestation_quorum_shortfall_reason_codes = attestation_quorum_shortfall_policy.get(
            "reason_codes"
        )
        if not isinstance(attestation_quorum_shortfall_reason_codes, list):
            print(
                "expected reason_codes list in attestation quorum shortfall policy output",
                file=sys.stderr,
            )
            return 1
        if "runtime_signer_attestation_quorum_shortfall" not in attestation_quorum_shortfall_reason_codes:
            print(
                "expected runtime_signer_attestation_quorum_shortfall in attestation quorum shortfall policy output",
                file=sys.stderr,
            )
            return 1
        if "runtime_signer_quorum_linkage_violation" not in attestation_quorum_shortfall_reason_codes:
            print(
                "expected runtime_signer_quorum_linkage_violation in attestation quorum shortfall policy output",
                file=sys.stderr,
            )
            return 1
        if attestation_quorum_shortfall_policy.get("final_decision") != "NO-GO":
            print(
                "expected NO-GO final decision for attestation quorum shortfall policy output",
                file=sys.stderr,
            )
            return 1

        attestation_schema_invalid_summary_file = negative_path / "attestation_schema_invalid_summary.json"
        attestation_schema_invalid_policy_file = negative_path / "attestation_schema_invalid_policy.json"
        attestation_schema_invalid_summary = dict(summary)
        attestation_schema_invalid_bundle = dict(
            attestation_schema_invalid_summary.get("runtime_signer_attestation_bundle", {})
        )
        attestation_schema_invalid_bundle["schema_version"] = "kamn.kolme.runtime-signer-attestation.v0"
        attestation_schema_invalid_summary["runtime_signer_attestation_bundle"] = (
            attestation_schema_invalid_bundle
        )
        attestation_schema_invalid_summary_file.write_text(
            json.dumps(attestation_schema_invalid_summary, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )

        attestation_schema_invalid_result = run_real_node_policy_check(
            report_file=attestation_schema_invalid_summary_file,
            output_json=attestation_schema_invalid_policy_file,
            expected_final_decision="NO-GO",
        )
        if attestation_schema_invalid_result.returncode == 0:
            print("expected schema-invalid signer attestation proof to fail closed", file=sys.stderr)
            return 1
        attestation_schema_invalid_policy = json.loads(
            attestation_schema_invalid_policy_file.read_text(encoding="utf-8")
        )
        attestation_schema_invalid_reason_codes = attestation_schema_invalid_policy.get(
            "reason_codes"
        )
        if not isinstance(attestation_schema_invalid_reason_codes, list):
            print(
                "expected reason_codes list in schema-invalid signer attestation policy output",
                file=sys.stderr,
            )
            return 1
        if "runtime_signer_attestation_schema_invalid" not in attestation_schema_invalid_reason_codes:
            print(
                "expected runtime_signer_attestation_schema_invalid in schema-invalid signer attestation policy output",
                file=sys.stderr,
            )
            return 1
        if attestation_schema_invalid_policy.get("final_decision") != "NO-GO":
            print(
                "expected NO-GO final decision for schema-invalid signer attestation policy output",
                file=sys.stderr,
            )
            return 1

        key_source_matrix_drift_summary_file = negative_path / "key_source_matrix_drift_summary.json"
        key_source_matrix_drift_policy_file = negative_path / "key_source_matrix_drift_policy.json"
        key_source_matrix_drift_summary = dict(summary)
        key_source_matrix_drift_summary["runtime_signer_profile"] = "ops-secondary"
        key_source_matrix_drift_summary["runtime_signer_previous_profile"] = "ops-secondary"
        key_source_matrix_drift_summary["runtime_signer_key_source"] = "managed-external"
        key_source_matrix_drift_summary["runtime_signer_private_key_env"] = (
            SIGNER_PRIVATE_KEY_ENV_BY_PROFILE["ops-secondary"]
        )
        key_source_matrix_drift_summary["runtime_commit_command"] = runtime_commit_command.replace(
            expected_signer_command_marker,
            "KAMN_KOLME_LIVE_SIGNER_PROFILE=ops-secondary",
        )
        key_source_matrix_drift_contracts = dict(summary.get("contracts", {}))
        key_source_matrix_drift_contracts["runtime_signer_profile"] = "ops-secondary"
        key_source_matrix_drift_contracts["runtime_signer_key_source"] = "managed-external"
        key_source_matrix_drift_contracts["runtime_signer_private_key_env"] = (
            SIGNER_PRIVATE_KEY_ENV_BY_PROFILE["ops-secondary"]
        )
        key_source_matrix_drift_summary["contracts"] = key_source_matrix_drift_contracts
        key_source_matrix_drift_summary_file.write_text(
            json.dumps(key_source_matrix_drift_summary, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )

        key_source_matrix_drift_result = run_real_node_policy_check(
            report_file=key_source_matrix_drift_summary_file,
            output_json=key_source_matrix_drift_policy_file,
            expected_final_decision="NO-GO",
        )
        if key_source_matrix_drift_result.returncode == 0:
            print("expected disallowed key-source/profile pair negative proof to fail closed", file=sys.stderr)
            return 1
        key_source_matrix_drift_policy = json.loads(
            key_source_matrix_drift_policy_file.read_text(encoding="utf-8")
        )
        key_source_matrix_drift_reason_codes = key_source_matrix_drift_policy.get("reason_codes")
        if not isinstance(key_source_matrix_drift_reason_codes, list):
            print("expected reason_codes list in key-source matrix drift policy output", file=sys.stderr)
            return 1
        if "runtime_signer_key_source_profile_pair_disallowed" not in key_source_matrix_drift_reason_codes:
            print(
                "expected runtime_signer_key_source_profile_pair_disallowed in key-source matrix drift policy output",
                file=sys.stderr,
            )
            return 1
        if key_source_matrix_drift_policy.get("final_decision") != "NO-GO":
            print("expected NO-GO final decision for key-source matrix drift policy output", file=sys.stderr)
            return 1

        key_source_command_marker_drift_summary_file = (
            negative_path / "key_source_command_marker_drift_summary.json"
        )
        key_source_command_marker_drift_policy_file = (
            negative_path / "key_source_command_marker_drift_policy.json"
        )
        key_source_command_marker_drift_summary = dict(summary)
        key_source_command_marker_drift_summary["runtime_commit_command"] = (
            runtime_commit_command.replace(
                f"KAMN_KOLME_LIVE_SIGNER_KEY_SOURCE={expected_signer_key_source}",
                "",
                1,
            )
        )
        key_source_command_marker_drift_summary_file.write_text(
            json.dumps(key_source_command_marker_drift_summary, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )

        key_source_command_marker_drift_result = run_real_node_policy_check(
            report_file=key_source_command_marker_drift_summary_file,
            output_json=key_source_command_marker_drift_policy_file,
            expected_final_decision="NO-GO",
        )
        if key_source_command_marker_drift_result.returncode == 0:
            print("expected signer key-source command marker negative proof to fail closed", file=sys.stderr)
            return 1
        key_source_command_marker_drift_policy = json.loads(
            key_source_command_marker_drift_policy_file.read_text(encoding="utf-8")
        )
        key_source_command_marker_drift_reason_codes = key_source_command_marker_drift_policy.get(
            "reason_codes"
        )
        if not isinstance(key_source_command_marker_drift_reason_codes, list):
            print(
                "expected reason_codes list in signer key-source command marker negative proof policy output",
                file=sys.stderr,
            )
            return 1
        if (
            "runtime_commit_signer_key_source_marker_missing"
            not in key_source_command_marker_drift_reason_codes
        ):
            print(
                "expected runtime_commit_signer_key_source_marker_missing in signer key-source command marker negative proof policy output",
                file=sys.stderr,
            )
            return 1
        if key_source_command_marker_drift_policy.get("final_decision") != "NO-GO":
            print(
                "expected NO-GO final decision for signer key-source command marker negative proof policy output",
                file=sys.stderr,
            )
            return 1

        managed_external_key_reference_drift_summary_file = (
            negative_path / "managed_external_key_reference_drift_summary.json"
        )
        managed_external_key_reference_drift_policy_file = (
            negative_path / "managed_external_key_reference_drift_policy.json"
        )
        managed_external_key_reference_drift_summary = dict(summary)
        managed_external_key_reference_drift_summary["runtime_signer_key_source"] = (
            "managed-external"
        )
        managed_external_key_reference_drift_contracts = dict(summary.get("contracts", {}))
        managed_external_key_reference_drift_contracts["runtime_signer_key_source"] = (
            "managed-external"
        )
        managed_external_key_reference_drift_summary["contracts"] = (
            managed_external_key_reference_drift_contracts
        )
        managed_external_key_reference_drift_summary["runtime_commit_command"] = (
            runtime_commit_command.replace(
                f"KAMN_KOLME_LIVE_SIGNER_KEY_SOURCE={expected_signer_key_source}",
                "KAMN_KOLME_LIVE_SIGNER_KEY_SOURCE=managed-external",
                1,
            )
        )
        managed_external_key_reference_drift_summary_file.write_text(
            json.dumps(managed_external_key_reference_drift_summary, sort_keys=True, indent=2)
            + "\n",
            encoding="utf-8",
        )

        managed_external_key_reference_drift_result = run_real_node_policy_check(
            report_file=managed_external_key_reference_drift_summary_file,
            output_json=managed_external_key_reference_drift_policy_file,
            expected_final_decision="NO-GO",
        )
        if managed_external_key_reference_drift_result.returncode == 0:
            print(
                "expected managed-external signer key-reference marker negative proof to fail closed",
                file=sys.stderr,
            )
            return 1
        managed_external_key_reference_drift_policy = json.loads(
            managed_external_key_reference_drift_policy_file.read_text(encoding="utf-8")
        )
        managed_external_key_reference_drift_reason_codes = (
            managed_external_key_reference_drift_policy.get("reason_codes")
        )
        if not isinstance(managed_external_key_reference_drift_reason_codes, list):
            print(
                "expected reason_codes list in managed-external signer key-reference marker negative proof policy output",
                file=sys.stderr,
            )
            return 1
        if (
            "runtime_commit_managed_external_signer_key_reference_marker_missing"
            not in managed_external_key_reference_drift_reason_codes
        ):
            print(
                "expected runtime_commit_managed_external_signer_key_reference_marker_missing in managed-external signer key-reference marker negative proof policy output",
                file=sys.stderr,
            )
            return 1
        if managed_external_key_reference_drift_policy.get("final_decision") != "NO-GO":
            print(
                "expected NO-GO final decision for managed-external signer key-reference marker negative proof policy output",
                file=sys.stderr,
            )
            return 1

        managed_external_public_key_drift_summary_file = (
            negative_path / "managed_external_public_key_drift_summary.json"
        )
        managed_external_public_key_drift_policy_file = (
            negative_path / "managed_external_public_key_drift_policy.json"
        )
        managed_external_public_key_drift_summary = dict(
            managed_external_key_reference_drift_summary
        )
        managed_external_public_key_drift_summary["runtime_commit_command"] = (
            managed_external_key_reference_drift_summary["runtime_commit_command"].replace(
                "KAMN_KOLME_LIVE_SIGNER_KEY_SOURCE=managed-external",
                "KAMN_KOLME_LIVE_SIGNER_KEY_SOURCE=managed-external "
                f"{expected_signer_key_reference_env}=secure:aws-kms:role-operator/key-live-{expected_signer_profile}",
                1,
            )
        )
        managed_external_public_key_drift_summary_file.write_text(
            json.dumps(managed_external_public_key_drift_summary, sort_keys=True, indent=2)
            + "\n",
            encoding="utf-8",
        )

        managed_external_public_key_drift_result = run_real_node_policy_check(
            report_file=managed_external_public_key_drift_summary_file,
            output_json=managed_external_public_key_drift_policy_file,
            expected_final_decision="NO-GO",
        )
        if managed_external_public_key_drift_result.returncode == 0:
            print(
                "expected managed-external signer public-key marker negative proof to fail closed",
                file=sys.stderr,
            )
            return 1
        managed_external_public_key_drift_policy = json.loads(
            managed_external_public_key_drift_policy_file.read_text(encoding="utf-8")
        )
        managed_external_public_key_drift_reason_codes = (
            managed_external_public_key_drift_policy.get("reason_codes")
        )
        if not isinstance(managed_external_public_key_drift_reason_codes, list):
            print(
                "expected reason_codes list in managed-external signer public-key marker negative proof policy output",
                file=sys.stderr,
            )
            return 1
        if (
            "runtime_commit_managed_external_signer_public_key_marker_missing"
            not in managed_external_public_key_drift_reason_codes
        ):
            print(
                "expected runtime_commit_managed_external_signer_public_key_marker_missing in managed-external signer public-key marker negative proof policy output",
                file=sys.stderr,
            )
            return 1
        if managed_external_public_key_drift_policy.get("final_decision") != "NO-GO":
            print(
                "expected NO-GO final decision for managed-external signer public-key marker negative proof policy output",
                file=sys.stderr,
            )
            return 1

        managed_external_private_key_command_drift_summary_file = (
            negative_path / "managed_external_private_key_command_drift_summary.json"
        )
        managed_external_private_key_command_drift_policy_file = (
            negative_path / "managed_external_private_key_command_drift_policy.json"
        )
        managed_external_private_key_command_drift_summary = dict(
            managed_external_public_key_drift_summary
        )
        managed_external_private_key_command_drift_summary["runtime_commit_command"] = (
            managed_external_public_key_drift_summary["runtime_commit_command"].replace(
                f"{expected_signer_key_reference_env}=secure:aws-kms:role-operator/key-live-{expected_signer_profile}",
                f"{expected_signer_key_reference_env}=secure:aws-kms:role-operator/key-live-{expected_signer_profile} "
                f"{expected_signer_public_key_env}=0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798 "
                f"{expected_signer_private_key_env}=1111111111111111111111111111111111111111111111111111111111111111",
                1,
            )
        )
        managed_external_private_key_command_drift_summary_file.write_text(
            json.dumps(managed_external_private_key_command_drift_summary, sort_keys=True, indent=2)
            + "\n",
            encoding="utf-8",
        )

        managed_external_private_key_command_drift_result = run_real_node_policy_check(
            report_file=managed_external_private_key_command_drift_summary_file,
            output_json=managed_external_private_key_command_drift_policy_file,
            expected_final_decision="NO-GO",
        )
        if managed_external_private_key_command_drift_result.returncode == 0:
            print(
                "expected managed-external private key command marker negative proof to fail closed",
                file=sys.stderr,
            )
            return 1
        managed_external_private_key_command_drift_policy = json.loads(
            managed_external_private_key_command_drift_policy_file.read_text(encoding="utf-8")
        )
        managed_external_private_key_command_drift_reason_codes = (
            managed_external_private_key_command_drift_policy.get("reason_codes")
        )
        if not isinstance(managed_external_private_key_command_drift_reason_codes, list):
            print(
                "expected reason_codes list in managed-external private key command marker negative proof policy output",
                file=sys.stderr,
            )
            return 1
        if (
            "runtime_commit_managed_external_private_key_command_marker_detected"
            not in managed_external_private_key_command_drift_reason_codes
        ):
            print(
                "expected runtime_commit_managed_external_private_key_command_marker_detected in managed-external private key command marker negative proof policy output",
                file=sys.stderr,
            )
            return 1
        if managed_external_private_key_command_drift_policy.get("final_decision") != "NO-GO":
            print(
                "expected NO-GO final decision for managed-external private key command marker negative proof policy output",
                file=sys.stderr,
            )
            return 1

        synthetic_regression_summary_file = negative_path / "synthetic_regression_summary.json"
        synthetic_regression_policy_file = negative_path / "synthetic_regression_policy.json"
        synthetic_regression_summary = dict(summary)
        synthetic_regression_summary["runtime_commit_command"] = (
            "KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_runtime_commit_live_finality_evidence_contract_lane.sh "
            "--expected-provider-client-contract KolmeRuntimeCommitLiveProvider "
            "--require-non-synthetic-run-evidence "
            "--live-command \"printf 'runtime=synthetic\\\\n'\" "
            "--output-json /tmp/runtime-summary.json --policy-output-json /tmp/runtime-policy.json"
        )
        synthetic_regression_summary_file.write_text(
            json.dumps(synthetic_regression_summary, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )

        synthetic_regression_result = run_real_node_policy_check(
            report_file=synthetic_regression_summary_file,
            output_json=synthetic_regression_policy_file,
            expected_final_decision="NO-GO",
        )
        if synthetic_regression_result.returncode == 0:
            print("expected synthetic command regression negative proof to fail closed", file=sys.stderr)
            return 1
        synthetic_regression_policy = json.loads(
            synthetic_regression_policy_file.read_text(encoding="utf-8")
        )
        synthetic_regression_reason_codes = synthetic_regression_policy.get("reason_codes")
        if not isinstance(synthetic_regression_reason_codes, list):
            print("expected reason_codes list in synthetic regression policy output", file=sys.stderr)
            return 1
        if "runtime_commit_non_synthetic_submit_probe_missing" not in synthetic_regression_reason_codes:
            print(
                "expected runtime_commit_non_synthetic_submit_probe_missing in synthetic regression policy output",
                file=sys.stderr,
            )
            return 1
        if "runtime_commit_real_signing_profile_marker_missing" not in synthetic_regression_reason_codes:
            print(
                "expected runtime_commit_real_signing_profile_marker_missing in synthetic regression policy output",
                file=sys.stderr,
            )
            return 1
        if "runtime_commit_signer_profile_marker_missing" not in synthetic_regression_reason_codes:
            print(
                "expected runtime_commit_signer_profile_marker_missing in synthetic regression policy output",
                file=sys.stderr,
            )
            return 1
        if synthetic_regression_policy.get("final_decision") != "NO-GO":
            print("expected NO-GO final decision for synthetic regression policy output", file=sys.stderr)
            return 1

        in_memory_provider_summary_file = negative_path / "in_memory_provider_summary.json"
        in_memory_provider_policy_file = negative_path / "in_memory_provider_policy.json"
        in_memory_provider_summary = dict(summary)
        in_memory_provider_summary["runtime_commit_command"] = (
            "KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_runtime_commit_live_finality_evidence_contract_lane.sh "
            "--expected-provider-client-contract KolmeRuntimeCommitLiveProvider "
            "--require-non-synthetic-run-evidence "
            "--live-command \"KAMN_KOLME_LIVE_BASE_URL=http://127.0.0.1:3000 "
            f"KAMN_KOLME_LIVE_PROVIDER_HINT=kolme-fork-local KAMN_KOLME_LIVE_SIGNER_PROFILE={expected_signer_profile} KAMN_KOLME_LIVE_SIGNING_PROFILE=kolme-fork-secp256k1-v1 cargo test -p kamn-core --test kolme_runtime_commit_http_transport "
            "-- --exact integration_kolme_fork_live_node_submit_reaches_endpoint && printf 'status=submitted\\\\n'\" "
            "--provider-hint InMemoryKolmeRuntimeCommitClient "
            "--output-json /tmp/runtime-summary.json --policy-output-json /tmp/runtime-policy.json"
        )
        in_memory_provider_summary_file.write_text(
            json.dumps(in_memory_provider_summary, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )

        in_memory_provider_result = run_real_node_policy_check(
            report_file=in_memory_provider_summary_file,
            output_json=in_memory_provider_policy_file,
            expected_final_decision="NO-GO",
        )
        if in_memory_provider_result.returncode == 0:
            print("expected in-memory provider regression negative proof to fail closed", file=sys.stderr)
            return 1
        in_memory_provider_policy = json.loads(in_memory_provider_policy_file.read_text(encoding="utf-8"))
        in_memory_provider_reason_codes = in_memory_provider_policy.get("reason_codes")
        if not isinstance(in_memory_provider_reason_codes, list):
            print("expected reason_codes list in in-memory provider regression policy output", file=sys.stderr)
            return 1
        if "runtime_commit_in_memory_provider_reference_detected" not in in_memory_provider_reason_codes:
            print(
                "expected runtime_commit_in_memory_provider_reference_detected in in-memory provider regression policy output",
                file=sys.stderr,
            )
            return 1
        if in_memory_provider_policy.get("final_decision") != "NO-GO":
            print("expected NO-GO final decision for in-memory provider regression policy output", file=sys.stderr)
            return 1

    doc_markers = [
        "--runtime-profile real-node",
        "--runtime-signer-profile ops-secondary",
        "check_local_kamn_live_runtime_real_node_profile_policy.py",
        "run_local_kamn_live_runtime_real_node_profile_contract_lane.sh",
        "--require-non-synthetic-run-evidence",
        "runtime_signer_profile=ops-primary",
        "runtime_signer_profile=ops-secondary",
        "runtime_signer_failover_active=true",
        "runtime_signer_previous_profile=ops-primary",
        "runtime_signer_rotation_epoch=2",
        "runtime_signer_key_source_contract_version",
        "runtime_signer_key_source",
        "runtime_signing_profile=kolme-fork-secp256k1-v1",
        "runtime_signer_attestation_schema_version=kamn.kolme.runtime-signer-attestation.v1",
        "runtime_signer_attestation_bundle",
        "runtime_signer_key_reference_env=KAMN_KOLME_LIVE_SIGNER_KEY_REF",
        "runtime_signer_fallback_guard_contract_version=v2",
        "runtime_signer_fallback_guard_mode=reject_if_present",
        "runtime_signer_fallback_private_key_present=false",
        "runtime_signer_raw_private_key_present=false",
        "runtime_signer_fallback_private_key_present_violation",
        "runtime_signer_managed_external_raw_private_key_present_violation",
        "runtime_signer_attestation_schema_invalid",
        "runtime_signer_attestation_approved_signers_not_unique",
        "runtime_signer_attestation_quorum_shortfall",
        "runtime_commit_signer_profile_split_brain_detected",
        "runtime_signer_failover_profile_unchanged",
        "runtime_signer_rotation_epoch_stale",
        "runtime_signer_key_source_profile_pair_disallowed",
        "runtime_signer_private_key_env_mismatch",
        "runtime_signer_private_key_env_zeroized=true",
        "runtime_signer_private_key_bytes_zeroized=true",
        "contracts.runtime_signer_private_key_env_zeroization_required=true",
        "contracts.runtime_signer_private_key_bytes_zeroization_required=true",
        "runtime_signer_private_key_env_zeroization_violation",
        "runtime_signer_private_key_bytes_zeroization_violation",
        "runtime_commit_signer_key_source_marker_missing",
        "runtime_commit_managed_external_signer_key_reference_marker_missing",
        "runtime_commit_managed_external_signer_public_key_marker_missing",
        "runtime_commit_managed_external_private_key_command_marker_detected",
        "runtime_signing_profile_mismatch",
        "runtime_signing_profile_contract_mismatch",
        "signer_hygiene_reason_taxonomy_version=kamn.kolme.local-kamn-live-runtime-signer-hygiene-reason-taxonomy.v1",
        "signer_hygiene_reason_codes_csv=runtime_signer_private_key_env_zeroization_violation,runtime_signer_private_key_bytes_zeroization_violation",
        "Regression: #2302",
        "Regression: #2337",
        "Regression: #2325",
        "Regression: #2327",
        "Regression: #2324",
        "Regression: #2139",
    ]
    ci_doc_markers = [
        "--runtime-profile real-node",
        "--runtime-signer-profile ops-secondary",
        "check_local_kamn_live_runtime_real_node_profile_policy.py",
        "run_local_kamn_live_runtime_real_node_profile_contract_lane.sh",
        "--require-non-synthetic-run-evidence",
        "runtime_signer_profile=ops-primary",
        "runtime_signer_profile=ops-secondary",
        "runtime_signer_failover_active=true",
        "runtime_signer_previous_profile=ops-primary",
        "runtime_signer_rotation_epoch=2",
        "runtime_signer_key_source_contract_version",
        "runtime_signer_key_source",
        "runtime_signing_profile=kolme-fork-secp256k1-v1",
        "runtime_signer_attestation_schema_version=kamn.kolme.runtime-signer-attestation.v1",
        "runtime_signer_attestation_bundle",
        "runtime_signer_key_reference_env=KAMN_KOLME_LIVE_SIGNER_KEY_REF",
        "runtime_signer_fallback_guard_contract_version=v2",
        "runtime_signer_fallback_guard_mode=reject_if_present",
        "runtime_signer_fallback_private_key_present=false",
        "runtime_signer_raw_private_key_present=false",
        "runtime_signer_fallback_private_key_present_violation",
        "runtime_signer_managed_external_raw_private_key_present_violation",
        "runtime_signer_attestation_schema_invalid",
        "runtime_signer_attestation_approved_signers_not_unique",
        "runtime_signer_attestation_quorum_shortfall",
        "runtime_commit_signer_profile_split_brain_detected",
        "runtime_signer_failover_profile_unchanged",
        "runtime_signer_rotation_epoch_stale",
        "runtime_signer_key_source_profile_pair_disallowed",
        "runtime_signer_private_key_env_mismatch",
        "runtime_signer_private_key_env_zeroized=true",
        "runtime_signer_private_key_bytes_zeroized=true",
        "contracts.runtime_signer_private_key_env_zeroization_required=true",
        "contracts.runtime_signer_private_key_bytes_zeroization_required=true",
        "runtime_signer_private_key_env_zeroization_violation",
        "runtime_signer_private_key_bytes_zeroization_violation",
        "runtime_commit_signer_key_source_marker_missing",
        "runtime_commit_managed_external_signer_key_reference_marker_missing",
        "runtime_commit_managed_external_signer_public_key_marker_missing",
        "runtime_commit_managed_external_private_key_command_marker_detected",
        "runtime_signing_profile_mismatch",
        "runtime_signing_profile_contract_mismatch",
        "signer_hygiene_reason_taxonomy_version=kamn.kolme.local-kamn-live-runtime-signer-hygiene-reason-taxonomy.v1",
        "signer_hygiene_reason_codes_csv=runtime_signer_private_key_env_zeroization_violation,runtime_signer_private_key_bytes_zeroization_violation",
        "Regression: #2302",
        "Regression: #2337",
        "Regression: #2325",
        "Regression: #2327",
        "Regression: #2324",
        "Regression: #2139",
    ]
    readme_markers = [
        "--runtime-profile real-node",
        "--runtime-signer-profile ops-secondary",
        "check_local_kamn_live_runtime_real_node_profile_policy.py",
        "run_local_kamn_live_runtime_real_node_profile_contract_lane.sh",
        "--require-non-synthetic-run-evidence",
        "runtime_signer_profile=ops-primary",
        "runtime_signer_profile=ops-secondary",
        "runtime_signer_failover_active=true",
        "runtime_signer_previous_profile=ops-primary",
        "runtime_signer_rotation_epoch=2",
        "runtime_signer_key_source_contract_version",
        "runtime_signer_key_source",
        "runtime_signing_profile=kolme-fork-secp256k1-v1",
        "runtime_signer_attestation_schema_version=kamn.kolme.runtime-signer-attestation.v1",
        "runtime_signer_attestation_bundle",
        "runtime_signer_key_reference_env=KAMN_KOLME_LIVE_SIGNER_KEY_REF",
        "runtime_signer_fallback_guard_contract_version=v2",
        "runtime_signer_fallback_guard_mode=reject_if_present",
        "runtime_signer_fallback_private_key_present=false",
        "runtime_signer_raw_private_key_present=false",
        "runtime_signer_fallback_private_key_present_violation",
        "runtime_signer_managed_external_raw_private_key_present_violation",
        "runtime_signer_attestation_schema_invalid",
        "runtime_signer_attestation_approved_signers_not_unique",
        "runtime_signer_attestation_quorum_shortfall",
        "runtime_commit_signer_profile_split_brain_detected",
        "runtime_signer_failover_profile_unchanged",
        "runtime_signer_rotation_epoch_stale",
        "runtime_signer_key_source_profile_pair_disallowed",
        "runtime_signer_private_key_env_mismatch",
        "runtime_signer_private_key_env_zeroized=true",
        "runtime_signer_private_key_bytes_zeroized=true",
        "contracts.runtime_signer_private_key_env_zeroization_required=true",
        "contracts.runtime_signer_private_key_bytes_zeroization_required=true",
        "runtime_signer_private_key_env_zeroization_violation",
        "runtime_signer_private_key_bytes_zeroization_violation",
        "runtime_commit_signer_key_source_marker_missing",
        "runtime_commit_managed_external_signer_key_reference_marker_missing",
        "runtime_commit_managed_external_signer_public_key_marker_missing",
        "runtime_commit_managed_external_private_key_command_marker_detected",
        "runtime_signing_profile_mismatch",
        "runtime_signing_profile_contract_mismatch",
        "signer_hygiene_reason_taxonomy_version=kamn.kolme.local-kamn-live-runtime-signer-hygiene-reason-taxonomy.v1",
        "signer_hygiene_reason_codes_csv=runtime_signer_private_key_env_zeroization_violation,runtime_signer_private_key_bytes_zeroization_violation",
        "Regression: #2302",
        "Regression: #2337",
        "Regression: #2325",
        "Regression: #2327",
        "Regression: #2324",
        "Regression: #2139",
    ]

    missing_markers: list[str] = []
    missing_markers.extend(
        ensure_markers_present(
            DOC_FILE.read_text(encoding="utf-8"), doc_markers, "docs/planning/kolme-devnet-ops.md"
        )
    )
    missing_markers.extend(
        ensure_markers_present(
            CI_DOC_FILE.read_text(encoding="utf-8"), ci_doc_markers, "docs/ci/strategy.md"
        )
    )
    missing_markers.extend(
        ensure_markers_present(
            README_FILE.read_text(encoding="utf-8"), readme_markers, "README.md"
        )
    )
    if missing_markers:
        print(",".join(missing_markers), file=sys.stderr)
        return 1

    elapsed_seconds = int(time.monotonic() - start_epoch)
    if elapsed_seconds > max_seconds:
        print(
            f"local KAMN live runtime real-node profile contract lane exceeded runtime budget: {elapsed_seconds}s",
            file=sys.stderr,
        )
        return 1

    print("local KAMN live runtime real-node profile contract lane tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
