#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
SCRIPT = ROOT / "solutions" / "seamgrim_ui_mvp" / "tools" / "extract_study_guide_ddn.py"


def resolve_build_dir() -> Path:
    primary = Path("I:/home/urihanl/ddn/codex/build")
    fallback = Path("C:/ddn/codex/build")
    target = primary if primary.exists() else fallback
    target.mkdir(parents=True, exist_ok=True)
    return target


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="study_practice_extract_") as tmp:
        tmp_root = Path(tmp)
        source_root = tmp_root / "study"
        source_root.mkdir(parents=True, exist_ok=True)
        (source_root / "ddonirang_vol1_lesson01_v1.md").write_text(
            """# sample

```ddn
바탕.인사 <- "안녕".
바탕.인사 보여주기.
```

```ddn
바탕.인사 보여주기.
```
""",
            encoding="utf-8",
        )
        out_dir = resolve_build_dir() / "tmp" / "study_practice_extract_check"
        if out_dir.exists():
            import shutil

            shutil.rmtree(out_dir)
        cmd = [
            "python",
            str(SCRIPT),
            "--source-root",
            str(source_root),
            "--output-dir",
            str(out_dir),
            "--quiet",
        ]
        result = subprocess.run(
            cmd,
            cwd=ROOT,
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
        if result.returncode != 0:
            print(result.stdout)
            print(result.stderr)
            return result.returncode

        report_path = out_dir / "study_practice_report.detjson"
        inventory_path = out_dir / "seamgrim_inventory.detjson"
        if not report_path.exists() or not inventory_path.exists():
            print("missing generated report/inventory")
            return 1

        report = json.loads(report_path.read_text(encoding="utf-8"))
        counts = report.get("counts", {})
        if int(counts.get("ddn_blocks", 0)) <= 0:
            print("ddn_blocks count must be > 0")
            return 1
        if int(counts.get("practice_ready", 0)) <= 0:
            print("practice_ready count must be > 0")
            return 1

        entries = report.get("entries", [])
        migrated = [
            entry
            for entry in entries
            if not entry.get("raw_canon_ok") and entry.get("normalized_canon_ok")
        ]
        if not migrated:
            print("expected at least one raw-fail normalized-ok migrated entry")
            return 1
        practice_ready = [entry for entry in entries if entry.get("status") == "practice_ready"]
        if not practice_ready:
            print("expected at least one practice_ready entry")
            return 1

        inventory = json.loads(inventory_path.read_text(encoding="utf-8"))
        examples = inventory.get("examples", [])
        if not examples:
            print("inventory examples must not be empty")
            return 1

        print(
            json.dumps(
                {
                    "ok": True,
                    "ddn_blocks": counts.get("ddn_blocks"),
                    "practice_ready": counts.get("practice_ready"),
                    "migrated_entries": len(migrated),
                },
                ensure_ascii=False,
            )
        )
        return 0


if __name__ == "__main__":
    raise SystemExit(main())
