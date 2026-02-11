#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import pathlib
from typing import Any


def parse_bool(value: str) -> bool:
    normalized = value.strip().lower()
    if normalized in {"true", "1", "yes", "y"}:
        return True
    if normalized in {"false", "0", "no", "n"}:
        return False
    raise argparse.ArgumentTypeError(
        f"invalid boolean value '{value}', expected true/false"
    )


def read_lines(path: pathlib.Path) -> list[str]:
    return [
        line.strip()
        for line in path.read_text(encoding="utf-8").splitlines()
        if line.strip()
    ]


def parse_checkpoints(path: pathlib.Path) -> list[dict[str, str]]:
    checkpoints: list[dict[str, str]] = []
    for idx, raw_line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        if not raw_line.strip():
            continue
        parts = raw_line.split("\t")
        if len(parts) != 3:
            raise SystemExit(
                f"invalid checkpoint row at {path}:{idx}; expected 3 tab-separated fields"
            )
        checkpoint_id, command, checkpoint_status = parts
        checkpoints.append(
            {
                "id": checkpoint_id,
                "command": command,
                "status": checkpoint_status,
            }
        )
    return checkpoints


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Generate deterministic JSON summaries for local orchestration lanes."
    )
    parser.add_argument(
        "--schema-version",
        required=True,
        help="Summary schema version identifier.",
    )
    parser.add_argument(
        "--summary-type",
        required=True,
        choices=["commands", "checkpoints"],
        help="Summary payload shape to emit.",
    )
    parser.add_argument(
        "--mode",
        required=True,
        help="Lane mode value (for example dry-run or run).",
    )
    parser.add_argument(
        "--status",
        required=True,
        help="Overall lane status value.",
    )
    parser.add_argument(
        "--reason-code",
        default=None,
        help="Optional reason code marker.",
    )
    parser.add_argument(
        "--local-only-enforced",
        required=True,
        type=parse_bool,
        help="Whether local-only guard is enforced.",
    )
    parser.add_argument(
        "--commands-file",
        help="Path to newline-delimited command entries.",
    )
    parser.add_argument(
        "--checkpoints-file",
        help="Path to tab-delimited checkpoint rows: id<TAB>command<TAB>status.",
    )
    parser.add_argument(
        "--artifacts-file",
        help="Path to newline-delimited artifact paths.",
    )
    parser.add_argument(
        "--elapsed-seconds",
        type=int,
        help="Optional elapsed-seconds field.",
    )
    parser.add_argument(
        "--max-seconds",
        type=int,
        help="Optional max-seconds field.",
    )
    parser.add_argument(
        "--budget-status",
        help="Optional budget status field.",
    )
    parser.add_argument(
        "--output-json",
        required=True,
        help="Target output JSON path.",
    )
    return parser


def main() -> None:
    args = build_parser().parse_args()

    if args.summary_type == "commands":
        if not args.commands_file:
            raise SystemExit("--commands-file is required for summary-type=commands")
        if args.checkpoints_file:
            raise SystemExit(
                "--checkpoints-file is not valid for summary-type=commands"
            )
    elif args.summary_type == "checkpoints":
        if not args.checkpoints_file:
            raise SystemExit("--checkpoints-file is required for summary-type=checkpoints")
        if args.commands_file:
            raise SystemExit(
                "--commands-file is not valid for summary-type=checkpoints"
            )

    summary: dict[str, Any] = {
        "schema_version": args.schema_version,
        "summary_type": args.summary_type,
        "mode": args.mode,
        "status": args.status,
        "local_only_enforced": args.local_only_enforced,
    }

    if args.reason_code is not None:
        summary["reason_code"] = args.reason_code

    if args.summary_type == "commands":
        summary["commands"] = read_lines(pathlib.Path(args.commands_file))
    else:
        summary["checkpoints"] = parse_checkpoints(pathlib.Path(args.checkpoints_file))

    if args.artifacts_file:
        summary["artifact_paths"] = read_lines(pathlib.Path(args.artifacts_file))
    else:
        summary["artifact_paths"] = []

    if args.elapsed_seconds is not None:
        summary["elapsed_seconds"] = args.elapsed_seconds
    if args.max_seconds is not None:
        summary["max_seconds"] = args.max_seconds
    if args.budget_status is not None:
        summary["budget_status"] = args.budget_status

    output_path = pathlib.Path(args.output_json).resolve()
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(
        json.dumps(summary, sort_keys=True, indent=2) + "\n",
        encoding="utf-8",
    )


if __name__ == "__main__":
    main()
