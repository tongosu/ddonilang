#!/usr/bin/env python
from __future__ import annotations

import json
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "learning_spine_index_v1"
INDEX = ROOT / "solutions" / "seamgrim_ui_mvp" / "learning_spine_index.detjson"
LESSON_INDEX = ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "index.json"

EXPECTED_TITLES = [
    "값·식·단위",
    "상태와 마디",
    "조건·반복·사건",
    "순수 함수·함수값·자료형·차림",
    "모듬·가지·테스트·진단",
    "관계·모델·논리·증명",
    "AI 협업과 뜻 단위 diff",
    "Python·Rust·JS·Modelica 대응 교과",
]
EXPECTED_STATUSES = ["landed", "landed", "landed", "mixed", "mixed", "mixed", "mixed", "gap_open"]
FROZEN_TAGS = {
    "declarative_modeling",
    "ai_audit",
    "multilang_curriculum",
    "manim",
    "nurigym_game",
    "freeze",
    "governance",
}
ALLOWED_OVERLAYS = ["learning_spine_index"]
ALLOWED_STATUS = {"landed", "mixed", "gap_open"}
ALLOWED_SOURCE_KINDS = {"textbook", "guide", "curriculum_index", "implementation_report", "proposal"}
ALLOWED_EVIDENCE_STATUS = {"verified_by_stage_gate", "reference_only"}
EXPECTED_LANDED_GATES = {
    "LS01": [["python", "tests/run_seamgrim_current_line_three_worlds_check.py"]],
    "LS02": [["python", "tests/run_seamgrim_first_delight_projectile_check.py"]],
    "LS03": [["python", "tests/run_onmungchi_10_worlds_pilot_check.py"]],
}


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def load(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def run(cmd: list[str], timeout: int = 300) -> subprocess.CompletedProcess[str]:
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


def check_required_files() -> int:
    required = [
        INDEX,
        LESSON_INDEX,
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "learning_spine_index.js",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css",
        ROOT / "tests" / "seamgrim_learning_spine_index_runner.mjs",
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    return fail("E_LEARNING_SPINE_MISSING", repr(missing)) if missing else 0


def check_contract() -> int:
    contract = load(PACK / "contract.detjson")
    if contract.get("artifact_schema") != "ddn.learning_spine_index.v1" or contract.get("stage_count") != 8:
        return fail("E_LEARNING_SPINE_CONTRACT", repr(contract))
    for key in ("new_curriculum", "new_roadmap", "completion_claim", "runtime_claim", "docs_ssot_change"):
        if contract.get(key) is not False:
            return fail("E_LEARNING_SPINE_OVERCLAIM", f"{key}={contract.get(key)!r}")
    if contract.get("owner_decision_required_for_promotion") is not True:
        return fail("E_LEARNING_SPINE_OWNER_BOUNDARY", repr(contract))
    if contract.get("state_axes_required") != ["source", "reflection", "implementation", "owner_decision"]:
        return fail("E_LEARNING_SPINE_STATE_AXIS_CONTRACT", repr(contract))
    for key in ("source_kind_required", "landed_evidence_gate_required", "landed_world_execution_required", "keyboard_focus_restore_required"):
        if contract.get(key) is not True:
            return fail("E_LEARNING_SPINE_BEHAVIOR_CONTRACT", f"{key}={contract.get(key)!r}")
    if contract.get("allowed_overlays") != ALLOWED_OVERLAYS or contract.get("frozen_tag_count") != len(FROZEN_TAGS):
        return fail("E_LEARNING_SPINE_VOCAB_CONTRACT", repr(contract))
    return 0


def check_manifest() -> int:
    data = load(INDEX)
    if data.get("schema") != "ddn.learning_spine_index.v1":
        return fail("E_LEARNING_SPINE_SCHEMA", repr(data.get("schema")))
    policy = data.get("policy", {})
    expected_policy = {
        "new_curriculum": False,
        "new_roadmap": False,
        "completion_claim": False,
        "connector_index_only": True,
        "ssot_change": False,
        "owner_decision_required_for_promotion": True,
    }
    for key, expected in expected_policy.items():
        if policy.get(key) is not expected:
            return fail("E_LEARNING_SPINE_POLICY", f"{key}={policy.get(key)!r}")
    if data.get("overlays") != ALLOWED_OVERLAYS:
        return fail("E_LEARNING_SPINE_OVERLAY", repr(data.get("overlays")))
    if set(data.get("frozen_tags", [])) != FROZEN_TAGS or len(data.get("frozen_tags", [])) != len(FROZEN_TAGS):
        return fail("E_LEARNING_SPINE_TAG_FREEZE", repr(data.get("frozen_tags")))

    stages = data.get("stages", [])
    if len(stages) != 8:
        return fail("E_LEARNING_SPINE_STAGE_COUNT", str(len(stages)))
    if [row.get("order") for row in stages] != list(range(1, 9)):
        return fail("E_LEARNING_SPINE_ORDER", repr([row.get("order") for row in stages]))
    if [row.get("id") for row in stages] != [f"LS{i:02d}" for i in range(1, 9)]:
        return fail("E_LEARNING_SPINE_IDS", repr([row.get("id") for row in stages]))
    if [row.get("title") for row in stages] != EXPECTED_TITLES:
        return fail("E_LEARNING_SPINE_TITLES", repr([row.get("title") for row in stages]))
    if [row.get("status") for row in stages] != EXPECTED_STATUSES:
        return fail("E_LEARNING_SPINE_STATUS", repr([row.get("status") for row in stages]))

    known_lessons = {row.get("id") for row in load(LESSON_INDEX).get("lessons", [])}
    for index, stage in enumerate(stages):
        stage_id = stage.get("id")
        if stage.get("status") not in ALLOWED_STATUS:
            return fail("E_LEARNING_SPINE_STATUS_VOCAB", f"{stage_id}: {stage.get('status')!r}")
        expected_next = stages[index + 1].get("id") if index < 7 else None
        if stage.get("next_stage_id") != expected_next:
            return fail("E_LEARNING_SPINE_NEXT", f"{stage_id}: {stage.get('next_stage_id')!r}")
        tags = stage.get("tags", [])
        if len(tags) != len(set(tags)) or not set(tags).issubset(FROZEN_TAGS):
            return fail("E_LEARNING_SPINE_STAGE_TAG", f"{stage_id}: {tags!r}")
        if not stage.get("outcome") or not stage.get("exposure_levels") or not stage.get("ganada_cells"):
            return fail("E_LEARNING_SPINE_STAGE_META", str(stage_id))
        if not stage.get("source_refs") or not stage.get("worlds") or not stage.get("pack_refs"):
            return fail("E_LEARNING_SPINE_COVERAGE", str(stage_id))
        if stage.get("status") != "landed" and not stage.get("open_items"):
            return fail("E_LEARNING_SPINE_OPEN_ITEMS", str(stage_id))

        axes = stage.get("state_axes", {})
        expected_axes = {
            "source": "referenced",
            "reflection": "connector_index_only_not_ssot",
            "implementation": stage.get("status"),
            "owner_decision": "pending",
        }
        if axes != expected_axes:
            return fail("E_LEARNING_SPINE_STATE_AXES", f"{stage_id}: {axes!r}")
        expected_gates = EXPECTED_LANDED_GATES.get(stage_id, [])
        if stage.get("evidence_gates") != expected_gates:
            return fail("E_LEARNING_SPINE_EVIDENCE_GATES", f"{stage_id}: {stage.get('evidence_gates')!r}")

        for row in stage.get("source_refs", []):
            path = str(row.get("path", ""))
            if not path or not (ROOT / path).is_file():
                return fail("E_LEARNING_SPINE_SOURCE_REF", f"{stage_id}: {path!r}")
            source_kind = row.get("source_kind")
            if source_kind not in ALLOWED_SOURCE_KINDS:
                return fail("E_LEARNING_SPINE_SOURCE_KIND", f"{stage_id}: {source_kind!r}")
            if source_kind == "proposal" and not path.startswith("docs/context/proposals/"):
                return fail("E_LEARNING_SPINE_PROPOSAL_KIND", f"{stage_id}: {path!r}")
            if source_kind == "implementation_report" and not path.startswith("docs/reports/impl/"):
                return fail("E_LEARNING_SPINE_IMPL_KIND", f"{stage_id}: {path!r}")
        for row in stage.get("pack_refs", []):
            path = str(row.get("path", ""))
            if not path or not (ROOT / path).is_dir():
                return fail("E_LEARNING_SPINE_PACK_REF", f"{stage_id}: {path!r}")
            if row.get("evidence_status") not in ALLOWED_EVIDENCE_STATUS:
                return fail("E_LEARNING_SPINE_PACK_EVIDENCE", f"{stage_id}: {row!r}")
            if row.get("evidence_status") == "verified_by_stage_gate" and not expected_gates:
                return fail("E_LEARNING_SPINE_PACK_GATE_MISSING", f"{stage_id}: {row!r}")
        if stage.get("status") == "landed" and not any(
            row.get("evidence_status") == "verified_by_stage_gate" for row in stage.get("pack_refs", [])
        ):
            return fail("E_LEARNING_SPINE_LANDED_EVIDENCE", str(stage_id))
        for row in stage.get("worlds", []):
            lesson_id = row.get("lesson_id")
            if lesson_id not in known_lessons:
                return fail("E_LEARNING_SPINE_WORLD_REF", f"{stage_id}: {lesson_id!r}")

    stage6 = stages[5]
    if not any("무제한 Prolog" in item for item in stage6.get("open_items", [])):
        return fail("E_LEARNING_SPINE_LOGIC_BOUNDARY", repr(stage6.get("open_items")))
    stage7 = stages[6]
    if not any("별도 사용자 승인" in item for item in stage7.get("open_items", [])):
        return fail("E_LEARNING_SPINE_AI_AUTHORITY", repr(stage7.get("open_items")))
    stage8 = stages[7]
    correspondence = stage8.get("correspondence", [])
    if [row.get("language") for row in correspondence] != ["Python", "Rust", "JavaScript", "Modelica"]:
        return fail("E_LEARNING_SPINE_CORRESPONDENCE", repr(correspondence))
    if any(row.get("status") in {"landed", "complete", "completed"} for row in correspondence):
        return fail("E_LEARNING_SPINE_CORRESPONDENCE_OVERCLAIM", repr(correspondence))
    modelica = correspondence[-1].get("relation", "")
    if "full DAE" not in modelica or "주장하지 않음" not in modelica:
        return fail("E_LEARNING_SPINE_MODELICA_BOUNDARY", repr(modelica))
    return 0


def check_product_wiring() -> int:
    html = (ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "index.html").read_text(encoding="utf-8")
    app = (ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js").read_text(encoding="utf-8")
    styles = (ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "styles.css").read_text(encoding="utf-8")
    module = (ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "learning_spine_index.js").read_text(encoding="utf-8")
    required = [
        ("btn-learning-spine", html),
        ("learning-spine-panel", html),
        ("LEARNING_SPINE_QUERY_KEY", app),
        ("mountLearningSpineIndex", app),
        ("learning_spine", app),
        ("learning-spine-open", styles),
        ("learning-spine-authority-line", styles),
        ("seamgrim.learning_spine.selection.v1", module),
        ("authority: \"none\"", module),
        ("connector_index_only_not_ssot", INDEX.read_text(encoding="utf-8")),
    ]
    missing = [token for token, source in required if token not in source]
    if missing:
        return fail("E_LEARNING_SPINE_WIRING", repr(missing))
    style_version = re.search(r'styles\.css\?v=([^"\']+)', html)
    app_version = re.search(r'app\.js\?v=([^"\']+)', html)
    if not style_version or not app_version or style_version.group(1) != app_version.group(1):
        return fail("E_LEARNING_SPINE_CACHE_VERSION", repr({
            "styles": style_version.group(1) if style_version else None,
            "app": app_version.group(1) if app_version else None,
        }))
    return 0


def gate(cmd: list[str], code: str, timeout: int) -> int:
    proc = run(cmd, timeout)
    return fail(code, proc.stdout.strip()) if proc.returncode != 0 else 0


def main() -> int:
    for check in (check_required_files, check_contract, check_manifest, check_product_wiring):
        rc = check()
        if rc:
            return rc
    stage_gates = []
    for stage in load(INDEX).get("stages", []):
        for gate_cmd in stage.get("evidence_gates", []):
            stage_gates.append((gate_cmd, f"E_LEARNING_SPINE_{stage.get('id')}_EVIDENCE", 1200))
    gates = stage_gates + [
        (["node", "--check", "solutions/seamgrim_ui_mvp/ui/learning_spine_index.js"], "E_LEARNING_SPINE_JS", 60),
        (["node", "tests/seamgrim_learning_spine_index_runner.mjs"], "E_LEARNING_SPINE_BROWSER", 240),
        (["python", "tests/run_pack_golden.py", "learning_spine_index_v1"], "E_LEARNING_SPINE_GOLDEN", 300),
        (["python", "tests/run_tool_world_diff_v1_check.py"], "E_LEARNING_SPINE_W5_REGRESSION", 1200),
    ]
    for cmd, code, timeout in gates:
        rc = gate(cmd, code, timeout)
        if rc:
            return rc
    docs = run(["git", "status", "--short", "--", "docs/ssot"], 60)
    if docs.returncode != 0 or docs.stdout.strip():
        return fail("E_LEARNING_SPINE_SSOT_DIRTY", docs.stdout.strip())
    print("[learning-spine-index-v1] OK connector-index-behavior-closed curriculum-completion-open owner-decision-required")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
