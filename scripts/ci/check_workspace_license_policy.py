#!/usr/bin/env python3
"""Fail-closed workspace Cargo license policy checker."""

from __future__ import annotations

import argparse
import json
import sys
import tomllib
from pathlib import Path

DEFAULT_LICENSE = "Apache-2.0"
DEFAULT_MANIFEST_GLOBS = ("crates/*/Cargo.toml",)
SCHEMA_VERSION = "kamn.ci.dependency-license-metadata-governance-report.v1"
REASON_TAXONOMY_VERSION = "kamn.ci.dependency-license-metadata-governance-reason-taxonomy.v1"
KNOWN_REASON_CODE_ORDER = (
    "expected_license_empty",
    "no_crate_manifests_found",
    "manifest_not_found",
    "manifest_invalid_toml",
    "package_section_missing",
    "license_missing",
    "license_mismatch",
    "metadata_governance_local_heavy_opt_in_required",
)
METADATA_MISMATCH_REASON_CODES = {
    "manifest_not_found",
    "manifest_invalid_toml",
    "package_section_missing",
    "license_missing",
    "license_mismatch",
}
CONFIGURATION_REASON_CODES = {"expected_license_empty", "no_crate_manifests_found"}
BOUNDARY_REASON_CODES = {"metadata_governance_local_heavy_opt_in_required"}


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Validate Cargo manifest license fields against workspace policy."
    )
    parser.add_argument(
        "--workspace-root",
        default=".",
        help="Workspace root used to discover Cargo manifests when --manifest is omitted.",
    )
    parser.add_argument(
        "--expected-license",
        default=DEFAULT_LICENSE,
        help="Expected SPDX license identifier.",
    )
    parser.add_argument(
        "--manifest",
        action="append",
        default=[],
        help="Explicit Cargo.toml path(s) to check. May be repeated.",
    )
    parser.add_argument(
        "--lane-profile",
        choices=("ci-smoke", "local-heavy"),
        default="ci-smoke",
        help="Execution profile for CI/local-heavy boundary enforcement.",
    )
    parser.add_argument(
        "--local-heavy-opt-in",
        action="store_true",
        help="Required explicit opt-in when --lane-profile local-heavy is selected.",
    )
    parser.add_argument(
        "--output-json",
        default="",
        help="Optional output path for machine-readable report JSON.",
    )
    return parser


def discover_manifests(workspace_root: Path) -> list[Path]:
    manifests: set[Path] = set()
    for pattern in DEFAULT_MANIFEST_GLOBS:
        for manifest in workspace_root.glob(pattern):
            if manifest.is_file():
                manifests.add(manifest.resolve())
    return sorted(manifests)


def resolve_manifests(args: argparse.Namespace) -> list[Path]:
    if args.manifest:
        manifests = [Path(path).resolve() for path in args.manifest]
        return sorted(manifests)
    return discover_manifests(Path(args.workspace_root).resolve())


def check_manifest(manifest_path: Path, expected_license: str) -> list[str]:
    if not manifest_path.is_file():
        return [f"manifest_not_found:{manifest_path}"]

    try:
        payload = tomllib.loads(manifest_path.read_text(encoding="utf-8"))
    except tomllib.TOMLDecodeError:
        return [f"manifest_invalid_toml:{manifest_path}"]

    package = payload.get("package")
    if not isinstance(package, dict):
        return [f"package_section_missing:{manifest_path}"]

    observed_license = package.get("license")
    if not isinstance(observed_license, str) or not observed_license.strip():
        return [f"license_missing:{manifest_path}"]

    normalized_license = observed_license.strip()
    if normalized_license != expected_license:
        return [
            "license_mismatch:"
            f"{manifest_path}:expected={expected_license}:observed={normalized_license}"
        ]

    return []


def sort_reason_codes(reason_codes: set[str]) -> list[str]:
    order_index = {code: idx for idx, code in enumerate(KNOWN_REASON_CODE_ORDER)}
    return sorted(reason_codes, key=lambda code: (order_index.get(code, len(order_index)), code))


