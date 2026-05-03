#!/usr/bin/env python
from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Iterable

TOOLS_DIR = Path(__file__).resolve().parent
ROOT = TOOLS_DIR.parents[2]

import sys

sys.path.insert(0, str(TOOLS_DIR))

from export_graph import _resolve_teul_cli_bin

PREFERRED_BUILD_DIR = Path("I:/home/urihanl/ddn/codex/build")
FALLBACK_BUILD_DIR = Path("C:/ddn/codex/build")
SOURCE_ROOT = ROOT / "docs" / "guides" / "study"
FENCE_OPEN = "```ddn"
FENCE_CLOSE = "```"
LEGACY_PREFIX_RE = re.compile(r"(?:바탕|살림)\.([0-9A-Za-z_가-힣]+)")
LEGACY_PRAGMA_RE = re.compile(r"^\s*#(?:바탕숨김|암묵살림)\b.*$", re.MULTILINE)
PLACEHOLDER_RE = re.compile(r"_{3,}|□{2,}|◻{2,}|○{2,}|빈칸|채우기")


@dataclass
class FenceBlock:
    source_path: Path
    block_index: int
    start_line: int
    end_line: int
    code: str

    @property
    def source_rel(self) -> str:
        try:
            return self.source_path.relative_to(ROOT).as_posix()
        except Exception:
            return self.source_path.as_posix()

    @property
    def doc_stem(self) -> str:
        return self.source_path.stem

    @property
    def example_id(self) -> str:
        return f"{self.doc_stem}_b{self.block_index:04d}"


def resolve_build_dir() -> Path:
    target = PREFERRED_BUILD_DIR if PREFERRED_BUILD_DIR.exists() else FALLBACK_BUILD_DIR
    target.mkdir(parents=True, exist_ok=True)
    return target


def normalize_text(text: str) -> str:
    normalized = str(text or "").replace("\r\n", "\n").replace("\r", "\n")
    if normalized.startswith("\ufeff"):
        normalized = normalized.lstrip("\ufeff")
    return normalized


def extract_fence_blocks(path: Path) -> list[FenceBlock]:
    lines = normalize_text(path.read_text(encoding="utf-8")).split("\n")
    out: list[FenceBlock] = []
    idx = 0
    block_index = 0
    while idx < len(lines):
        if lines[idx].strip() != FENCE_OPEN:
            idx += 1
            continue
        start_line = idx + 1
        idx += 1
        body: list[str] = []
        while idx < len(lines) and lines[idx].strip() != FENCE_CLOSE:
            body.append(lines[idx])
            idx += 1
        end_line = idx + 1 if idx < len(lines) else len(lines)
        block_index += 1
        out.append(
            FenceBlock(
                source_path=path,
                block_index=block_index,
                start_line=start_line + 1,
                end_line=max(start_line + 1, end_line - 1),
                code="\n".join(body).strip() + "\n",
            )
        )
        if idx < len(lines):
            idx += 1
    return out


def normalize_code_for_seamgrim(raw_code: str) -> tuple[str, list[str]]:
    code = normalize_text(raw_code).strip("\n")
    actions: list[str] = []
    removed_pragma_count = len(LEGACY_PRAGMA_RE.findall(code))
    if removed_pragma_count:
        code = LEGACY_PRAGMA_RE.sub("", code)
        actions.append(f"drop_legacy_pragma:{removed_pragma_count}")
    code, prefix_count = LEGACY_PREFIX_RE.subn(r"\1", code)
    if prefix_count:
        actions.append(f"strip_legacy_root_prefix:{prefix_count}")
    code = "\n".join(line.rstrip() for line in code.split("\n")).strip()
    if code:
        code += "\n"
    return code, actions


def has_placeholder(code: str) -> bool:
    return bool(PLACEHOLDER_RE.search(code))


def build_output_text(block: FenceBlock, code: str) -> str:
    return code.rstrip() + "\n"


