#!/usr/bin/env python
from __future__ import annotations

import runpy
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
runpy.run_path(str(ROOT / "tools" / "extract_llm_corpus_v1.py"), run_name="__main__")
