#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_malblock_0_v1"


def fail(message: str) -> int:
    print(f"[seamgrim-malblock-design] fail: {message}", file=sys.stderr)
    return 1


def sort_json(value):
    if isinstance(value, list):
        return [sort_json(item) for item in value]
    if isinstance(value, dict):
        return {key: sort_json(value[key]) for key in sorted(value)}
    return value


def format_json(value) -> str:
    return json.dumps(sort_json(value), ensure_ascii=False, indent=2) + "\n"


def load_contract() -> dict:
    payload = json.loads((PACK / "fixtures" / "contract.detjson").read_text(encoding="utf-8"))
    if payload.get("schema") != "ddn.seamgrim_malblock_design_contract.v1":
        raise RuntimeError("contract.detjson schema mismatch")
    return payload


def read_palette_kinds() -> list[str]:
    script = r"""
const { pathToFileURL } = require("url");
const path = require("path");

async function main() {
  const root = process.cwd();
  const mod = await import(pathToFileURL(path.join(root, "solutions", "seamgrim_ui_mvp", "ui", "block_editor", "seamgrim_palette.js")).href);
  const kinds = [];
  function walk(groups) {
    for (const item of Array.isArray(groups) ? groups : []) {
      if (item?.block?.kind) kinds.push(String(item.block.kind));
      if (item?.kind) kinds.push(String(item.kind));
      if (item?.blocks) walk(item.blocks);
      if (item?.children) walk(item.children);
    }
  }
  walk(mod.SEAMGRIM_PALETTE?.categories);
  console.log(JSON.stringify([...new Set(kinds)].sort()));
}

main().catch((err) => {
  console.error(String((err && err.stack) || err));
  process.exit(1);
});
"""
    proc = subprocess.run(
        ["node", "-"],
        cwd=ROOT,
        input=script,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        timeout=60,
    )
    if proc.returncode != 0:
        raise RuntimeError((proc.stderr or proc.stdout or "palette read failed").strip())
    return [str(item) for item in json.loads(proc.stdout)]


def build_report(contract: dict) -> dict:
    if contract.get("canonical_truth") != "ddn_source":
        raise RuntimeError("canonical_truth must be ddn_source")
    if contract.get("block_representation") != "editor_projection":
        raise RuntimeError("block_representation must be editor_projection")

    required_boundaries = sorted(str(item) for item in contract.get("required_boundaries", []))
    for required in (
        "block_json_is_not_canonical",
        "unsupported_ddn_must_be_preserved_as_raw_or_text_island",
        "generated_ddn_must_pass_canon_before_runtime_claim",
        "palette_must_not_drop_source_text",
    ):
        if required not in required_boundaries:
            raise RuntimeError(f"required boundary missing: {required}")

    palette_kinds = set(read_palette_kinds())
    required_kinds = sorted(str(item) for item in contract.get("required_palette_kinds", []))
    missing_kinds = [kind for kind in required_kinds if kind not in palette_kinds]
    if missing_kinds:
        raise RuntimeError(f"palette kinds missing: {', '.join(missing_kinds)}")

    existing_checks = []
    for rel in contract.get("required_existing_checks", []):
        path = str(rel)
        exists = (ROOT / path).exists()
        if not exists:
            raise RuntimeError(f"required check missing: {path}")
        existing_checks.append({"path": path, "exists": exists})

    return {
        "schema": "ddn.seamgrim_malblock_design_report.v1",
        "canonical_truth": "ddn_source",
        "representation": "editor_projection",
        "boundaries": required_boundaries,
        "palette": {
            "required_kinds": required_kinds,
            "missing_kinds": missing_kinds,
        },
        "existing_checks": existing_checks,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="ROADMAP_V2 라-0 말블록 설계 계약 checker")
    parser.add_argument("--update", action="store_true", help="expected report 갱신")
    args = parser.parse_args()

    try:
        report = build_report(load_contract())
        expected_path = PACK / "expected" / "malblock_design.detjson"
        actual_text = format_json(report)
        if args.update:
            expected_path.parent.mkdir(parents=True, exist_ok=True)
            expected_path.write_text(actual_text, encoding="utf-8")
            print(f"[seamgrim-malblock-design] updated {expected_path.relative_to(ROOT)}")
            return 0
        expected_text = expected_path.read_text(encoding="utf-8")
        if expected_text != actual_text:
            raise RuntimeError(f"expected mismatch: {expected_path.relative_to(ROOT)}")
    except Exception as exc:
        return fail(str(exc))

    print("[seamgrim-malblock-design] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
