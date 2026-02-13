#!/usr/bin/env python3
"""Fail-closed workspace Cargo license policy checker."""

from __future__ import annotations

import argparse
import sys
import tomllib
from pathlib import Path

DEFAULT_LICENSE = "Apache-2.0"
DEFAULT_MANIFEST_GLOBS = ("crates/*/Cargo.toml",)


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
    errors: list[str] = []
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

    return errors


def main() -> int:
    args = build_parser().parse_args()
    expected_license = args.expected_license.strip()
    if not expected_license:
        print("workspace license policy check failed: expected license must be non-empty", file=sys.stderr)
        return 1

    manifests = resolve_manifests(args)
    if not manifests:
        print("workspace license policy check failed: no crate Cargo manifests found", file=sys.stderr)
        return 1

    failures: list[str] = []
    for manifest in manifests:
        failures.extend(check_manifest(manifest, expected_license))

    if failures:
        print("workspace license policy check failed:", file=sys.stderr)
        for failure in failures:
            print(failure, file=sys.stderr)
        return 1

    print("workspace license policy check passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
