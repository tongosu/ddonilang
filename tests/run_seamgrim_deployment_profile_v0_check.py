#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_deployment_profile_v0_v1"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def run(cmd: list[str], *, timeout: int = 240) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        timeout=timeout,
    )


def check_contract() -> int:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        ROOT / "tests" / "seamgrim_deployment_profile_v0_runner.mjs",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_DEPLOYMENT_PROFILE_MISSING", repr(missing))
    contract = json.loads(read(PACK / "contract.detjson"))
    expected = {
        "schema": "ddn.pack.contract.v1",
        "pack": "seamgrim_deployment_profile_v0_v1",
        "kind": "seamgrim_deployment_profile_v0_product_vertical",
        "product_behavior_claim": True,
        "parser_change": False,
        "runtime_semantics_change": False,
        "state_hash_formula_change": False,
        "docs_ssot_change": False,
        "signature_claim": False,
        "browser_runner": "tests/seamgrim_deployment_profile_v0_runner.mjs",
    }
    for key, value in expected.items():
        if contract.get(key) != value:
            return fail("E_SEAMGRIM_DEPLOYMENT_PROFILE_CONTRACT", f"{key}={contract.get(key)!r}")
    covers = set(contract.get("covers") or [])
    required_covers = {
        "publish_profile_edit",
        "package_hash_includes_deployment_profile",
        "paste_reopen_roundtrip",
        "run_screen_readonly_profile",
        "tamper_detection_korean_diag",
    }
    if not required_covers.issubset(covers):
        return fail("E_SEAMGRIM_DEPLOYMENT_PROFILE_COVERS", repr(sorted(covers)))
    return 0


def check_sources() -> int:
    html = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html")
    package_js = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "studio_local_share_package.js")
    browse = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "browse.js")
    run_js = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "run.js")
    for token in [
        "data-deployment-profile-editor",
        "deployment-profile-preset",
        "deployment-profile-control-min",
        "deployment-profile-control-max",
        "deployment-profile-control-step",
        "data-deployment-profile-hash",
    ]:
        if token not in html:
            return fail("E_SEAMGRIM_DEPLOYMENT_PROFILE_HTML", token)
    for token in [
        "buildDeploymentProfileV0",
        "profile_hash",
        "deployment_profile_hash",
        "package_hash_inputs",
        "studio_local_package_deployment_profile_hash_mismatch",
        "studio_local_package_hash_mismatch",
    ]:
        if token not in package_js:
            return fail("E_SEAMGRIM_DEPLOYMENT_PROFILE_PACKAGE_JS", token)
    for token in ["readDeploymentProfileDraft", "updateDeploymentProfileHashPreview", "deploymentProfile: this.readDeploymentProfileDraft()"]:
        if token not in browse:
            return fail("E_SEAMGRIM_DEPLOYMENT_PROFILE_BROWSE", token)
    for token in ["renderLocalPackageDeploymentProfileHtml", "data-run-deployment-profile-readonly", "deploymentProfileReadonly", "localPackageHash"]:
        if token not in run_js:
            return fail("E_SEAMGRIM_DEPLOYMENT_PROFILE_RUN", token)
    blocked = ["data-classroom-mode-switch", "applyClassroomModeSwitch", "__STUDIO_CLASSROOM_MODE_SWITCH__"]
    for token in blocked:
      if token in run_js:
          return fail("E_SEAMGRIM_DEPLOYMENT_PROFILE_ROLE_SWITCH", token)
    return 0


def run_gate(cmd: list[str], code: str, timeout: int) -> int:
    proc = run(cmd, timeout=timeout)
    if proc.returncode != 0:
        return fail(code, f"{' '.join(cmd)}: {(proc.stdout or '').strip()}")
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_DEPLOYMENT_PROFILE_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_SEAMGRIM_DEPLOYMENT_PROFILE_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    for check in (check_contract, check_sources):
        rc = check()
        if rc:
            return rc
    rc = run_gate(["node", "tests/seamgrim_deployment_profile_v0_runner.mjs"], "E_SEAMGRIM_DEPLOYMENT_PROFILE_BROWSER", 240)
    if rc:
        return rc
    rc = run_gate(["python", "tests/run_pack_golden.py", "seamgrim_deployment_profile_v0_v1"], "E_SEAMGRIM_DEPLOYMENT_PROFILE_PACK", 120)
    if rc:
        return rc
    rc = check_docs_ssot_clean()
    if rc:
        return rc
    print("[seamgrim-deployment-profile-v0-v1] OK behavior-closed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
