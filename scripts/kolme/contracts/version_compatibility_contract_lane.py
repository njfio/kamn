#!/usr/bin/env python3
"""Contract lane runner for Kolme version compatibility checks."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parents[3]
VALIDATOR = ROOT_DIR / "scripts/kolme/validate_version_compatibility.py"
FORK_EVIDENCE_GENERATOR = ROOT_DIR / "scripts/kolme/generate_fork_compatibility_evidence.py"
FORK_POLICY_CHECKER = ROOT_DIR / "scripts/kolme/check_fork_compatibility_policy.py"
MATRIX_POLICY_CHECKER = (
    ROOT_DIR / "scripts/kolme/check_upgrade_compatibility_marker_matrix_policy.py"
)
REPLAY_RUNNER = ROOT_DIR / "scripts/kolme/run_version_compatibility_replay.py"
RUNTIME_COMMIT_LANE = ROOT_DIR / "scripts/kolme/run_runtime_commit_contract_lane.sh"
RUNTIME_COMMIT_REPLAY_LANE = ROOT_DIR / "scripts/kolme/run_runtime_commit_replay_contract_lane.sh"
NONCE_BROADCAST_PARITY_LANE = ROOT_DIR / "scripts/kolme/run_nonce_broadcast_parity_contract_lane.sh"
BLOCK_FALLBACK_LANE = ROOT_DIR / "scripts/kolme/run_block_fallback_reconciliation_contract_lane.sh"
LOCAL_RUNTIME_COMMIT_LIVE_LANE = ROOT_DIR / "scripts/kolme/run_local_runtime_commit_live_lane.sh"
LOCAL_RUNTIME_COMMIT_POLICY_CHECKER = (
    ROOT_DIR / "scripts/kolme/check_local_runtime_commit_live_evidence_policy.py"
)
LIVE_HTTPS_DEPENDENCY_POSTURE_CHECKER = (
    ROOT_DIR / "scripts/ci/check_kamn_core_live_https_dependency_posture.sh"
)
FAST_GATE_WORKFLOW = ROOT_DIR / ".github/workflows/ci-fast-gate.yml"
CI_TOOLS_SCRIPT = ROOT_DIR / "scripts/ci/test_ci_tools.sh"
FIXTURE_FILE = ROOT_DIR / "fixtures/kolme_compatibility/version_compatibility_cases.json"
FORK_FIXTURE_FILE = ROOT_DIR / "fixtures/kolme_compatibility/fork_compatibility_cases.json"
ROADMAP_DOC = ROOT_DIR / "docs/planning/kolme-integration-roadmap.md"
GONOGO_DOC = ROOT_DIR / "docs/foundation/release-gonogo-checklist.md"
CI_STRATEGY_DOC = ROOT_DIR / "docs/ci/strategy.md"
OPS_CONFIG_DOC = ROOT_DIR / "docs/ops/configuration.md"
DEPLOY_OPS_DOC = ROOT_DIR / "docs/deploy/kolme_devnet_ops.md"
MAX_SECONDS = 60
VERSION_COMPAT_REASON_TAXONOMY_VERSION = (
    "kamn.kolme.version-compatibility-reason-taxonomy.v1"
)
FORK_COMPAT_REASON_TAXONOMY_VERSION = (
    "kamn.kolme.fork-compatibility-reason-taxonomy.v1"
)
UPGRADE_COMPAT_MATRIX_REASON_TAXONOMY_VERSION = (
    "kamn.kolme.upgrade-compatibility-marker-matrix-reason-taxonomy.v1"
)
UPGRADE_COMPAT_RUNBOOK_REASON_TAXONOMY_VERSION = (
    "kamn.kolme.upgrade-compatibility-runbook-reason-taxonomy.v1"
)
UPGRADE_COMPAT_RUNBOOK_REASON_CODES_CSV = (
    "upgrade_compatibility_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch"
)
UPGRADE_COMPAT_RUNBOOK_MARKER_PARITY_STATUS = (
    "upgrade_compatibility_runbook_marker_parity_status=verified"
)
LIVE_HTTPS_POSTURE_REASON_TAXONOMY_VERSION = (
    "kamn.ci.kamn-core-live-https-dependency-posture-reason-taxonomy.v1"
)
LOCAL_HEAVY_RUNTIME_COMMIT_RUN_MODE_COMMAND = (
    "KAMN_KOLME_LOCAL_HEAVY=1 bash scripts/kolme/run_local_runtime_commit_live_lane.sh --mode run"
)


def run_capture(command: list[str]) -> tuple[int, str]:
    result = subprocess.run(
        command,
        cwd=ROOT_DIR,
        check=False,
        capture_output=True,
        text=True,
    )
    return result.returncode, (result.stdout or "") + (result.stderr or "")


def require_executable(path: Path, message: str) -> bool:
    if not path.is_file() or not path.stat().st_mode & 0o111:
        print(message, file=sys.stderr)
        return False
    return True


def extract_ci_tools_fast_mode_block(ci_tools_text: str) -> str:
    lines = ci_tools_text.splitlines()
    fast_mode_lines: list[str] = []
    in_fast_mode = False
    for line in lines:
        if line.strip() == 'if [ "${KAMN_CI_TOOLS_FAST_MODE:-false}" = "true" ]; then':
            in_fast_mode = True
            continue
        if in_fast_mode and line.strip() == "exit 0":
            in_fast_mode = False
            break
        if in_fast_mode:
            fast_mode_lines.append(line)
    return "\n".join(fast_mode_lines)


def main() -> int:
    if not require_executable(VALIDATOR, "expected Kolme version compatibility validator to be executable"):
        return 1
    if not require_executable(
        FORK_EVIDENCE_GENERATOR,
        "expected Kolme fork compatibility evidence generator to be executable",
    ):
        return 1
    if not require_executable(
        FORK_POLICY_CHECKER,
        "expected Kolme fork compatibility policy checker to be executable",
    ):
        return 1
    if not require_executable(
        MATRIX_POLICY_CHECKER,
        "expected Kolme upgrade compatibility marker matrix checker to be executable",
    ):
        return 1
    if not require_executable(REPLAY_RUNNER, "expected Kolme version compatibility replay runner to be executable"):
        return 1
    if not require_executable(
        RUNTIME_COMMIT_LANE,
        "expected Kolme runtime commit contract lane script to be executable",
    ):
        return 1
    if not require_executable(
        RUNTIME_COMMIT_REPLAY_LANE,
        "expected Kolme runtime commit replay contract lane script to be executable",
    ):
        return 1
    if not require_executable(
        NONCE_BROADCAST_PARITY_LANE,
        "expected Kolme nonce/broadcast parity contract lane script to be executable",
    ):
        return 1
    if not require_executable(
        BLOCK_FALLBACK_LANE,
        "expected Kolme block fallback reconciliation contract lane script to be executable",
    ):
        return 1
    if not require_executable(
        LOCAL_RUNTIME_COMMIT_LIVE_LANE,
        "expected local runtime commit live lane script to be executable",
    ):
        return 1
    if not require_executable(
        LOCAL_RUNTIME_COMMIT_POLICY_CHECKER,
        "expected local runtime commit live policy checker to be executable",
    ):
        return 1
    if not require_executable(
        LIVE_HTTPS_DEPENDENCY_POSTURE_CHECKER,
        "expected kamn-core live HTTPS dependency posture checker to be executable",
    ):
        return 1

    if not FIXTURE_FILE.is_file():
        print("expected Kolme version compatibility fixture file to exist", file=sys.stderr)
        return 1
    if not FORK_FIXTURE_FILE.is_file():
        print("expected Kolme fork compatibility fixture file to exist", file=sys.stderr)
        return 1
    if (
        not ROADMAP_DOC.is_file()
        or not GONOGO_DOC.is_file()
        or not OPS_CONFIG_DOC.is_file()
        or not DEPLOY_OPS_DOC.is_file()
    ):
        print(
            "expected Kolme roadmap, release go/no-go, ops configuration, and deploy ops docs to exist",
            file=sys.stderr,
        )
        return 1

    start_epoch = time.monotonic()
    with tempfile.TemporaryDirectory(prefix="kolme-version-compat-") as temp_dir:
        temp_path = Path(temp_dir)

        go_code, go_output = run_capture(
            [
                "python3",
                str(VALIDATOR),
                "--kamn-version",
                "1.1.0",
                "--kolme-release-tag",
                "v0.15.2",
                "--ci-fast-gate",
                "PASS",
                "--output-json",
                str(temp_path / "go-report.json"),
            ]
        )
        if go_code != 0:
            print(go_output, file=sys.stderr)
            return go_code
        if "final_decision=GO" not in go_output:
            print("expected supported Kolme/KAMN version pair to produce GO", file=sys.stderr)
            return 1
        if (
            "reason_taxonomy_version="
            f"{VERSION_COMPAT_REASON_TAXONOMY_VERSION}"
        ) not in go_output:
            print("expected version compatibility taxonomy marker for GO path", file=sys.stderr)
            return 1
        if "upgrade_rehearsal_bypass_guard_status=verified" not in go_output:
            print("expected upgrade rehearsal bypass guard marker for GO path", file=sys.stderr)
            return 1

        no_go_code, no_go_output = run_capture(
            [
                "python3",
                str(VALIDATOR),
                "--kamn-version",
                "1.2.0",
                "--kolme-release-tag",
                "v0.14.9",
                "--ci-fast-gate",
                "PASS",
                "--output-json",
                str(temp_path / "no-go-report.json"),
            ]
        )
        if no_go_code == 0:
            print("expected unsupported Kolme/KAMN version pair to fail closed", file=sys.stderr)
            return 1
        if "final_decision=NO-GO" not in no_go_output:
            print("expected unsupported Kolme/KAMN version pair to produce NO-GO", file=sys.stderr)
            return 1
        if (
            "reason_taxonomy_version="
            f"{VERSION_COMPAT_REASON_TAXONOMY_VERSION}"
        ) not in no_go_output:
            print("expected version compatibility taxonomy marker for NO-GO path", file=sys.stderr)
            return 1

        fork_go_code, fork_go_output = run_capture(
            [
                "python3",
                str(FORK_EVIDENCE_GENERATOR),
                "--upstream-release-tag",
                "v0.15.2",
                "--fork-release-tag",
                "v0.15.2",
                "--fork-repo",
                "njfio/kolme_fork",
                "--fork-ref",
                "refs/heads/main",
                "--ci-fast-gate",
                "PASS",
                "--output-json",
                str(temp_path / "fork-go-report.json"),
            ]
        )
        if fork_go_code != 0:
            print(fork_go_output, file=sys.stderr)
            return fork_go_code
        if "final_decision=GO" not in fork_go_output:
            print("expected synced fork tuple to produce GO", file=sys.stderr)
            return 1
        if (
            "reason_taxonomy_version="
            f"{FORK_COMPAT_REASON_TAXONOMY_VERSION}"
        ) not in fork_go_output:
            print("expected fork compatibility taxonomy marker for GO path", file=sys.stderr)
            return 1
        if "upgrade_rehearsal_bypass_guard_status=verified" not in fork_go_output:
            print("expected upgrade rehearsal bypass guard marker for fork GO path", file=sys.stderr)
            return 1

        fork_no_go_code, fork_no_go_output = run_capture(
            [
                "python3",
                str(FORK_EVIDENCE_GENERATOR),
                "--upstream-release-tag",
                "v0.15.2",
                "--fork-release-tag",
                "v0.14.9",
                "--fork-repo",
                "njfio/kolme_fork",
                "--fork-ref",
                "refs/heads/main",
                "--ci-fast-gate",
                "PASS",
                "--output-json",
                str(temp_path / "fork-no-go-report.json"),
            ]
        )
        if fork_no_go_code == 0:
            print("expected drifted fork tuple to fail closed", file=sys.stderr)
            return 1
        if "final_decision=NO-GO" not in fork_no_go_output:
            print("expected drifted fork tuple to produce NO-GO", file=sys.stderr)
            return 1
        if "fork_release_tag_mismatch" not in fork_no_go_output:
            print(
                "expected drifted fork tuple to emit fork_release_tag_mismatch reason code",
                file=sys.stderr,
            )
            return 1

        fork_policy_go_code, fork_policy_go_output = run_capture(
            [
                "python3",
                str(FORK_POLICY_CHECKER),
                "--report-file",
                str(temp_path / "fork-go-report.json"),
                "--expected-upstream-release-tag",
                "v0.15.2",
                "--expected-fork-release-tag",
                "v0.15.2",
                "--expected-fork-repo",
                "njfio/kolme_fork",
                "--expected-final-decision",
                "GO",
                "--ci-fast-gate",
                "PASS",
                "--output-json",
                str(temp_path / "fork-policy-go-report.json"),
            ]
        )
        if fork_policy_go_code != 0:
            print(fork_policy_go_output, file=sys.stderr)
            return fork_policy_go_code
        if "final_decision=GO" not in fork_policy_go_output:
            print("expected fork policy checker GO path to pass", file=sys.stderr)
            return 1
        if (
            "reason_taxonomy_version="
            f"{FORK_COMPAT_REASON_TAXONOMY_VERSION}"
        ) not in fork_policy_go_output:
            print("expected fork policy checker taxonomy marker for GO path", file=sys.stderr)
            return 1
        if "upgrade_rehearsal_bypass_guard_status=verified" not in fork_policy_go_output:
            print("expected fork policy checker bypass guard marker for GO path", file=sys.stderr)
            return 1

        matrix_policy_go_code, matrix_policy_go_output = run_capture(
            [
                "python3",
                str(MATRIX_POLICY_CHECKER),
                "--version-report-file",
                str(temp_path / "go-report.json"),
                "--fork-policy-report-file",
                str(temp_path / "fork-policy-go-report.json"),
                "--expected-final-decision",
                "GO",
                "--ci-fast-gate",
                "PASS",
                "--output-json",
                str(temp_path / "compatibility-marker-matrix-go-report.json"),
            ]
        )
        if matrix_policy_go_code != 0:
            print(matrix_policy_go_output, file=sys.stderr)
            return matrix_policy_go_code
        if "final_decision=GO" not in matrix_policy_go_output:
            print("expected compatibility marker matrix GO path to produce GO", file=sys.stderr)
            return 1
        if (
            "reason_taxonomy_version="
            f"{UPGRADE_COMPAT_MATRIX_REASON_TAXONOMY_VERSION}"
        ) not in matrix_policy_go_output:
            print(
                "expected compatibility marker matrix checker taxonomy marker for GO path",
                file=sys.stderr,
            )
            return 1
        if "reason_codes_value=none" not in matrix_policy_go_output:
            print(
                "expected compatibility marker matrix checker reason_codes_value=none for GO path",
                file=sys.stderr,
            )
            return 1

        fork_policy_no_go_code, fork_policy_no_go_output = run_capture(
            [
                "python3",
                str(FORK_POLICY_CHECKER),
                "--report-file",
                str(temp_path / "fork-no-go-report.json"),
                "--expected-upstream-release-tag",
                "v0.15.2",
                "--expected-fork-release-tag",
                "v0.14.9",
                "--expected-fork-repo",
                "njfio/kolme_fork",
                "--expected-final-decision",
                "NO-GO",
                "--require-reason-code",
                "fork_release_tag_mismatch",
                "--ci-fast-gate",
                "PASS",
                "--output-json",
                str(temp_path / "fork-policy-no-go-report.json"),
            ]
        )
        if fork_policy_no_go_code != 0:
            print(fork_policy_no_go_output, file=sys.stderr)
            return fork_policy_no_go_code
        if "final_decision=GO" not in fork_policy_no_go_output:
            print("expected fork policy checker expected-NO-GO path to pass", file=sys.stderr)
            return 1

        tampered_version_report = temp_path / "go-report.version-schema-tampered.json"
        version_payload = json.loads((temp_path / "go-report.json").read_text(encoding="utf-8"))
        version_payload["schema_version"] = "kamn.kolme.version-compatibility-report.v0"
        tampered_version_report.write_text(
            json.dumps(version_payload, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )

        matrix_policy_tampered_code, matrix_policy_tampered_output = run_capture(
            [
                "python3",
                str(MATRIX_POLICY_CHECKER),
                "--version-report-file",
                str(tampered_version_report),
                "--fork-policy-report-file",
                str(temp_path / "fork-policy-go-report.json"),
                "--expected-final-decision",
                "NO-GO",
                "--ci-fast-gate",
                "PASS",
                "--output-json",
                str(temp_path / "compatibility-marker-matrix-tampered-report.json"),
            ]
        )
        if matrix_policy_tampered_code == 0:
            print(
                "expected compatibility marker matrix checker to fail closed for tampered schema",
                file=sys.stderr,
            )
            return 1
        if "version_report_schema_mismatch" not in matrix_policy_tampered_output:
            print(
                "expected compatibility marker matrix checker to emit version_report_schema_mismatch",
                file=sys.stderr,
            )
            return 1

        replay_code, replay_output = run_capture(
            [
                "python3",
                str(REPLAY_RUNNER),
                "--fixture",
                str(FIXTURE_FILE),
                "--max-cases",
                "2",
                "--output-json",
                str(temp_path / "replay-smoke.json"),
            ]
        )
        if replay_code != 0:
            print(replay_output, file=sys.stderr)
            return replay_code

        for lane in (
            RUNTIME_COMMIT_LANE,
            RUNTIME_COMMIT_REPLAY_LANE,
            NONCE_BROADCAST_PARITY_LANE,
            BLOCK_FALLBACK_LANE,
        ):
            lane_result = subprocess.run(
                ["bash", str(lane)],
                cwd=ROOT_DIR,
                check=False,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.PIPE,
                text=True,
            )
            if lane_result.returncode != 0:
                print(lane_result.stderr or f"lane failed: {lane}", file=sys.stderr)
                return lane_result.returncode

        retry_tls_summary_file = temp_path / "retry-tls-smoke-summary.json"
        retry_tls_policy_file = temp_path / "retry-tls-smoke-policy.json"
        retry_tls_summary_code, retry_tls_summary_output = run_capture(
            [
                "bash",
                str(LOCAL_RUNTIME_COMMIT_LIVE_LANE),
                "--mode",
                "dry-run",
                "--output-json",
                str(retry_tls_summary_file),
            ]
        )
        if retry_tls_summary_code != 0:
            print(retry_tls_summary_output, file=sys.stderr)
            return retry_tls_summary_code
        if "reason_code=dry_run_no_commands_executed" not in retry_tls_summary_output:
            print(
                "expected retry/TLS smoke summary lane to emit dry_run_no_commands_executed",
                file=sys.stderr,
            )
            return 1
        if "local_only_enforced=true" not in retry_tls_summary_output:
            print(
                "expected retry/TLS smoke summary lane to enforce local-only boundary marker",
                file=sys.stderr,
            )
            return 1

        retry_tls_policy_code, retry_tls_policy_output = run_capture(
            [
                "python3",
                str(LOCAL_RUNTIME_COMMIT_POLICY_CHECKER),
                "--report-file",
                str(retry_tls_summary_file),
                "--expected-final-decision",
                "GO",
                "--ci-fast-gate",
                "PASS",
                "--require-reason-code",
                "dry_run_no_commands_executed",
                "--output-json",
                str(retry_tls_policy_file),
            ]
        )
        if retry_tls_policy_code != 0:
            print(retry_tls_policy_output, file=sys.stderr)
            return retry_tls_policy_code
        if "final_decision=GO" not in retry_tls_policy_output:
            print(
                "expected retry/TLS smoke policy checker to produce GO in dry-run mode",
                file=sys.stderr,
            )
            return 1

        retry_tls_summary_payload = json.loads(
            retry_tls_summary_file.read_text(encoding="utf-8")
        )
        if retry_tls_summary_payload.get("finality_retry_contract_version") != "v1":
            print(
                "expected retry/TLS smoke summary to include finality_retry_contract_version=v1",
                file=sys.stderr,
            )
            return 1
        retry_max_attempts = retry_tls_summary_payload.get("finality_retry_max_attempts")
        if not isinstance(retry_max_attempts, int) or retry_max_attempts <= 0:
            print(
                "expected retry/TLS smoke summary to include positive finality_retry_max_attempts",
                file=sys.stderr,
            )
            return 1
        retry_backoff_seconds = retry_tls_summary_payload.get("finality_retry_backoff_seconds")
        if not isinstance(retry_backoff_seconds, int) or retry_backoff_seconds < 0:
            print(
                "expected retry/TLS smoke summary to include non-negative finality_retry_backoff_seconds",
                file=sys.stderr,
            )
            return 1
        if retry_tls_summary_payload.get("local_only_enforced") is not True:
            print(
                "expected retry/TLS smoke summary local_only_enforced marker to be true",
                file=sys.stderr,
            )
            return 1

        retry_tls_policy_payload = json.loads(
            retry_tls_policy_file.read_text(encoding="utf-8")
        )
        if retry_tls_policy_payload.get("final_decision") != "GO":
            print(
                "expected retry/TLS smoke policy payload final_decision=GO",
                file=sys.stderr,
            )
            return 1
        retry_tls_reason_codes = retry_tls_policy_payload.get("reason_codes")
        if retry_tls_reason_codes != []:
            print(
                "expected retry/TLS smoke policy payload reason_codes=[] for dry-run GO path",
                file=sys.stderr,
            )
            return 1

        https_posture_report_file = temp_path / "live-https-dependency-posture-report.json"
        https_posture_code, https_posture_output = run_capture(
            [
                "bash",
                str(LIVE_HTTPS_DEPENDENCY_POSTURE_CHECKER),
                "--output-json",
                str(https_posture_report_file),
            ]
        )
        if https_posture_code != 0:
            print(https_posture_output, file=sys.stderr)
            return https_posture_code
        if "status=ok" not in https_posture_output:
            print(
                "expected live HTTPS dependency posture checker status=ok marker",
                file=sys.stderr,
            )
            return 1
        if (
            "reason_taxonomy_version="
            f"{LIVE_HTTPS_POSTURE_REASON_TAXONOMY_VERSION}"
        ) not in https_posture_output:
            print(
                "expected live HTTPS dependency posture checker taxonomy marker",
                file=sys.stderr,
            )
            return 1

        https_posture_payload = json.loads(
            https_posture_report_file.read_text(encoding="utf-8")
        )
        if https_posture_payload.get("status") != "pass":
            print(
                "expected live HTTPS dependency posture report status=pass",
                file=sys.stderr,
            )
            return 1
        if https_posture_payload.get("reason_codes_value") != "none":
            print(
                "expected live HTTPS dependency posture report reason_codes_value=none",
                file=sys.stderr,
            )
            return 1
        if https_posture_payload.get("reason_codes_csv") != "none":
            print(
                "expected live HTTPS dependency posture report reason_codes_csv=none",
                file=sys.stderr,
            )
            return 1

    roadmap_doc_text = ROADMAP_DOC.read_text(encoding="utf-8")
    gonogo_doc_text = GONOGO_DOC.read_text(encoding="utf-8")
    ci_strategy_doc_text = CI_STRATEGY_DOC.read_text(encoding="utf-8")
    ops_config_doc_text = OPS_CONFIG_DOC.read_text(encoding="utf-8")
    deploy_ops_doc_text = DEPLOY_OPS_DOC.read_text(encoding="utf-8")
    fast_gate_workflow_text = FAST_GATE_WORKFLOW.read_text(encoding="utf-8")
    ci_tools_fast_mode_block = extract_ci_tools_fast_mode_block(
        CI_TOOLS_SCRIPT.read_text(encoding="utf-8")
    )
    if "validate_version_compatibility.py" not in roadmap_doc_text:
        print("expected Kolme roadmap doc to reference version validator command", file=sys.stderr)
        return 1
    if "run_runtime_commit_contract_lane.sh" not in roadmap_doc_text:
        print("expected Kolme roadmap doc to reference runtime commit contract lane command", file=sys.stderr)
        return 1
    if "generate_fork_compatibility_evidence.py" not in roadmap_doc_text:
        print("expected Kolme roadmap doc to reference fork compatibility evidence command", file=sys.stderr)
        return 1
    if "check_fork_compatibility_policy.py" not in roadmap_doc_text:
        print("expected Kolme roadmap doc to reference fork compatibility policy checker command", file=sys.stderr)
        return 1
    if "fixtures/kolme_compatibility/fork_compatibility_cases.json" not in roadmap_doc_text:
        print("expected Kolme roadmap doc to reference fork compatibility fixture path", file=sys.stderr)
        return 1
    if "run_runtime_commit_replay_contract_lane.sh" not in roadmap_doc_text:
        print(
            "expected Kolme roadmap doc to reference runtime commit replay contract lane command",
            file=sys.stderr,
        )
        return 1
    if "run_nonce_broadcast_parity_contract_lane.sh" not in roadmap_doc_text:
        print("expected Kolme roadmap doc to reference nonce/broadcast parity contract lane command", file=sys.stderr)
        return 1
    if "run_block_fallback_reconciliation_contract_lane.sh" not in roadmap_doc_text:
        print(
            "expected Kolme roadmap doc to reference block fallback reconciliation contract lane command",
            file=sys.stderr,
        )
        return 1
    if "run_version_compatibility_replay_deep_lane.sh" not in gonogo_doc_text:
        print("expected release go/no-go doc to reference scheduled version replay lane", file=sys.stderr)
        return 1
    if "check_fork_compatibility_policy.py" not in gonogo_doc_text:
        print("expected release go/no-go doc to reference fork compatibility policy checker command", file=sys.stderr)
        return 1
    if "check_upgrade_compatibility_marker_matrix_policy.py" not in gonogo_doc_text:
        print(
            "expected release go/no-go doc to reference compatibility marker matrix checker command",
            file=sys.stderr,
        )
        return 1
    if UPGRADE_COMPAT_MATRIX_REASON_TAXONOMY_VERSION not in gonogo_doc_text:
        print(
            "expected release go/no-go doc to reference compatibility marker matrix taxonomy marker",
            file=sys.stderr,
        )
        return 1
    if UPGRADE_COMPAT_RUNBOOK_REASON_TAXONOMY_VERSION not in gonogo_doc_text:
        print(
            "expected release go/no-go doc to reference upgrade compatibility runbook taxonomy marker",
            file=sys.stderr,
        )
        return 1
    if (
        f"upgrade_compatibility_runbook_reason_codes_csv={UPGRADE_COMPAT_RUNBOOK_REASON_CODES_CSV}"
        not in gonogo_doc_text
    ):
        print(
            "expected release go/no-go doc to reference upgrade compatibility runbook reason-codes marker",
            file=sys.stderr,
        )
        return 1
    if UPGRADE_COMPAT_RUNBOOK_MARKER_PARITY_STATUS not in gonogo_doc_text:
        print(
            "expected release go/no-go doc to reference upgrade compatibility runbook marker-parity status marker",
            file=sys.stderr,
        )
        return 1
    if "check_upgrade_compatibility_marker_matrix_policy.py" not in ops_config_doc_text:
        print(
            "expected ops configuration doc to reference compatibility marker matrix checker command",
            file=sys.stderr,
        )
        return 1
    if UPGRADE_COMPAT_MATRIX_REASON_TAXONOMY_VERSION not in ops_config_doc_text:
        print(
            "expected ops configuration doc to reference compatibility marker matrix taxonomy marker",
            file=sys.stderr,
        )
        return 1
    if "check_upgrade_compatibility_marker_matrix_policy.py" not in deploy_ops_doc_text:
        print(
            "expected deploy ops doc to reference compatibility marker matrix checker command",
            file=sys.stderr,
        )
        return 1
    if UPGRADE_COMPAT_MATRIX_REASON_TAXONOMY_VERSION not in deploy_ops_doc_text:
        print(
            "expected deploy ops doc to reference compatibility marker matrix taxonomy marker",
            file=sys.stderr,
        )
        return 1
    if UPGRADE_COMPAT_RUNBOOK_REASON_TAXONOMY_VERSION not in deploy_ops_doc_text:
        print(
            "expected deploy ops doc to reference upgrade compatibility runbook taxonomy marker",
            file=sys.stderr,
        )
        return 1
    if (
        f"upgrade_compatibility_runbook_reason_codes_csv={UPGRADE_COMPAT_RUNBOOK_REASON_CODES_CSV}"
        not in deploy_ops_doc_text
    ):
        print(
            "expected deploy ops doc to reference upgrade compatibility runbook reason-codes marker",
            file=sys.stderr,
        )
        return 1
    if UPGRADE_COMPAT_RUNBOOK_MARKER_PARITY_STATUS not in deploy_ops_doc_text:
        print(
            "expected deploy ops doc to reference upgrade compatibility runbook marker-parity status marker",
            file=sys.stderr,
        )
        return 1
    if (
        "retry/tls local-heavy run-mode commands remain excluded from ci-fast-gate and ci-tools fast mode."
        not in ci_strategy_doc_text
    ):
        print(
            "expected CI strategy doc retry/TLS local-heavy exclusion marker",
            file=sys.stderr,
        )
        return 1
    if LOCAL_HEAVY_RUNTIME_COMMIT_RUN_MODE_COMMAND in fast_gate_workflow_text:
        print(
            "expected local-heavy runtime-commit run-mode command to remain excluded from ci-fast-gate workflow",
            file=sys.stderr,
        )
        return 1
    if LOCAL_HEAVY_RUNTIME_COMMIT_RUN_MODE_COMMAND in ci_tools_fast_mode_block:
        print(
            "expected local-heavy runtime-commit run-mode command to remain excluded from ci-tools fast mode",
            file=sys.stderr,
        )
        return 1

    elapsed_seconds = int(time.monotonic() - start_epoch)
    if elapsed_seconds > MAX_SECONDS:
        print(
            f"Kolme version compatibility contract lane exceeded runtime budget: {elapsed_seconds}s",
            file=sys.stderr,
        )
        return 1

    print("Kolme version compatibility contract lane tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
