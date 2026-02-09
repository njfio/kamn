#!/usr/bin/env python3
"""Shared helpers for contract-lane evidence and policy scripts."""

from __future__ import annotations

from dataclasses import dataclass, field
import json
import pathlib
import re
from typing import Any, Mapping, Sequence


class ContractError(ValueError):
    """Raised when contract input or policy data is invalid."""


def fail(message: str) -> None:
    """Raise a contract error with a stable message."""
    raise ContractError(message)


def parse_int(name: str, raw_value: str) -> int:
    """Parse an integer with stable error text."""
    try:
        return int(raw_value)
    except (TypeError, ValueError):
        fail(f"{name} must be an integer")


def require_non_negative_int(name: str, raw_value: str) -> int:
    """Parse and require integer >= 0."""
    value = parse_int(name, raw_value)
    if value < 0:
        fail(f"{name} must be >= 0")
    return value


def require_positive_int(name: str, raw_value: str) -> int:
    """Parse and require integer > 0."""
    value = parse_int(name, raw_value)
    if value <= 0:
        fail(f"{name} must be greater than zero")
    return value


def require_enum(name: str, value: str, allowed: Sequence[str]) -> str:
    """Require a value to be in an allow-list."""
    if value not in set(allowed):
        fail(f"{name} must be one of: {', '.join(allowed)}")
    return value


def require_pattern(name: str, value: str, pattern: str, detail: str) -> str:
    """Require regex match using a stable error message."""
    if re.fullmatch(pattern, value or "") is None:
        fail(detail or f"{name} is invalid")
    return value


def load_json(path: pathlib.Path) -> Mapping[str, Any]:
    """Load JSON object from path with stable parse errors."""
    try:
        payload = json.loads(path.read_text())
    except json.JSONDecodeError as exc:
        fail(f"bundle file is not valid JSON: {exc}")

    if not isinstance(payload, dict):
        fail("bundle payload must be a JSON object")

    return payload


def write_json(path: pathlib.Path, payload: Mapping[str, Any]) -> None:
    """Write sorted/indented JSON contract output."""
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n")


def require_keys(payload: Mapping[str, Any], required_fields: Sequence[str]) -> None:
    """Validate required top-level keys."""
    for field_name in required_fields:
        if field_name not in payload:
            fail(f"missing bundle field: {field_name}")


def require_object(payload: Mapping[str, Any], field_name: str) -> Mapping[str, Any]:
    """Read and validate object field."""
    value = payload.get(field_name)
    if not isinstance(value, dict):
        fail(f"bundle field '{field_name}' must be an object")
    return value


def require_string(payload: Mapping[str, Any], field_name: str) -> str:
    """Read and validate string field."""
    value = payload.get(field_name)
    if not isinstance(value, str):
        fail(f"{field_name} must be a string")
    return value


def require_int(payload: Mapping[str, Any], field_name: str, *, min_value: int | None = None) -> int:
    """Read and validate integer field."""
    value = payload.get(field_name)
    if not isinstance(value, int):
        fail(f"{field_name} must be an integer")
    if min_value is not None and value < min_value:
        fail(f"{field_name} must be >= {min_value}")
    return value


@dataclass
class DecisionAccumulator:
    """Collect invariant failure reasons and compute final decision."""

    reasons: list[str] = field(default_factory=list)

    def reject_if(self, condition: bool, reason: str) -> None:
        """Record a reason if condition is true."""
        if condition:
            self.reasons.append(reason)

    def finalize(self, success_reason: str) -> tuple[str, list[str]]:
        """Compute GO/NO-GO with at least one reason."""
        if self.reasons:
            return "NO-GO", self.reasons
        return "GO", [success_reason]

