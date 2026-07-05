#!/usr/bin/env python
from __future__ import annotations

import re
import sys
import json
from pathlib import Path


UI_ROOT = Path("solutions/seamgrim_ui_mvp/ui")
REGISTRY = UI_ROOT / "OBSERVER_REGISTRY.json"
FORBIDDEN_PATTERN = re.compile(
    r"set_param|setParam|reset|step_one|stepOne|run_ticks|runTicks|"
    r"restore_state|restoreState|inject_ai_action|injectAiAction"
)
TARGET = Path("solutions/seamgrim_ui_mvp/ui/runtime/wasm_state_observer_client.js")


def fail(code: str, msg: str) -> int:
    print(f"[wasm-state-observer-client-capability-check] fail code={code} msg={msg}", file=sys.stderr)
    return 1


def load_observers() -> tuple[list[Path] | None, int | None]:
    if not REGISTRY.exists():
        return None, fail("E_OBSERVER_REGISTRY_MISSING", str(REGISTRY))
    try:
        data = json.loads(REGISTRY.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        return None, fail("E_OBSERVER_REGISTRY_INVALID", f"{REGISTRY}: {exc}")
    observers = data.get("observers")
    if not isinstance(observers, list) or not all(isinstance(item, str) for item in observers):
        return None, fail("E_OBSERVER_REGISTRY_INVALID", "observers must be a list of strings")
    if len(observers) != len(set(observers)):
        return None, fail("E_OBSERVER_REGISTRY_INVALID", "duplicate observer path")
    return [UI_ROOT / item for item in observers], None


def check_forbidden_tokens(path: Path) -> int | None:
    if not path.exists():
        return fail("E_OBSERVER_REGISTRY_FILE_MISSING", str(path))
    text = path.read_text(encoding="utf-8")
    match = FORBIDDEN_PATTERN.search(text)
    if match:
        line = text.count("\n", 0, match.start()) + 1
        return fail(
            "E_OBSERVER_REGISTRY_MUTATION_LEAK",
            f"{path}:{line}: forbidden token {match.group(0)!r}",
        )
    return None


def main() -> int:
    observers, error = load_observers()
    if error is not None:
        return error
    assert observers is not None
    for path in observers:
        error = check_forbidden_tokens(path)
        if error is not None:
            return error
    if TARGET not in observers:
        return fail("E_OBSERVER_REGISTRY_TARGET_MISSING", str(TARGET))
    text = TARGET.read_text(encoding="utf-8")
    required = ["getStateHash", "getStateParsed", "createWasmStateObserverClient"]
    missing = [name for name in required if name not in text]
    if missing:
        return fail("E_OBSERVER_CLIENT_EXPORT_MISSING", ",".join(missing))
    print("[wasm-state-observer-client-capability-check] ok")
    print(f"registry={REGISTRY}")
    print(f"observer_count={len(observers)}")
    for path in observers:
        print(f"observer={path.as_posix()}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
