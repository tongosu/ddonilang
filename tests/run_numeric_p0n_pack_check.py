#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path


PACKS = [
    "numeric_exact_numbers_smoke_v1",
    "numeric_fraction_normalize_v1",
    "numeric_promotion_rules_v1",
    "numeric_factor_form_v1",
    "numeric_display_policy_v1",
    "numeric_sized_variants_v1",
    "numeric_type_pin_vs_constructor_v1",
    "numeric_maegim_binding_v1",
    "numeric_type_alias_korean_v1",
    "numeric_exact_universe_v1",
    "numeric_factor_kernel_unbounded_v1",
    "numeric_factor_job_resume_v1",
]


def load_jsonl(path: Path) -> list[dict[str, object]]:
    rows = []
    for line in path.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if line:
            rows.append(json.loads(line))
    return rows


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    missing: list[str] = []
    for pack in PACKS:
        pack_dir = root / "pack" / pack
        golden = pack_dir / "golden.jsonl"
        readme = pack_dir / "README.md"
        if not pack_dir.is_dir():
            missing.append(f"{pack}: pack directory missing")
            continue
        if not golden.is_file():
            missing.append(f"{pack}: golden.jsonl missing")
            continue
        if not readme.is_file():
            missing.append(f"{pack}: README.md missing")
        rows = load_jsonl(golden)
        if not rows:
            missing.append(f"{pack}: no golden rows")
        for idx, row in enumerate(rows, start=1):
            cmd = row.get("cmd")
            if not isinstance(cmd, list) or not cmd:
                missing.append(f"{pack}: row {idx} cmd missing")
            if row.get("exit_code", 0) == 0 and "stdout_path" not in row:
                missing.append(f"{pack}: row {idx} stdout_path missing")
    if missing:
        for item in missing:
            print(f"numeric_p0n_pack_check fail: {item}")
        return 1

    for pack in PACKS:
        proc = subprocess.run(
            ["python", "tests/run_pack_golden.py", pack],
            cwd=root,
            text=True,
            encoding="utf-8",
            errors="replace",
            capture_output=True,
        )
        if proc.returncode != 0:
            print(proc.stdout)
            print(proc.stderr)
            print(f"numeric_p0n_pack_check fail: {pack}")
            return proc.returncode

    print("numeric_p0n_pack_check ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
