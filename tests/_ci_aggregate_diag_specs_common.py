from __future__ import annotations

import re


def build_identity_field_specs(*keys: str) -> tuple[tuple[str, str], ...]:
    return tuple((key, key) for key in keys)


def build_default_snapshot_from_typed_specs(
    field_specs: tuple[tuple[str, str, str, str], ...],
) -> dict[str, str]:
    return {
        output_key: str(default_value)
        for output_key, _, __, default_value in field_specs
    }


CONTROL_EXPOSURE_VIOLATION_RE = re.compile(
    r"(?P<kind>[a-z_]+):(?P<file>[^:\s,]+\.ddn):(?P<name>[A-Za-z0-9_가-힣]+)",
    re.UNICODE,
)
CONTROL_EXPOSURE_MORE_RE = re.compile(r"\.\.\.\s*\((?P<count>\d+)\s+more\)", re.UNICODE)
