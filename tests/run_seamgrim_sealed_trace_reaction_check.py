#!/usr/bin/env python
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "pack" / "seamgrim_sealed_trace_reaction_product_v1"
LESSON = ROOT / "solutions" / "seamgrim_ui_mvp" / "lessons" / "rep_phys_pendulum_wave1_v1"


def fail(code: str, message: str) -> int:
    print(f"{code}: {message}")
    return 1


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def run(cmd: list[str], *, timeout: int = 300) -> subprocess.CompletedProcess[str]:
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


def check_files_and_contract() -> int:
    required = [
        PACK / "README.md",
        PACK / "contract.detjson",
        PACK / "input.ddn",
        PACK / "golden.jsonl",
        LESSON / "sealed_trace_reactions.detjson",
        ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "trace_reaction_runtime.js",
        ROOT / "tests" / "seamgrim_sealed_trace_reaction_runner.mjs",
    ]
    missing = [str(path.relative_to(ROOT)) for path in required if not path.exists()]
    if missing:
        return fail("E_SEAMGRIM_TRACE_REACTION_MISSING", repr(missing))
    contract = json.loads(read(PACK / "contract.detjson"))
    if contract.get("kind") != "seamgrim_sealed_trace_reaction_product_behavior":
        return fail("E_SEAMGRIM_TRACE_REACTION_CONTRACT_KIND", repr(contract))
    if contract.get("predicate_kinds") != ["edge", "level"]:
        return fail("E_SEAMGRIM_TRACE_REACTION_PREDICATES", repr(contract.get("predicate_kinds")))
    if contract.get("action_kinds") != ["highlight", "visibility", "camera_focus"]:
        return fail("E_SEAMGRIM_TRACE_REACTION_ACTIONS", repr(contract.get("action_kinds")))
    if contract.get("condition_kinds") != ["zero_crossing", "sign_change", "extremum", "tolerance_entry"]:
        return fail("E_SEAMGRIM_TRACE_REACTION_CONDITIONS", repr(contract.get("condition_kinds")))
    if contract.get("condition_evaluation_claim") is not True or contract.get("schedule_kind") != "sealed_observation_trace_evaluator":
        return fail("E_SEAMGRIM_TRACE_REACTION_SCHEDULE_BOUNDARY", repr(contract))
    if contract.get("manual_utterance_tick_truth") is not False:
        return fail("E_SEAMGRIM_TRACE_REACTION_MANUAL_TRUTH", repr(contract))
    if contract.get("observation_trace_schema") != "seamgrim.sealed_observation_trace.v1":
        return fail("E_SEAMGRIM_TRACE_REACTION_OBSERVATION_TRACE_SCHEMA", repr(contract.get("observation_trace_schema")))
    if contract.get("predicate_event_schema") != "seamgrim.trace_predicate_events.v1":
        return fail("E_SEAMGRIM_TRACE_REACTION_EVENT_SCHEMA", repr(contract.get("predicate_event_schema")))
    if contract.get("resolved_scene_trace_schema") != "seamgrim.resolved_scene_trace.v1":
        return fail("E_SEAMGRIM_TRACE_REACTION_RESOLVED_SCHEMA", repr(contract.get("resolved_scene_trace_schema")))
    for key in ("live_online_claim", "implicit_lww_allowed", "stage_plan_created", "state_hash_formula_change", "trace_reaction_state_hash_participation", "parser_change", "runtime_semantics_change", "docs_ssot_change"):
        if contract.get(key) is not False:
            return fail("E_SEAMGRIM_TRACE_REACTION_BOUNDARY", f"{key}={contract.get(key)!r}")
    return 0


