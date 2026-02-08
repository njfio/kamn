#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path
from typing import Dict, List


ROOT_DIR = Path(__file__).resolve().parents[2]
LANGUAGE_ORDER = ["rust", "python", "typescript"]
RUNNERS = {
    "rust": ["bash", str(ROOT_DIR / "scripts/sdk/run_transport_profile_probe_rust.sh")],
    "python": ["bash", str(ROOT_DIR / "scripts/sdk/run_transport_profile_probe_python.sh")],
    "typescript": [
        "bash",
        str(ROOT_DIR / "scripts/sdk/run_transport_profile_probe_typescript.sh"),
    ],
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--languages", default="all")
    parser.add_argument("--output-json", default="")
    parser.add_argument("--expect-default-mode", default="in-memory")
    parser.add_argument("--expect-live-mode", default="live")
    parser.add_argument("--expect-memory-mismatch-expected", default="live")
    parser.add_argument("--expect-memory-mismatch-found", default="in-memory")
    parser.add_argument("--expect-live-mismatch-expected", default="in-memory")
    parser.add_argument("--expect-live-mismatch-found", default="live")
    return parser.parse_args()


def parse_languages(raw: str) -> List[str]:
    normalized = raw.strip().lower()
    if normalized in {"", "all"}:
        return list(LANGUAGE_ORDER)

    selected: List[str] = []
    seen = set()
    for token in normalized.split(","):
        language = token.strip()
        if not language:
            continue
        if language not in RUNNERS:
            raise ValueError(f"unsupported language selector: {language}")
        if language not in seen:
            seen.add(language)
            selected.append(language)

    if not selected:
        raise ValueError("at least one language must be selected")
    return selected


def parse_key_values(stdout: str) -> Dict[str, str]:
    result: Dict[str, str] = {}
    for raw_line in stdout.splitlines():
        line = raw_line.strip()
        if not line or "=" not in line:
            continue
        key, value = line.split("=", 1)
        result[key] = value
    return result


def sanitize(value: str) -> str:
    return value.replace("\n", " ").strip()


def run_runner(language: str) -> Dict[str, str]:
    completed = subprocess.run(
        RUNNERS[language],
        cwd=ROOT_DIR,
        text=True,
        capture_output=True,
    )

    if completed.returncode != 0:
        message = sanitize(completed.stderr or completed.stdout or "runner failed")
        return {
            "status": "error",
            "error": message,
        }

    values = parse_key_values(completed.stdout)
    status = values.get("status", "error")
    if status != "ok":
        return {
            "status": "error",
            "error": values.get("error", "runner returned non-ok status"),
        }

    return {
        "status": "ok",
        "default_transport_mode": values.get("default_transport_mode", ""),
        "live_transport_mode": values.get("live_transport_mode", ""),
        "memory_mismatch_expected": values.get("memory_mismatch_expected", ""),
        "memory_mismatch_found": values.get("memory_mismatch_found", ""),
        "live_mismatch_expected": values.get("live_mismatch_expected", ""),
        "live_mismatch_found": values.get("live_mismatch_found", ""),
    }


def main() -> int:
    args = parse_args()
    try:
        languages = parse_languages(args.languages)
    except ValueError as error:
        print(f"status=fail; reason={sanitize(str(error))}")
        return 2

    expected = {
        "default_transport_mode": args.expect_default_mode,
        "live_transport_mode": args.expect_live_mode,
        "memory_mismatch_expected": args.expect_memory_mismatch_expected,
        "memory_mismatch_found": args.expect_memory_mismatch_found,
        "live_mismatch_expected": args.expect_live_mismatch_expected,
        "live_mismatch_found": args.expect_live_mismatch_found,
    }

    failed_reasons: List[str] = []
    results: Dict[str, Dict[str, str]] = {}

    for language in languages:
        result = run_runner(language)
        results[language] = result
        if result.get("status") != "ok":
            failed_reasons.append(
                f"{language}: runner error ({result.get('error', 'unknown')})"
            )
            continue

        for key, expected_value in expected.items():
            actual_value = result.get(key, "")
            if actual_value != expected_value:
                failed_reasons.append(
                    f"{language}: {key} expected {expected_value} got {actual_value}"
                )

    if len(languages) > 1:
        reference_language = languages[0]
        reference = results.get(reference_language, {})
        for language in languages[1:]:
            candidate = results.get(language, {})
            if candidate.get("status") != "ok" or reference.get("status") != "ok":
                continue
            for key in expected:
                if candidate.get(key, "") != reference.get(key, ""):
                    failed_reasons.append(
                        f"{language}: parity mismatch for {key} against {reference_language}"
                    )

    status = "pass" if not failed_reasons else "fail"
    report = {
        "status": status,
        "languages": languages,
        "expected": expected,
        "failed_reasons": failed_reasons,
        "results": results,
    }

    if args.output_json:
        Path(args.output_json).write_text(json.dumps(report, indent=2), encoding="utf-8")

    if status == "pass":
        print(
            "status=pass; "
            f"languages={','.join(languages)}; "
            f"checks={len(expected)}"
        )
        return 0

    print(
        "status=fail; "
        f"languages={','.join(languages)}; "
        f"reason_count={len(failed_reasons)}; "
        f"reasons={' | '.join(failed_reasons)}"
    )
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
