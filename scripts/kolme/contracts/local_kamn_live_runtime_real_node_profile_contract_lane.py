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
    if "KAMN_KOLME_LIVE_SIGNER_PROFILE=ops-primary" not in runtime_commit_command:
        print("expected signer profile marker in contract-lane summary command", file=sys.stderr)
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
    if summary.get("runtime_signer_profile") != "ops-primary":
        print("expected signer profile marker in contract-lane summary", file=sys.stderr)
        return 1
    if summary.get("runtime_signer_previous_profile") != "ops-primary":
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
    if summary.get("runtime_signer_private_key_env") != "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX":
        print("expected signer private key env marker in contract-lane summary", file=sys.stderr)
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
    if contracts.get("runtime_signer_profile") != "ops-primary":
        print("expected contracts signer profile marker in contract-lane summary", file=sys.stderr)
        return 1
    if contracts.get("runtime_signer_failover_requires_profile_change") is not True:
        print("expected contracts failover profile-change guard marker in contract-lane summary", file=sys.stderr)
        return 1
    if contracts.get("runtime_signer_rotation_epoch_must_increase_on_failover") is not True:
        print("expected contracts signer rotation epoch guard marker in contract-lane summary", file=sys.stderr)
        return 1
    if contracts.get("runtime_signer_private_key_env") != "KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX":
        print("expected contracts signer private key env marker in contract-lane summary", file=sys.stderr)
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

        failover_stale_summary_file = negative_path / "failover_stale_summary.json"
        failover_stale_policy_file = negative_path / "failover_stale_policy.json"
        failover_stale_summary = dict(summary)
        failover_stale_summary["runtime_signer_profile"] = "ops-primary"
        failover_stale_summary["runtime_signer_previous_profile"] = "ops-primary"
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
            "KAMN_KOLME_LIVE_PROVIDER_HINT=kolme-fork-local KAMN_KOLME_LIVE_SIGNER_PROFILE=ops-primary KAMN_KOLME_LIVE_SIGNING_PROFILE=kolme-fork-secp256k1-v1 cargo test -p kamn-core --test kolme_runtime_commit_http_transport "
            "-- --ignored --exact integration_kolme_fork_live_node_submit_reaches_endpoint && printf 'status=submitted\\\\n'\" "
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
        "check_local_kamn_live_runtime_real_node_profile_policy.py",
        "run_local_kamn_live_runtime_real_node_profile_contract_lane.sh",
        "--require-non-synthetic-run-evidence",
        "runtime_signer_profile=ops-primary",
        "runtime_signer_failover_profile_unchanged",
        "runtime_signer_rotation_epoch_stale",
        "Regression: #2139",
    ]
    ci_doc_markers = [
        "--runtime-profile real-node",
        "check_local_kamn_live_runtime_real_node_profile_policy.py",
        "run_local_kamn_live_runtime_real_node_profile_contract_lane.sh",
        "--require-non-synthetic-run-evidence",
        "runtime_signer_profile=ops-primary",
        "runtime_signer_failover_profile_unchanged",
        "runtime_signer_rotation_epoch_stale",
        "Regression: #2139",
    ]
    readme_markers = [
        "--runtime-profile real-node",
        "check_local_kamn_live_runtime_real_node_profile_policy.py",
        "run_local_kamn_live_runtime_real_node_profile_contract_lane.sh",
        "--require-non-synthetic-run-evidence",
        "runtime_signer_profile=ops-primary",
        "runtime_signer_failover_profile_unchanged",
        "runtime_signer_rotation_epoch_stale",
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