def check_product_sources() -> int:
    trace = json.loads(read(LESSON / "sealed_trace_reactions.detjson"))
    if trace.get("schema") != "seamgrim.sealed_trace_reactions.v1":
        return fail("E_SEAMGRIM_TRACE_REACTION_SCHEMA", repr(trace.get("schema")))
    if trace.get("sealed") is not True or trace.get("live_online_claim") is not False:
        return fail("E_SEAMGRIM_TRACE_REACTION_SEALED", repr(trace))
    if trace.get("state_hash_participation") is not False or trace.get("view_plan_hash_participation") is not False:
        return fail("E_SEAMGRIM_TRACE_REACTION_HASH_BOUNDARY", repr(trace))
    predicates = {str(row.get("kind")) for row in trace.get("predicates", []) if isinstance(row, dict)}
    if predicates != {"edge", "level"}:
        return fail("E_SEAMGRIM_TRACE_REACTION_PREDICATE_KIND", repr(predicates))
    condition_kinds = {str(row.get("condition_kind")) for row in trace.get("condition_rules", []) if isinstance(row, dict)}
    if condition_kinds != {"zero_crossing", "sign_change", "extremum", "tolerance_entry"}:
        return fail("E_SEAMGRIM_TRACE_REACTION_CONDITION_KIND", repr(condition_kinds))
    if len(trace.get("condition_rules", [])) != 4:
        return fail("E_SEAMGRIM_TRACE_REACTION_RULE_ROWS", repr(trace.get("condition_rules")))
    if trace.get("predicate_contract", {}).get("condition_evaluation_claim") is not True:
        return fail("E_SEAMGRIM_TRACE_REACTION_EVALUATION_OVERCLAIM", repr(trace.get("predicate_contract")))
    if trace.get("predicate_contract", {}).get("source") != "sealed_observation_trace_evaluator":
        return fail("E_SEAMGRIM_TRACE_REACTION_EVALUATOR_SOURCE", repr(trace.get("predicate_contract")))
    if trace.get("reaction_schedule"):
        return fail("E_SEAMGRIM_TRACE_REACTION_MANUAL_SCHEDULE_PRESENT", repr(trace.get("reaction_schedule")))
    for row in trace.get("condition_rules", []):
        if "utterance_tick" in row or "tick_min" in row or "tick_max" in row:
            return fail("E_SEAMGRIM_TRACE_REACTION_MANUAL_TICK_RULE", repr(row))
    policy = trace.get("resolved_scene_trace_policy", {})
    if policy.get("schema") != "seamgrim.resolved_scene_trace.v1" or policy.get("stage_plan_created") is not False:
        return fail("E_SEAMGRIM_TRACE_REACTION_RESOLVED_POLICY", repr(policy))
    actions = {str(row.get("kind")) for row in trace.get("actions", []) if isinstance(row, dict)}
    if actions != {"highlight", "visibility", "camera_focus"}:
        return fail("E_SEAMGRIM_TRACE_REACTION_ACTION_KIND", repr(actions))
    meta = read(LESSON / "meta.toml")
    if "sealed_trace_reactions_path" not in meta:
        return fail("E_SEAMGRIM_TRACE_REACTION_META_PATH", "sealed_trace_reactions_path missing")
    app_js = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "app.js")
    for token in ["sealed_trace_reactions_path", "sealedTraceReactions"]:
        if token not in app_js:
            return fail("E_SEAMGRIM_TRACE_REACTION_APP_JS", token)
    run_js = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "screens" / "run.js")
    for token in [
        "buildSeamgrimTraceReactionRuntime",
        "__SEAMGRIM_TRACE_REACTION_RUNTIME__",
        "traceReactionStateHashParticipation",
        "traceReactionFrameHash",
    ]:
        if token not in run_js:
            return fail("E_SEAMGRIM_TRACE_REACTION_RUN_JS", token)
    runtime = read(ROOT / "solutions" / "seamgrim_ui_mvp" / "ui" / "trace_reaction_runtime.js")
    for token in [
        "seamgrim.trace_reaction.runtime.v1",
        "seamgrim.resolved_scene_trace.v1",
        "sealed_trace_reaction",
        "camera_focus",
        "E_TRACE_REACTION_CONFLICT",
        "implicit_lww_allowed",
        "condition_kinds",
        "state_hash_participation",
        "view_plan_hash_participation",
        "evaluateSealedTracePredicates",
        "predicate_events",
        "observation_trace_hash_candidate",
    ]:
        if token not in runtime:
            return fail("E_SEAMGRIM_TRACE_REACTION_RUNTIME_JS", token)
    return 0


