#!/usr/bin/env python3
"""Contract lane dispatcher implementation shared by legacy wrappers."""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path

FALLBACK_REASON_TAXONOMY_VERSION = "kamn.kolme.dispatch-fallback-reason-taxonomy.v1"
FALLBACK_REASON_CODES_CSV = (
    "dispatcher_unknown_wrapper,dispatcher_manifest_missing,dispatcher_phase_unmapped"
)
REQUIRED_PHASE = "contract"


def usage() -> str:
    return """Usage:
  bash scripts/kolme/run_contract_lane_dispatch.sh --lane-wrapper <wrapper-name> [--resolve-manifest-path] [-- <lane-args...>]

Wrapper compatibility mode:
  scripts/kolme/run_<lane>_contract_lane.sh [lane-args...]
"""


def emit_fallback_error(reason_code: str, reason_detail: str) -> int:
    print("dispatch_status=fail", file=sys.stderr)
    print(
        f"fallback_reason_taxonomy_version={FALLBACK_REASON_TAXONOMY_VERSION}",
        file=sys.stderr,
    )
    print(f"fallback_reason_codes_csv={FALLBACK_REASON_CODES_CSV}", file=sys.stderr)
    print(f"fallback_reason_code={reason_code}", file=sys.stderr)
    print(f"fallback_reason_detail={reason_detail}", file=sys.stderr)
    return 1


def parse_key_value_lines(raw_output: str) -> dict[str, str]:
    parsed: dict[str, str] = {}
    for line in raw_output.splitlines():
        if "=" not in line:
            continue
        key, value = line.split("=", 1)
        parsed[key.strip()] = value.strip()
    return parsed


def resolve_manifest_record(*, kamn_root: Path, wrapper_name: str) -> dict[str, str]:
    result = subprocess.run(
        [
            "python3",
            str(kamn_root / "scripts/kolme/resolve_manifest.py"),
            "--manifests-dir",
            str(kamn_root / "scripts/framework/manifests"),
            "--wrapper-name",
            wrapper_name,
            "--required-phase",
            REQUIRED_PHASE,
        ],
        check=False,
        text=True,
        capture_output=True,
    )
    return parse_key_value_lines(result.stdout)


def parse_dispatch_args(
    *, script_name: str, argv: list[str]
) -> tuple[str, bool, list[str], str | None]:
    wrapper_name = script_name
    resolve_manifest_only = False
    lane_args = list(argv)

    if script_name != "run_contract_lane_dispatch.sh":
        return wrapper_name, resolve_manifest_only, lane_args, None

    lane_args = []
    idx = 0
    while idx < len(argv):
        arg = argv[idx]
        if arg == "--lane-wrapper":
            if idx + 1 >= len(argv):
                return wrapper_name, resolve_manifest_only, lane_args, "missing value for --lane-wrapper"
            wrapper_name = argv[idx + 1]
            idx += 2
            continue
        if arg == "--resolve-manifest-path":
            resolve_manifest_only = True
            idx += 1
            continue
        if arg == "--":
            lane_args = argv[idx + 1 :]
            break
        return wrapper_name, resolve_manifest_only, lane_args, f"unknown dispatcher argument: {arg}"

    if not wrapper_name or wrapper_name == "run_contract_lane_dispatch.sh":
        return (
            wrapper_name,
            resolve_manifest_only,
            lane_args,
            "--lane-wrapper is required when invoking the dispatcher directly",
        )
    return wrapper_name, resolve_manifest_only, lane_args, None


def main(argv: list[str]) -> int:
    script_name = os.environ.get("KAMN_DISPATCH_SCRIPT_NAME", Path(sys.argv[0]).name)
    wrapper_name, resolve_manifest_only, lane_args, parse_error = parse_dispatch_args(
        script_name=script_name,
        argv=argv,
    )
    if parse_error:
        print(parse_error, file=sys.stderr)
        print(usage(), file=sys.stderr, end="")
        return 1

    kamn_root = Path(os.environ.get("KAMN_ROOT", Path(__file__).resolve().parents[2])).resolve()
    resolved = resolve_manifest_record(kamn_root=kamn_root, wrapper_name=wrapper_name)
    if resolved.get("status") != "ok":
        error_code = resolved.get("error_code", "")
        error_detail = resolved.get("error_detail", "")
        if error_code == "unknown_wrapper":
            return emit_fallback_error(
                "dispatcher_unknown_wrapper",
                error_detail or f"unknown lane wrapper for dispatch: {wrapper_name}",
            )
        if error_code in {
            "invalid_phase",
            "duplicate_wrapper",
            "required_phase_mismatch",
            "invalid_manifest",
        }:
            return emit_fallback_error(
                "dispatcher_phase_unmapped",
                error_detail or f"unable to resolve lane phase for wrapper: {wrapper_name}",
            )
        return emit_fallback_error(
            "dispatcher_unknown_wrapper",
            f"manifest resolver failed for wrapper: {wrapper_name}",
        )

    manifest_path = resolved.get("manifest_path", "")
    phase_name = resolved.get("phase", "")
    if not manifest_path:
        return emit_fallback_error(
            "dispatcher_manifest_missing",
            f"manifest resolver returned empty manifest path for wrapper: {wrapper_name}",
        )
    if not Path(manifest_path).is_file():
        return emit_fallback_error(
            "dispatcher_manifest_missing",
            f"resolved manifest does not exist: {manifest_path}",
        )
    if resolve_manifest_only:
        print(manifest_path)
        return 0
    if phase_name != REQUIRED_PHASE:
        return emit_fallback_error(
            "dispatcher_phase_unmapped",
            f"manifest resolver returned non-contract phase for wrapper: {wrapper_name}",
        )

    return subprocess.call(
        [
            "bash",
            str(kamn_root / "scripts/framework/run_manifest_lane.sh"),
            "--manifest",
            manifest_path,
            "--phase",
            phase_name,
            "--",
            *lane_args,
        ]
    )


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
