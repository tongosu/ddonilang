#!/usr/bin/env python
from __future__ import annotations

import re
import sys
from pathlib import Path


FORBIDDEN_PATTERN = re.compile(
    r"set_param|setParam|reset|step_one|stepOne|run_ticks|runTicks|"
    r"restore_state|restoreState|inject_ai_action|injectAiAction"
)
TARGET = Path("solutions/seamgrim_ui_mvp/ui/runtime/wasm_state_observer_client.js")


def fail(code: str, msg: str) -> int:
    print(f"[wasm-state-observer-client-capability-check] fail code={code} msg={msg}", file=sys.stderr)
    return 1


def main() -> int:
    if not TARGET.exists():
        return fail("E_OBSERVER_CLIENT_MISSING", str(TARGET))
    text = TARGET.read_text(encoding="utf-8")
    match = FORBIDDEN_PATTERN.search(text)
    if match:
        line = text.count("\n", 0, match.start()) + 1
        return fail(
            "E_OBSERVER_CLIENT_MUTATION_LEAK",
            f"{TARGET}:{line}: forbidden token {match.group(0)!r}",
        )
    required = ["getStateHash", "getStateParsed", "createWasmStateObserverClient"]
    missing = [name for name in required if name not in text]
    if missing:
        return fail("E_OBSERVER_CLIENT_EXPORT_MISSING", ",".join(missing))
    print("[wasm-state-observer-client-capability-check] ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