def check_conflict_diagnostic_runtime() -> int:
    script = r"""
import { buildSeamgrimTraceReactionRuntime } from './solutions/seamgrim_ui_mvp/ui/trace_reaction_runtime.js';
const trace = {
  schema: 'seamgrim.sealed_trace_reactions.v1',
  lesson_id: 'conflict',
  mode: 'sealed_observation_trace',
  sealed: true,
  state_hash_participation: false,
  trace_hash: 'candidate:test',
  predicates: [{ predicate_id: 'edge.a', kind: 'edge' }],
  trace_rows: [{ tick_min: 0, tick_max: 1, time_min: 0, time_max: 1, predicate_states: { 'edge.a': true } }],
  actions: [
    { action_id: 'a1', kind: 'highlight', when: 'edge.a', priority: 1, targets: { shape_ids: ['same'] }, style: { stroke: '#000' } },
    { action_id: 'a2', kind: 'highlight', when: 'edge.a', priority: 1, targets: { shape_ids: ['same'] }, style: { stroke: '#111' } }
  ]
};
const result = buildSeamgrimTraceReactionRuntime({
  traceReactions: trace,
  observation: { values: { tick_id: 0, t: 0 }, all_values: { tick_id: 0, t: 0 } },
  views: { space2d: { schema: 'seamgrim.space2d.v0', shapes: [{ id: 'same', kind: 'point', x: 0, y: 0 }] } },
  viewPlanRuntime: { views: { view_plan: { anchors: [] } } }
});
if (!result?.report?.diagnostics?.some((row) => row.code === 'E_TRACE_REACTION_CONFLICT')) {
  console.error(JSON.stringify(result?.report ?? null));
  process.exit(1);
}
"""
    proc = run(["node", "--input-type=module", "-e", script], timeout=60)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_TRACE_REACTION_CONFLICT_RUNTIME", proc.stdout.strip())
    return 0


def run_gate(cmd: list[str], code: str, timeout: int) -> int:
    proc = run(cmd, timeout=timeout)
    if proc.returncode != 0:
        return fail(code, f"{' '.join(cmd)}: {(proc.stdout or '').strip()}")
    return 0


def check_docs_ssot_clean() -> int:
    proc = run(["git", "status", "--short", "--", "docs/ssot"], timeout=60)
    if proc.returncode != 0:
        return fail("E_SEAMGRIM_TRACE_REACTION_SSOT_STATUS", proc.stdout.strip())
    if proc.stdout.strip():
        return fail("E_SEAMGRIM_TRACE_REACTION_SSOT_DIRTY", proc.stdout.strip())
    return 0


def main() -> int:
    for check in (check_files_and_contract, check_product_sources, check_conflict_diagnostic_runtime):
        rc = check()
        if rc:
            return rc
    gates = [
        (["node", "--check", "tests/seamgrim_sealed_trace_reaction_runner.mjs"], "E_SEAMGRIM_TRACE_REACTION_NODE_CHECK", 60),
        (["node", "tests/seamgrim_sealed_trace_reaction_runner.mjs"], "E_SEAMGRIM_TRACE_REACTION_BROWSER", 240),
        (["python", "tests/run_pack_golden.py", "seamgrim_sealed_trace_reaction_product_v1"], "E_SEAMGRIM_TRACE_REACTION_GOLDEN", 300),
    ]
    for cmd, code, timeout in gates:
        rc = run_gate(cmd, code, timeout)
        if rc:
            return rc
    rc = check_docs_ssot_clean()
    if rc:
        return rc
    print("[seamgrim-sealed-trace-reaction-product-v1] OK behavior-closed sealed-trace-evaluator")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
