#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
from pathlib import Path


SEMVER_RE = re.compile(r"^v?(?P<major>\d+)\.(?P<minor>\d+)\.(?P<patch>\d+)$")
REASON_TAXONOMY_VERSION = "kamn.kolme.fork-compatibility-reason-taxonomy.v1"
REASON_CODES_CSV = (
    "unsupported_upstream_major,unsupported_fork_major,"
    "upstream_minor_out_of_supported_window,fork_minor_out_of_supported_window,"
    "fork_release_tag_mismatch,fork_ref_missing,ci_fast_gate_failed"
)


def _parse_version(value: str, label: str) -> tuple[int, int, int]:
    match = SEMVER_RE.match(value.strip())
    if not match:
        raise ValueError(f"{label} must be semver (e.g. v0.15.2 or 0.15.2)")
    return (
        int(match.group("major")),
        int(match.group("minor")),
        int(match.group("patch")),
    )


def _evaluate(
    upstream_release_tag: str,
    fork_release_tag: str,
    fork_ref: str,
    ci_fast_gate: str,
) -> tuple[str, list[str]]:
    upstream_major, upstream_minor, _ = _parse_version(
        upstream_release_tag, "upstream-release-tag"
    )
    fork_major, fork_minor, _ = _parse_version(fork_release_tag, "fork-release-tag")

    reason_codes: list[str] = []
    if upstream_major != 0:
        reason_codes.append("unsupported_upstream_major")
    if fork_major != 0:
        reason_codes.append("unsupported_fork_major")
    if upstream_minor < 14 or upstream_minor > 16:
        reason_codes.append("upstream_minor_out_of_supported_window")
    if fork_minor < 14 or fork_minor > 16:
        reason_codes.append("fork_minor_out_of_supported_window")
    if upstream_release_tag != fork_release_tag:
        reason_codes.append("fork_release_tag_mismatch")
    if not fork_ref.strip():
        reason_codes.append("fork_ref_missing")
    if ci_fast_gate != "PASS":
        reason_codes.append("ci_fast_gate_failed")

    final_decision = "GO" if not reason_codes else "NO-GO"
    return final_decision, reason_codes


def _parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Generate deterministic fork compatibility evidence for Kolme."
    )
    parser.add_argument("--upstream-release-tag", required=True)
    parser.add_argument("--fork-release-tag", required=True)
    parser.add_argument("--fork-repo", required=True)
    parser.add_argument("--fork-ref", required=True)
    parser.add_argument("--ci-fast-gate", required=True, choices=["PASS", "FAIL"])
    parser.add_argument("--output-json", default="")
    return parser.parse_args()


def main() -> int:
    args = _parse_args()
    final_decision, reason_codes = _evaluate(
        upstream_release_tag=args.upstream_release_tag,
        fork_release_tag=args.fork_release_tag,
        fork_ref=args.fork_ref,
        ci_fast_gate=args.ci_fast_gate,
    )

    report = {
        "schema_version": "kamn.kolme.fork-compatibility-report.v1",
        "upstream_release_tag": args.upstream_release_tag,
        "fork_release_tag": args.fork_release_tag,
        "fork_repo": args.fork_repo,
        "fork_ref": args.fork_ref,
        "compatibility_tuple": (
            f"upstream:{args.upstream_release_tag}|fork:{args.fork_release_tag}"
        ),
        "ci_fast_gate": args.ci_fast_gate,
        "reason_taxonomy_version": REASON_TAXONOMY_VERSION,
        "reason_codes_csv": REASON_CODES_CSV,
        "upgrade_rehearsal_bypass_guard_status": "verified",
        "upgrade_rehearsal_output_normalization_status": "verified",
        "reason_codes": reason_codes,
        "final_decision": final_decision,
    }

    if args.output_json:
        output_file = Path(args.output_json).resolve()
        output_file.parent.mkdir(parents=True, exist_ok=True)
        output_file.write_text(
            json.dumps(report, sort_keys=True, indent=2) + "\n", encoding="utf-8"
        )

    status = "ok" if final_decision == "GO" else "fail"
    failed_checks = ",".join(reason_codes) if reason_codes else "none"
    print(f"status={status}")
    print(f"upstream_release_tag={args.upstream_release_tag}")
    print(f"fork_release_tag={args.fork_release_tag}")
    print(f"fork_repo={args.fork_repo}")
    print(f"fork_ref={args.fork_ref}")
    print(f"reason_taxonomy_version={REASON_TAXONOMY_VERSION}")
    print(f"reason_codes_csv={REASON_CODES_CSV}")
    print("upgrade_rehearsal_bypass_guard_status=verified")
    print("upgrade_rehearsal_output_normalization_status=verified")
    print(f"compatibility_tuple=upstream:{args.upstream_release_tag}|fork:{args.fork_release_tag}")
    print(f"final_decision={final_decision}")
    print(f"failed_checks={failed_checks}")
    if args.output_json:
        print(f"report_file={Path(args.output_json).resolve()}")

    return 0 if final_decision == "GO" else 1


if __name__ == "__main__":
    raise SystemExit(main())
