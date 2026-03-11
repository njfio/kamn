#!/usr/bin/env python3
import json
import sys
from pathlib import Path


def summary_line(report_name: str, status: str) -> str:
    return f"{report_name}_report_status={status}\n"


def write_placeholder(path: Path, payload: dict) -> None:
    path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")


def append_summary(report_name: str, status: str, summary_path: Path) -> None:
    with summary_path.open("a", encoding="utf-8") as handle:
        handle.write(summary_line(report_name, status))


def main() -> int:
    report_name, output_path, payload_kind, summary_path = sys.argv[1:5]
    output = Path(output_path)
    summary = Path(summary_path)
    if output.exists():
      append_summary(report_name, "generated_by_scan", summary)
      return 0

    if payload_kind == "sbom":
      payload = {
          "bomFormat": "CycloneDX",
          "specVersion": "1.6",
          "version": 1,
          "metadata": {"component": {"name": "kamn-supply-chain-advisory", "type": "application"}},
          "properties": [
              {"name": "kamn:status", "value": "placeholder_due_to_missing_output"},
              {"name": "kamn:report", "value": "sbom"},
              {"name": "kamn:reason", "value": "scan_step_did_not_emit_output"},
          ],
      }
    else:
      payload = {
          "status": "placeholder_due_to_missing_output",
          "report": payload_kind,
          "generator": "ci-supply-chain-advisory",
          "reason": "scan_step_did_not_emit_output",
      }
    write_placeholder(output, payload)
    append_summary(report_name, "placeholder_due_to_missing_output", summary)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
