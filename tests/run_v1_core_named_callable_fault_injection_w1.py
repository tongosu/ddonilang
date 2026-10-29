from __future__ import annotations

import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "v1_core_named_callable_evidence_rebase_w1"
NATIVE = ROOT / "tests" / "run_v1_core_named_callable_evidence_rebase_w1.py"
WASM = ROOT / "tests" / "run_v1_core_named_callable_wasm_parity_w1.mjs"


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes().replace(b"\r\n", b"\n")).hexdigest()


def tracked_identity() -> dict[str, str]:
    paths = [NATIVE, WASM, Path(__file__), *sorted(path for path in PACK.rglob("*") if path.is_file())]
    return {str(path.relative_to(ROOT)).replace("\\", "/"): sha256(path) for path in paths}


def run(script: Path, pack: Path) -> subprocess.CompletedProcess[str]:
    env = os.environ.copy()
    env["DDN_VC09_PACK_ROOT"] = str(pack)
    env["PYTHONUTF8"] = "1"
    command = [sys.executable, str(script)] if script.suffix == ".py" else ["node", str(script)]
    return subprocess.run(
        command,
        cwd=ROOT,
        env=env,
        text=True,
        encoding="utf-8",
        capture_output=True,
    )


def require_red(mutation_id: str, completed: subprocess.CompletedProcess[str], marker: str) -> None:
    output = completed.stdout + completed.stderr
    if completed.returncode == 0:
        raise SystemExit(f"{mutation_id}: mutation unexpectedly stayed GREEN")
    if marker not in output:
        raise SystemExit(f"{mutation_id}: RED missing marker={marker}\n{output}")
    print(f"{mutation_id}: RED exit={completed.returncode} signature={marker} -> temp-copy restore")


def main() -> None:
    before = tracked_identity()
    print(f"VC09_FAULT_RUNNER_SHA256={sha256(Path(__file__))}")
    with tempfile.TemporaryDirectory(prefix="ddn-vc09-fault-") as temp_text:
        temp = Path(temp_text)

        omission = temp / "case_omission"
        shutil.copytree(PACK, omission)
        manifest_path = omission / "manifest.detjson"
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        manifest["cases"] = manifest["cases"][:-1]
        manifest_path.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        require_red("CASE_OMISSION", run(NATIVE, omission), "VC09_CASE_COUNT_MISMATCH")

        order = temp / "case_order_tamper"
        shutil.copytree(PACK, order)
        order_manifest = order / "manifest.detjson"
        order_data = json.loads(order_manifest.read_text(encoding="utf-8"))
        order_data["cases"][0], order_data["cases"][1] = order_data["cases"][1], order_data["cases"][0]
        order_manifest.write_text(json.dumps(order_data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        require_red("CASE_ORDER_TAMPER", run(NATIVE, order), "VC09_CASE_ID_OR_ORDER_MISMATCH")

        fixture = temp / "fixture_tamper"
        shutil.copytree(PACK, fixture)
        fixture_path = fixture / "fixtures" / "nc04_arity_mismatch.ddn"
        fixture_path.write_text(fixture_path.read_text(encoding="utf-8") + "// tamper\n", encoding="utf-8")
        require_red("FIXTURE_HASH_TAMPER", run(NATIVE, fixture), "VC09_FIXTURE_HASH_MISMATCH")

        receipt = temp / "receipt_tamper"
        shutil.copytree(PACK, receipt)
        receipt_path = receipt / "expected" / "nc01.detjson"
        receipt_path.write_text(receipt_path.read_text(encoding="utf-8") + " ", encoding="utf-8")
        require_red("RECEIPT_TAMPER", run(NATIVE, receipt), "VC09_RECEIPT_HASH_MISMATCH")

        outcome = temp / "outcome_tamper"
        shutil.copytree(PACK, outcome)
        outcome_manifest = outcome / "manifest.detjson"
        outcome_data = json.loads(outcome_manifest.read_text(encoding="utf-8"))
        outcome_data["cases"][0]["kind"] = "failure"
        outcome_manifest.write_text(json.dumps(outcome_data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        require_red("OUTCOME_TAMPER", run(NATIVE, outcome), "VC09_OUTCOME_MISMATCH")

        profile = temp / "profile_tamper"
        shutil.copytree(PACK, profile)
        profile_manifest = profile / "manifest.detjson"
        profile_data = json.loads(profile_manifest.read_text(encoding="utf-8"))
        profile_data["source_profile_identity"] = "strict"
        profile_manifest.write_text(json.dumps(profile_data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        require_red("PROFILE_IDENTITY_TAMPER", run(NATIVE, profile), "VC09_PROFILE_IDENTITY_MISMATCH")

    native_green = run(NATIVE, PACK)
    if native_green.returncode != 0 or "VC09_NATIVE_EVIDENCE_PASS" not in native_green.stdout:
        raise SystemExit(f"native restore GREEN failed\n{native_green.stdout}{native_green.stderr}")
    wasm_green = run(WASM, PACK)
    if wasm_green.returncode != 0 or "VC09_WASM_PARITY_PASS" not in wasm_green.stdout:
        raise SystemExit(f"WASM restore GREEN failed\n{wasm_green.stdout}{wasm_green.stderr}")
    after = tracked_identity()
    if after != before:
        raise SystemExit("VC09_TRACKED_IDENTITY_CHANGED_AFTER_FAULTS")
    print(
        "VC09_FAULT_INJECTION_PASS mutations=6 exact_restore=6 native_green=1 wasm_green=1 residue=0 "
        f"tracked_identity_sha256={hashlib.sha256(json.dumps(after, sort_keys=True).encode()).hexdigest()}"
    )


if __name__ == "__main__":
    main()
