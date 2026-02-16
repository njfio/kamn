#!/usr/bin/env python3
"""Contract lane runner for Kolme version compatibility checks."""

from __future__ import annotations

import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parents[3]
VALIDATOR = ROOT_DIR / "scripts/kolme/validate_version_compatibility.py"
FORK_EVIDENCE_GENERATOR = ROOT_DIR / "scripts/kolme/generate_fork_compatibility_evidence.py"
FORK_POLICY_CHECKER = ROOT_DIR / "scripts/kolme/check_fork_compatibility_policy.py"
REPLAY_RUNNER = ROOT_DIR / "scripts/kolme/run_version_compatibility_replay.py"
RUNTIME_COMMIT_LANE = ROOT_DIR / "scripts/kolme/run_runtime_commit_contract_lane.sh"
RUNTIME_COMMIT_REPLAY_LANE = ROOT_DIR / "scripts/kolme/run_runtime_commit_replay_contract_lane.sh"
NONCE_BROADCAST_PARITY_LANE = ROOT_DIR / "scripts/kolme/run_nonce_broadcast_parity_contract_lane.sh"
BLOCK_FALLBACK_LANE = ROOT_DIR / "scripts/kolme/run_block_fallback_reconciliation_contract_lane.sh"
FIXTURE_FILE = ROOT_DIR / "fixtures/kolme_compatibility/version_compatibility_cases.json"
FORK_FIXTURE_FILE = ROOT_DIR / "fixtures/kolme_compatibility/fork_compatibility_cases.json"
ROADMAP_DOC = ROOT_DIR / "docs/planning/kolme-integration-roadmap.md"
GONOGO_DOC = ROOT_DIR / "docs/foundation/release-gonogo-checklist.md"
MAX_SECONDS = 60
VERSION_COMPAT_REASON_TAXONOMY_VERSION = (
    "kamn.kolme.version-compatibility-reason-taxonomy.v1"
)
FORK_COMPAT_REASON_TAXONOMY_VERSION = (
    "kamn.kolme.fork-compatibility-reason-taxonomy.v1"
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

    if not FIXTURE_FILE.is_file():
        print("expected Kolme version compatibility fixture file to exist", file=sys.stderr)
        return 1
    if not FORK_FIXTURE_FILE.is_file():
        print("expected Kolme fork compatibility fixture file to exist", file=sys.stderr)
        return 1
    if not ROADMAP_DOC.is_file() or not GONOGO_DOC.is_file():
        print("expected Kolme roadmap and release go/no-go docs to exist", file=sys.stderr)
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

    roadmap_doc_text = ROADMAP_DOC.read_text(encoding="utf-8")
    gonogo_doc_text = GONOGO_DOC.read_text(encoding="utf-8")
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
