from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACKS = [
    ROOT / "pack" / "lang_seum_algebraic_v1",
    ROOT / "pack" / "lang_seum_linear_v1",
]
REQUIRED_FIELDS = [
    "schema",
    "source_hash",
    "stage",
    "method",
    "relations",
    "unknowns",
    "lowering",
    "diagnostics",
    "state_hash_inputs",
]


def fail(message: str) -> None:
    print(f"declarative_closure_g1_red_check: FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def read_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except FileNotFoundError:
        fail(f"missing required file: {path.relative_to(ROOT)}")


def read_json(path: Path) -> dict:
    try:
        return json.loads(read_text(path))
    except json.JSONDecodeError as exc:
        fail(f"invalid JSON {path.relative_to(ROOT)}: {exc}")


def run(cmd: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        check=False,
    )


def require_contains(path: Path, needles: list[str]) -> None:
    text = read_text(path)
    missing = [needle for needle in needles if needle not in text]
    if missing:
        fail(f"{path.relative_to(ROOT)} missing tokens: {missing}")


def check_contract(pack: Path) -> dict:
    contract = read_json(pack / "contract.detjson")
    if contract.get("schema") != "ddn.pack.contract.v1":
        fail(f"{pack.name} contract schema mismatch")
    if contract.get("pack") != pack.name:
        fail(f"{pack.name} contract pack mismatch")
    if contract.get("goal") != "G1":
        fail(f"{pack.name} goal must be G1")
    for flag in ["runtime_claim", "ssot_edit_claim"]:
        if contract.get(flag) is not False:
            fail(f"{pack.name} {flag} must be false in red pack")
    if not isinstance(contract.get("product_code_change"), bool):
        fail(f"{pack.name} product_code_change must be boolean")
    if contract.get("artifact_schema") != "ddn.solveplan.v1":
        fail(f"{pack.name} artifact schema mismatch")
    if contract.get("required_fields") != REQUIRED_FIELDS:
        fail(f"{pack.name} required fields mismatch")
    if contract.get("state_hash_integration") != "manifest_field_only_pending_tonni_confirmation":
        fail(f"{pack.name} state_hash integration boundary mismatch")
    return contract


def check_artifact(path: Path, expected_stage: str | None = None) -> dict:
    artifact = read_json(path)
    for field in REQUIRED_FIELDS:
        if field not in artifact:
            fail(f"{path.relative_to(ROOT)} missing required field {field}")
    if artifact["schema"] != "ddn.solveplan.v1":
        fail(f"{path.relative_to(ROOT)} schema mismatch")
    if expected_stage and artifact["stage"] != expected_stage:
        fail(f"{path.relative_to(ROOT)} stage expected {expected_stage}, got {artifact['stage']}")
    state_hash_inputs = artifact["state_hash_inputs"]
    if not isinstance(state_hash_inputs, dict):
        fail(f"{path.relative_to(ROOT)} state_hash_inputs must be object")
    include = state_hash_inputs.get("include")
    exclude = state_hash_inputs.get("exclude")
    if not isinstance(include, list) or "diagnostics" in include:
        fail(f"{path.relative_to(ROOT)} include must be list and exclude diagnostics")
    if not isinstance(exclude, list) or "diagnostics" not in exclude or "execution_results" not in exclude:
        fail(f"{path.relative_to(ROOT)} exclude must contain diagnostics and execution_results")
    return artifact


def check_s0(pack: Path) -> None:
    contract = check_contract(pack)
    if contract.get("stage") != "S0":
        fail("S0 contract stage mismatch")
    artifact = check_artifact(pack / contract["expected_artifact"], "S0")
    if artifact["method"] != "explicit_algebra_causalized_v1":
        fail("S0 method mismatch")
    if artifact["lowering"].get("causal_order") != ["r1", "r2"]:
        fail("S0 causal order must follow dependency then source order")
    require_contains(pack / contract["source_fixture"], ["세움{", "x =:= 2", "y =:= x + 3"])


def check_s1(pack: Path) -> None:
    contract = check_contract(pack)
    if contract.get("stage") != "S1":
        fail("S1 contract stage mismatch")
    boundary = contract.get("boundary") or {}
    if boundary != {"accept_unknown_count": 8, "reject_unknown_count": 9, "reject_code": "E_SEUM_UNSUPPORTED"}:
        fail(f"S1 boundary mismatch: {boundary!r}")
    if contract.get("tie_break") != "source_declaration_order":
        fail("S1 tie-break must be source_declaration_order")
    artifacts = [check_artifact(pack / rel) for rel in contract["expected_artifacts"]]
    n8, n9, tie = artifacts
    if n8["stage"] != "S1" or len(n8["unknowns"]) != 8:
        fail("S1 N=8 artifact must be accepted as S1 with eight unknowns")
    if n9["stage"] != "S3" or len(n9["unknowns"]) != 9:
        fail("S1 N=9 artifact must be rejected as S3 with nine unknowns")
    diag_codes = [diag.get("code") for diag in n9.get("diagnostics", [])]
    if diag_codes != ["E_SEUM_UNSUPPORTED"]:
        fail(f"S1 N=9 diagnostic mismatch: {diag_codes!r}")
    if tie["unknowns"] != ["b", "a", "c"] or tie["lowering"].get("causal_order") != ["r1", "r2", "r3"]:
        fail("S1 tie-break fixture must preserve source declaration order")
    for source in contract["source_fixtures"]:
        require_contains(pack / source, ["세움{", "=:="])


def check_docs() -> None:
    require_contains(
        ROOT / "docs" / "context" / "proposals" / "PROPOSAL_DECLARATIVE_CLOSURE_11AXIS_SSOT_PATCH_V2_20260709.md",
        [
            "S0 명시적 대수 + 결정적 인과화",
            "state_hash_inputs",
            "`채비 {}` 본문 안의 `세움 {}`",
            "`막음`, 동사는 `막기`",
        ],
    )
    require_contains(
        ROOT / "docs" / "reports" / "impl" / "DECLARATIVE_CLOSURE_G1_G3_IMPLEMENTATION_GAP_V1_20260709.md",
        ["[DC-G1-RED]", "[DC-G1-CORE]", "[DC-G2-PHASE]", "[DC-G3-S2]"],
    )


def check_pack_golden() -> None:
    proc = run(["python", "tests/run_pack_golden.py", "lang_seum_algebraic_v1", "lang_seum_linear_v1"])
    if proc.returncode != 0:
        fail(f"run_pack_golden failed:\n{proc.stdout}")


def check_docs_ssot_clean() -> None:
    proc = run(["git", "status", "--short", "--", "docs/ssot"])
    if proc.returncode != 0:
        fail(f"git status docs/ssot failed:\n{proc.stdout}")
    lines = [line for line in proc.stdout.splitlines() if line.strip() and not line.startswith("warning:")]
    if lines:
        fail(f"docs/ssot changed:\n{proc.stdout}")


def main() -> None:
    for pack in PACKS:
        for name in ["README.md", "input.ddn", "golden.jsonl", "contract.detjson"]:
            if not (pack / name).exists():
                fail(f"missing required file: {(pack / name).relative_to(ROOT)}")
    check_s0(PACKS[0])
    check_s1(PACKS[1])
    check_docs()
    check_pack_golden()
    check_docs_ssot_clean()
    print("declarative_closure_g1_red_check: ok")


if __name__ == "__main__":
    main()
