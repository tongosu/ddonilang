#!/usr/bin/env python
from __future__ import annotations

import json
import re
import subprocess
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
SCRIPT = ROOT / "solutions" / "seamgrim_ui_mvp" / "tools" / "repair_study_guide_markdown_ddn.py"


def main() -> int:
    with tempfile.TemporaryDirectory(prefix="study_markdown_repair_") as tmp:
        tmp_root = Path(tmp)
        work_path = tmp_root / "ddonirang_vol1_lesson01_v1.md"
        work_path.write_text(
            """# sample

설명에서 `바탕.인사`를 언급한다.

```ddn
바탕.인사 <- "안녕".
(값) (1, 2, 3) 차림에 대해: {
  바탕.인사 보여주기.
}
```
""",
            encoding="utf-8",
        )
        report_path = tmp_root / "repair_report.detjson"
        cmd = [
            "python",
            str(SCRIPT),
            "--source-root",
            str(tmp_root),
            "--apply-safe",
            "--json-out",
            str(report_path),
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

        text = work_path.read_text(encoding="utf-8")
        ddn_blocks = re.findall(r"```ddn\n(.*?)\n```", text, flags=re.DOTALL)
        if not ddn_blocks:
            print("expected at least one ddn block in repaired sample file")
            return 1
        if any("바탕." in block or "살림." in block for block in ddn_blocks):
            print("expected no legacy root token inside repaired ddn blocks")
            return 1
        if "에 대해: {" in text:
            print("expected deprecated block header colon to be removed")
            return 1

        report = json.loads(report_path.read_text(encoding="utf-8"))
        counts = report.get("counts", {})
        if int(counts.get("changed_files", 0)) <= 0:
            print("expected changed_files > 0")
            return 1
        if int(counts.get("prefix_replacements", 0)) <= 0:
            print("expected prefix_replacements > 0")
            return 1

        print(json.dumps({"ok": True, "counts": counts}, ensure_ascii=False))
        return 0


if __name__ == "__main__":
    raise SystemExit(main())