def run_cmd(cmd: list[str], cwd: Path) -> tuple[bool, str]:
    result = subprocess.run(
        cmd,
        cwd=cwd,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    merged = "\n".join(part.strip() for part in [result.stdout, result.stderr] if part and part.strip()).strip()
    return result.returncode == 0, merged


def canon_check(teul_cli: Path, path: Path) -> tuple[bool, str]:
    return run_cmd([str(teul_cli), "canon", str(path), "--check"], ROOT)


def run_check(teul_cli: Path, path: Path, madi: int) -> tuple[bool, str]:
    return run_cmd([str(teul_cli), "run", str(path), "--madi", str(madi)], ROOT)


def clip(text: str, limit: int = 180) -> str:
    value = " ".join(str(text or "").split())
    return value if len(value) <= limit else value[:limit] + "..."


def iter_markdown_files(source_root: Path, include_globs: Iterable[str]) -> list[Path]:
    patterns = list(include_globs)
    if not patterns:
        patterns = ["*.md"]
    seen: set[Path] = set()
    out: list[Path] = []
    for pattern in patterns:
        for path in sorted(source_root.glob(pattern)):
            if not path.is_file() or path.suffix.lower() != ".md":
                continue
            if path.name.startswith("SEAMGRIM_"):
                continue
            if path in seen:
                continue
            seen.add(path)
            out.append(path)
    return out


def ensure_clean_dir(path: Path) -> None:
    def _onerror(func, target, exc_info):
        try:
            os.chmod(target, 0o700)
            func(target)
        except Exception:
            raise exc_info[1]

    if path.exists():
        shutil.rmtree(path, onerror=_onerror)
    path.mkdir(parents=True, exist_ok=True)


def main() -> int:
    parser = argparse.ArgumentParser(description="docs/guides/study의 ```ddn 블록을 셈그림 실습용 .ddn로 추출한다.")
    parser.add_argument("--source-root", default=str(SOURCE_ROOT))
    parser.add_argument("--output-dir")
    parser.add_argument("--include-glob", action="append", default=[], help="예: ddonirang_vol1_lesson01_v1.md")
    parser.add_argument("--max-blocks", type=int, default=0)
    parser.add_argument("--run-madi", type=int, default=1)
    parser.add_argument("--no-clean", action="store_true")
    parser.add_argument("--quiet", action="store_true")
    args = parser.parse_args()

    source_root = Path(args.source_root)
    if not source_root.is_absolute():
        source_root = (ROOT / source_root).resolve()
    if not source_root.is_dir():
        raise SystemExit(f"source root not found: {source_root}")

    build_dir = resolve_build_dir()
    output_dir = Path(args.output_dir).resolve() if args.output_dir else (build_dir / "study_practice").resolve()
    if args.no_clean:
        output_dir.mkdir(parents=True, exist_ok=True)
    else:
        ensure_clean_dir(output_dir)
    raw_root = output_dir / "raw"
    seamgrim_root = output_dir / "seamgrim"
    raw_root.mkdir(parents=True, exist_ok=True)
    seamgrim_root.mkdir(parents=True, exist_ok=True)

    teul_cli = _resolve_teul_cli_bin(ROOT)
    files = iter_markdown_files(source_root, args.include_glob)
    entries: list[dict] = []
    counters = {
        "markdown_files": len(files),
        "ddn_blocks": 0,
        "raw_canon_ok": 0,
        "normalized_canon_ok": 0,
        "practice_ready": 0,
        "placeholder_blocks": 0,
        "legacy_raw_fail_but_normalized_ok": 0,
    }

    processed_blocks = 0
    for source_path in files:
        for block in extract_fence_blocks(source_path):
            if args.max_blocks and processed_blocks >= args.max_blocks:
                break
            processed_blocks += 1
            counters["ddn_blocks"] += 1
            norm_code, actions = normalize_code_for_seamgrim(block.code)
            placeholder = has_placeholder(norm_code)
            if placeholder:
                counters["placeholder_blocks"] += 1
            raw_dir = raw_root / block.doc_stem
            seamgrim_dir = seamgrim_root / block.doc_stem
            raw_dir.mkdir(parents=True, exist_ok=True)
            seamgrim_dir.mkdir(parents=True, exist_ok=True)
            raw_path = raw_dir / f"b{block.block_index:04d}.ddn"
            seamgrim_path = seamgrim_dir / f"b{block.block_index:04d}.ddn"
            raw_path.write_text(build_output_text(block, block.code), encoding="utf-8")
            seamgrim_path.write_text(build_output_text(block, norm_code), encoding="utf-8")

            raw_ok, raw_diag = canon_check(teul_cli, raw_path)
            if raw_ok:
                counters["raw_canon_ok"] += 1
            normalized_ok, normalized_diag = canon_check(teul_cli, seamgrim_path)
            if normalized_ok:
                counters["normalized_canon_ok"] += 1
            run_ok = False
            run_diag = ""
            if not placeholder and args.run_madi > 0:
                run_ok, run_diag = run_check(teul_cli, seamgrim_path, args.run_madi)
                if run_ok:
                    counters["practice_ready"] += 1
            status = "normalize_failed"
            if placeholder:
                status = "placeholder"
            elif run_ok:
                status = "practice_ready"
            elif normalized_ok:
                status = "canon_only"
            if (not raw_ok) and normalized_ok:
                counters["legacy_raw_fail_but_normalized_ok"] += 1

            entry = {
                "id": block.example_id,
                "status": status,
                "source_md": block.source_rel,
                "block_index": block.block_index,
                "line_range": f"{block.start_line}-{block.end_line}",
                "raw_path": raw_path.relative_to(output_dir).as_posix(),
                "seamgrim_path": seamgrim_path.relative_to(output_dir).as_posix(),
                "normalization_actions": actions,
                "placeholder": placeholder,
                "raw_canon_ok": raw_ok,
                "raw_diag": clip(raw_diag),
                "normalized_canon_ok": normalized_ok,
                "normalized_diag": clip(normalized_diag),
                "run_ok": run_ok,
                "run_diag": clip(run_diag),
            }
            entries.append(entry)
            if not args.quiet and processed_blocks % 50 == 0:
                print(f"[study-practice] processed={processed_blocks} practice_ready={counters['practice_ready']}")
        if args.max_blocks and processed_blocks >= args.max_blocks:
            break

    practice_entries = [
        {
            "id": entry["id"],
            "source_md": entry["source_md"],
            "line_range": entry["line_range"],
            "path": entry["seamgrim_path"],
        }
        for entry in entries
        if entry["status"] == "practice_ready"
    ]
    report = {
        "schema": "ddn.seamgrim.study_practice_report.v1",
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "source_root": source_root.as_posix(),
        "output_dir": output_dir.as_posix(),
        "run_madi": args.run_madi,
        "counts": counters,
        "entries": entries,
    }
    inventory = {
        "schema": "ddn.seamgrim.study_practice_inventory.v1",
        "generated_at": report["generated_at"],
        "root": output_dir.as_posix(),
        "examples": practice_entries,
    }
    report_path = output_dir / "study_practice_report.detjson"
    inventory_path = output_dir / "seamgrim_inventory.detjson"
    ready_txt_path = output_dir / "practice_ready_paths.txt"
    report_path.write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")
    inventory_path.write_text(json.dumps(inventory, ensure_ascii=False, indent=2), encoding="utf-8")
    ready_txt_path.write_text(
        "\n".join(entry["path"] for entry in practice_entries) + ("\n" if practice_entries else ""),
        encoding="utf-8",
    )

    print(f"source_root={source_root}")
    print(f"output_dir={output_dir}")
    print(f"markdown_files={counters['markdown_files']}")
    print(f"ddn_blocks={counters['ddn_blocks']}")
    print(f"practice_ready={counters['practice_ready']}")
    print(f"placeholder_blocks={counters['placeholder_blocks']}")
    print(f"report={report_path}")
    print(f"inventory={inventory_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
