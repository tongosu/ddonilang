"""Private WIP browser fixture, not a ZIP/release or a language lowering."""
import hashlib
import json
from pathlib import Path
import shutil
import sys
import tempfile

repo, package, wasm, build = map(Path, sys.argv[1:5])
host_binary = Path(sys.argv[5]) if len(sys.argv)>5 else package / "bin/ddonilang.exe"
sys.path.insert(0, str(repo / "tools"))
from alpha2_completion_archive import BASE, native_workbench_payload

out = Path(tempfile.mkdtemp(prefix="vc04-source-workbench-", dir=build))
old = package / "bin" / "workbench"
payload = {BASE + p.relative_to(old).as_posix(): p.read_bytes()
           for p in old.rglob("*") if p.is_file() and p.name != "asset_manifest.detjson"}
changed = ["workbench/workbench.js", "workbench/authored_project_panel.js",
           "workbench/persistence/authored_source_save.js",
           "workbench/persistence/runtime_project_store.js", "workbench/persistence/local_fallback.js",
           "runtime/wasm_vm_runtime.js", "runtime/native_source_transport.js",
           "runtime/runtime_bridge.js", "runtime/worker_runtime_adapter.js", "runtime/runtime_worker.js"]
hashes = {}
for relative in changed:
    data = (repo / "solutions/seamgrim_ui_mvp/ui" / relative).read_bytes()
    payload[BASE + "ui/" + relative] = data
    hashes[relative] = hashlib.sha256(data).hexdigest()
for p in wasm.rglob("*"):
    if p.is_file():
        payload[BASE + "ui/wasm/" + p.relative_to(wasm).as_posix()] = p.read_bytes()
for relative, data in native_workbench_payload(payload).items():
    target = out / relative
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(data)
shutil.copyfile(host_binary, out / "bin/ddonilang.exe")
record = {"scope": "PRIVATE_WIP_NATIVE_SOURCE_CONTROLLER_NOT_PRODUCT_FIXED_OR_ZIP" if len(sys.argv)>5 else "PRIVATE_WIP_UI_WITH_FIXED_R58_STATIC_NATIVE_HOST_NOT_PRODUCT_FIXED",
          "root": str(out), "ui": hashes,
          "wasm_sha256": hashlib.sha256((wasm / "ddonirang_tool_bg.wasm").read_bytes()).hexdigest(),
          "host_sha256": hashlib.sha256((out / "bin/ddonilang.exe").read_bytes()).hexdigest()}
(out / "IDENTITY.json").write_text(json.dumps(record, ensure_ascii=False, indent=2), encoding="utf-8")
print(json.dumps(record, ensure_ascii=True))
