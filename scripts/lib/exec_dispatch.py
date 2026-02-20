#!/usr/bin/env python3
"""Universal wrapper dispatcher backed by exec registry metadata."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import shutil
import sys
from typing import Any


def emit_failure(code: str, detail: str) -> int:
    print("dispatch_status=fail", file=sys.stderr)
    print(f"dispatch_error_code={code}", file=sys.stderr)
    print(f"dispatch_error_detail={detail}", file=sys.stderr)
    return 1


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Dispatch a tiny wrapper via exec registry.")
    parser.add_argument("--registry", required=True, help="Path to exec registry JSON file.")
    parser.add_argument(
        "--invoked-path",
        required=True,
        help="The wrapper path used to invoke the dispatcher ($0).",
    )
    parser.add_argument(
        "forward_args",
        nargs=argparse.REMAINDER,
        help="Arguments forwarded to the registered target after '--'.",
    )
    return parser.parse_args(argv)


def absolute_invoked_path(invoked_path: str) -> Path:
    candidate = Path(invoked_path)
    if candidate.is_absolute():
        return Path(os.path.abspath(candidate))
    if "/" in invoked_path:
        return Path(os.path.abspath(Path.cwd() / candidate))

    discovered = shutil.which(invoked_path)
    if discovered is not None:
        return Path(os.path.abspath(discovered))
    return Path(os.path.abspath(Path.cwd() / candidate))


def count_lines(path: Path) -> int:
    with path.open("r", encoding="utf-8") as handle:
        return sum(1 for _ in handle)


def expand_args_prefix(args_prefix: list[str], *, repo_root: Path) -> list[str]:
    """Expand supported registry tokens in args_prefix."""
    root = str(repo_root)
    return [
        item.replace("${KAMN_ROOT}", root).replace("${ROOT_DIR}", root)
        for item in args_prefix
    ]


def is_declarative_policy_migration_v1_candidate(
    *,
    wrapper_rel: str,
    interpreter: Any,
    target: Any,
    target_abs: Path,
) -> bool:
    if interpreter != "python3":
        return False
    if not isinstance(wrapper_rel, str):
        return False
    if not wrapper_rel.startswith("scripts/"):
        return False
    if (
        "/check_" not in wrapper_rel
        and "/validate_" not in wrapper_rel
    ) or not wrapper_rel.endswith(".sh"):
        return False
    if not isinstance(target, str):
        return False
    if not (target.endswith("_contract.py") or target.endswith("_policy_contract.py")):
        return False
    if not target_abs.is_file():
        return False
    return count_lines(target_abs) <= 1500


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    registry_path = Path(args.registry)
    if not registry_path.is_file():
        return emit_failure(
            "registry_missing",
            f"exec registry file not found: {registry_path}",
        )

    try:
        payload = json.loads(registry_path.read_text(encoding="utf-8"))
    except Exception as exc:  # noqa: BLE001
        return emit_failure("registry_invalid_json", f"failed to parse registry: {exc}")

    entries = payload.get("entries")
    if not isinstance(entries, dict):
        return emit_failure("registry_invalid_shape", "registry entries must be an object map")

    repo_root = registry_path.parent.parent.parent
    wrapper_abs = absolute_invoked_path(args.invoked_path)
    wrapper_rel = os.path.relpath(wrapper_abs, repo_root).replace("\\", "/")
    if wrapper_rel.startswith("../"):
        return emit_failure(
            "wrapper_outside_repo",
            f"wrapper path is outside repository root: {wrapper_abs}",
        )

    entry = entries.get(wrapper_rel)
    if not isinstance(entry, dict):
        return emit_failure(
            "wrapper_unregistered",
            f"no registry entry for wrapper: {wrapper_rel}",
        )

    interpreter = entry.get("interpreter")
    target = entry.get("target")
    args_prefix = entry.get("args_prefix")
    passthrough = entry.get("passthrough")

    if interpreter not in {"python3", "bash"}:
        return emit_failure(
            "entry_invalid_interpreter",
            f"invalid interpreter for {wrapper_rel}: {interpreter!r}",
        )
    if not isinstance(target, str) or target.strip() == "":
        return emit_failure(
            "entry_invalid_target",
            f"invalid target for {wrapper_rel}: {target!r}",
        )
    if not isinstance(args_prefix, list) or any(not isinstance(item, str) for item in args_prefix):
        return emit_failure(
            "entry_invalid_args_prefix",
            f"args_prefix must be a string list for {wrapper_rel}",
        )
    if not isinstance(passthrough, bool):
        return emit_failure(
            "entry_invalid_passthrough",
            f"passthrough must be boolean for {wrapper_rel}",
        )

    target_abs = repo_root / target
    if not target_abs.exists():
        return emit_failure(
            "entry_target_missing",
            f"target path does not exist for {wrapper_rel}: {target}",
        )

    forward_args = list(args.forward_args)
    if forward_args and forward_args[0] == "--":
        forward_args = forward_args[1:]

    expanded_args_prefix = expand_args_prefix(
        args_prefix,
        repo_root=repo_root,
    )

    if is_declarative_policy_migration_v1_candidate(
        wrapper_rel=wrapper_rel,
        interpreter=interpreter,
        target=target,
        target_abs=target_abs,
    ):
        checker = repo_root / "scripts/framework/declarative_policy_checker.py"
        command = [
            "python3",
            str(checker),
            "--legacy-interpreter",
            interpreter,
            "--legacy-target",
            str(target_abs),
        ]
        for prefix_arg in expanded_args_prefix:
            command.append(f"--legacy-args-prefix={prefix_arg}")
        if passthrough:
            command.append("--")
            command.extend(forward_args)
        os.execvp(command[0], command)
        return 0

    command: list[str] = [interpreter, str(target_abs), *expanded_args_prefix]
    if passthrough:
        command.extend(forward_args)

    os.execvp(interpreter, command)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