def classify_reason_class(reason_codes: list[str]) -> str:
    if not reason_codes or reason_codes == ["none"]:
        return "stable"

    classes: set[str] = set()
    reason_code_set = set(reason_codes)
    if reason_code_set & METADATA_MISMATCH_REASON_CODES:
        classes.add("metadata_mismatch")
    if reason_code_set & CONFIGURATION_REASON_CODES:
        classes.add("configuration")
    if reason_code_set & BOUNDARY_REASON_CODES:
        classes.add("boundary")
    if len(classes) == 1:
        return next(iter(classes))
    return "mixed"


def main() -> int:
    args = build_parser().parse_args()

    failures: list[str] = []
    reason_codes: set[str] = set()

    expected_license = args.expected_license.strip()
    if not expected_license:
        failures.append("expected license must be non-empty")
        reason_codes.add("expected_license_empty")

    manifests = resolve_manifests(args)
    if not manifests:
        failures.append("no crate Cargo manifests found")
        reason_codes.add("no_crate_manifests_found")

    if args.lane_profile == "local-heavy":
        if args.local_heavy_opt_in:
            ci_boundary_status = "verified"
            local_heavy_mode = "opt_in"
        else:
            ci_boundary_status = "violation"
            local_heavy_mode = "blocked"
            reason_codes.add("metadata_governance_local_heavy_opt_in_required")
            failures.append("local-heavy lane requires explicit --local-heavy-opt-in")
        ci_smoke_cost_profile = "not-applicable"
    else:
        ci_boundary_status = "verified"
        ci_smoke_cost_profile = "low"
        local_heavy_mode = "not_requested"

    if expected_license:
        for manifest in manifests:
            manifest_failures = check_manifest(manifest, expected_license)
            failures.extend(manifest_failures)
            for failure in manifest_failures:
                reason_codes.add(failure.split(":", 1)[0])

    status = "pass" if not failures else "fail"
    ordered_reason_codes = ["none"] if status == "pass" else sort_reason_codes(reason_codes)
    reason_codes_csv = "none" if status == "pass" else ",".join(ordered_reason_codes)
    reason_codes_value = reason_codes_csv
    reason_class = classify_reason_class(ordered_reason_codes)

    report = {
        "schema_version": SCHEMA_VERSION,
        "reason_taxonomy_version": REASON_TAXONOMY_VERSION,
        "status": status,
        "reason_codes": ordered_reason_codes,
        "reason_codes_csv": reason_codes_csv,
        "reason_codes_value": reason_codes_value,
        "reason_class": reason_class,
        "lane_profile": args.lane_profile,
        "local_heavy_opt_in": args.local_heavy_opt_in,
        "ci_smoke_local_heavy_boundary_status": ci_boundary_status,
        "ci_smoke_lane_cost_profile": ci_smoke_cost_profile,
        "local_heavy_lane_execution_mode": local_heavy_mode,
        "manifest_count": len(manifests),
        "violation_count": len(failures),
        "violations": failures,
    }

    if args.output_json:
        output_path = Path(args.output_json)
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    if status == "pass":
        print("status=ok")
        print(f"reason_taxonomy_version={REASON_TAXONOMY_VERSION}")
        print(f"reason_codes_csv={reason_codes_csv}")
        print(f"reason_codes_value={reason_codes_value}")
        print(f"reason_class={reason_class}")
        print(f"ci_smoke_local_heavy_boundary_status={ci_boundary_status}")
        print(f"ci_smoke_lane_cost_profile={ci_smoke_cost_profile}")
        print(f"local_heavy_lane_execution_mode={local_heavy_mode}")
        print("violation_count=0")
        print("workspace license policy check passed.")
        return 0

    print("status=fail")
    print(f"reason_taxonomy_version={REASON_TAXONOMY_VERSION}")
    print(f"reason_codes_csv={reason_codes_csv}")
    print(f"reason_codes_value={reason_codes_value}")
    print(f"reason_class={reason_class}")
    print(f"ci_smoke_local_heavy_boundary_status={ci_boundary_status}")
    print(f"ci_smoke_lane_cost_profile={ci_smoke_cost_profile}")
    print(f"local_heavy_lane_execution_mode={local_heavy_mode}")
    print(f"violation_count={len(failures)}")
    for failure in failures:
        print(f"violation={failure}")

    print("workspace license policy check failed:", file=sys.stderr)
    for failure in failures:
        print(failure, file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
