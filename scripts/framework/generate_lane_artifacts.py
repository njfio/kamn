#!/usr/bin/env python3
"""Generate or validate lane manifests and wrapper symlinks from lane registry."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import sys
from typing import Any

REGISTRY_SCHEMA_VERSION = "kamn.framework.lane-registry.v1"


def fail(message: str) -> int:
    print("status=fail", file=sys.stderr)
    print(f"error={message}", file=sys.stderr)
    return 1


def load_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Generate/check lane artifacts from lane_registry.json."
    )
    parser.add_argument("--registry-file", required=True, help="Lane registry JSON file.")
    parser.add_argument("--repo-root", required=True, help="Repository root path.")
    parser.add_argument(
        "--mode",
        choices=("check", "render"),
        default="check",
        help="check verifies repository artifacts; render writes artifacts to --output-root.",
    )
    parser.add_argument(
        "--output-root",
        default="",
        help="Render destination root when --mode=render.",
    )
    return parser.parse_args(argv)


def _validate_manifest_entry(entry: Any) -> tuple[str, dict[str, Any]]:
    if not isinstance(entry, dict):
        raise ValueError("manifest entry must be an object")
    relpath = entry.get("manifest_relpath")
    payload = entry.get("manifest_payload")
    if not isinstance(relpath, str) or not relpath.startswith("scripts/framework/manifests/"):
        raise ValueError("manifest_relpath must be under scripts/framework/manifests/")
    if not relpath.endswith(".json"):
        raise ValueError("manifest_relpath must end with .json")
    if not isinstance(payload, dict):
        raise ValueError(f"manifest_payload must be object for {relpath}")
    return relpath, payload


def _validate_wrapper_entry(entry: Any) -> tuple[str, str, str]:
    if not isinstance(entry, dict):
        raise ValueError("wrapper entry must be an object")
    wrapper_relpath = entry.get("wrapper_relpath")
    wrapper_name = entry.get("wrapper_name")
    link_target = entry.get("link_target")
    if not isinstance(wrapper_relpath, str) or not wrapper_relpath.startswith("scripts/"):
        raise ValueError("wrapper_relpath must start with scripts/")
    if not isinstance(wrapper_name, str) or not wrapper_name:
        raise ValueError(f"wrapper_name missing for {wrapper_relpath}")
    if not wrapper_relpath.endswith(wrapper_name):
        raise ValueError(
            f"wrapper_relpath basename mismatch for {wrapper_relpath}: expected suffix {wrapper_name}"
        )
    if not isinstance(link_target, str) or not link_target:
        raise ValueError(f"link_target missing for {wrapper_relpath}")
    return wrapper_relpath, wrapper_name, link_target


def run_check(
    repo_root: Path,
    manifest_entries: list[tuple[str, dict[str, Any]]],
    wrapper_entries: list[tuple[str, str, str]],
) -> int:
    manifest_drift = 0
    for relpath, expected_payload in manifest_entries:
        manifest_path = repo_root / relpath
        if not manifest_path.is_file():
            return fail(f"manifest not found: {relpath}")
        try:
            actual_payload = load_json(manifest_path)
        except Exception as exc:  # noqa: BLE001
            return fail(f"failed to parse manifest {relpath}: {exc}")
        if actual_payload != expected_payload:
            manifest_drift += 1

    wrapper_drift = 0
    for wrapper_relpath, _wrapper_name, link_target in wrapper_entries:
        wrapper_path = repo_root / wrapper_relpath
        if not wrapper_path.exists():
            return fail(f"wrapper not found: {wrapper_relpath}")
        if not wrapper_path.is_symlink():
            return fail(f"wrapper is not symlink: {wrapper_relpath}")
        actual_link = wrapper_path.readlink().as_posix()
        if actual_link != link_target:
            wrapper_drift += 1

    if manifest_drift > 0:
        return fail(f"manifest drift detected: {manifest_drift} entries")
    if wrapper_drift > 0:
        return fail(f"wrapper drift detected: {wrapper_drift} entries")

    print("status=ok")
    print("validation_mode=check")
    print(f"registry_schema_version={REGISTRY_SCHEMA_VERSION}")
    print(f"manifest_entries={len(manifest_entries)}")
    print(f"wrapper_entries={len(wrapper_entries)}")
    print("manifest_drift=0")
    print("wrapper_drift=0")
    return 0


def run_render(
    output_root: Path,
    manifest_entries: list[tuple[str, dict[str, Any]]],
    wrapper_entries: list[tuple[str, str, str]],
) -> int:
    for relpath, payload in manifest_entries:
        write_json(output_root / relpath, payload)

    for wrapper_relpath, _wrapper_name, link_target in wrapper_entries:
        wrapper_path = output_root / wrapper_relpath
        wrapper_path.parent.mkdir(parents=True, exist_ok=True)
        if wrapper_path.exists() or wrapper_path.is_symlink():
            wrapper_path.unlink()
        wrapper_path.symlink_to(Path(link_target))

    print("status=ok")
    print("validation_mode=render")
    print(f"registry_schema_version={REGISTRY_SCHEMA_VERSION}")
    print(f"manifest_entries={len(manifest_entries)}")
    print(f"wrapper_entries={len(wrapper_entries)}")
    print(f"output_root={output_root}")
    return 0


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    repo_root = Path(args.repo_root).resolve()
    registry_file = Path(args.registry_file).resolve()

    if not repo_root.is_dir():
        return fail(f"repo root does not exist: {repo_root}")
    if not registry_file.is_file():
        return fail(f"registry file not found: {registry_file}")

    try:
        registry = load_json(registry_file)
    except Exception as exc:  # noqa: BLE001
        return fail(f"failed to parse registry file: {exc}")

    if not isinstance(registry, dict):
        return fail("registry root must be an object")
    if registry.get("schema_version") != REGISTRY_SCHEMA_VERSION:
        return fail("registry schema_version mismatch")

    manifests = registry.get("manifests")
    wrappers = registry.get("wrappers")
    if not isinstance(manifests, list):
        return fail("registry manifests must be an array")
    if not isinstance(wrappers, list):
        return fail("registry wrappers must be an array")

    try:
        manifest_entries = [_validate_manifest_entry(entry) for entry in manifests]
        wrapper_entries = [_validate_wrapper_entry(entry) for entry in wrappers]
    except ValueError as exc:
        return fail(str(exc))

    if args.mode == "check":
        return run_check(repo_root, manifest_entries, wrapper_entries)

    output_root_raw = args.output_root.strip()
    if not output_root_raw:
        return fail("--output-root is required for render mode")
    output_root = Path(output_root_raw).resolve()
    output_root.mkdir(parents=True, exist_ok=True)
    return run_render(output_root, manifest_entries, wrapper_entries)


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
