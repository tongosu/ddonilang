#!/usr/bin/env python3
import argparse, json, os, re, subprocess, sys
from pathlib import Path

HASH_RE = re.compile(r'^(state_hash|trace_hash|bogae_hash)=(blake3:)?([0-9a-fA-F]{64})$')
LEGACY_HASH_RE = re.compile(r'^(state_hash|trace_hash|bogae_hash)=(0x)?([0-9a-fA-F]{16,128})$')

def parse_walk_arg(s: str):
    parts = []
    for chunk in s.split(","):
        chunk = chunk.strip()
        if not chunk:
            continue
        if "-" in chunk:
            a,b = chunk.split("-",1)
            parts.extend(range(int(a), int(b)+1))
        else:
            parts.append(int(chunk))
    return sorted(set(parts))

def cli_args_from_spec(spec_args):
    cli = list(spec_args.get("cli", []))
    diag = spec_args.get("diag")
    if diag:
        cli += ["--diag", diag]
    if spec_args.get("enable_repro"):
        cli += ["--enable-repro"]
    return cli

def run_cmd(cmd, cwd=None, stdin_text=""):
    p = subprocess.Popen(
        cmd,
        cwd=cwd,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    out, err = p.communicate(stdin_text)
    return p.returncode, out, err

def extract_hashes_and_lines(stdout: str):
    state_hash = None
    trace_hash = None
    bogae_hash = None
    out_lines = [ln.rstrip("\n") for ln in stdout.splitlines()]
    user_lines = []
    for ln in out_lines:
        stripped = ln.strip()
        tokens = stripped.split()
        if not tokens:
            user_lines.append(ln)
            continue
        all_hash_tokens = True
        hash_matches = []
        for token in tokens:
            m = HASH_RE.match(token)
            if m:
                kind = m.group(1)
                hexpart = m.group(3).lower()
                value = f"blake3:{hexpart}"
                hash_matches.append((kind, value))
                continue
            m = LEGACY_HASH_RE.match(token)
            if m:
                kind = m.group(1)
                hexpart = m.group(3).lower()
                if not hexpart.startswith("0x"):
                    hexpart = "0x" + hexpart
                hash_matches.append((kind, hexpart))
                continue
            all_hash_tokens = False
        if all_hash_tokens and hash_matches:
            for kind, value in hash_matches:
                if kind == "state_hash":
                    state_hash = value
                elif kind == "trace_hash":
                    trace_hash = value
                else:
                    bogae_hash = value
            continue
        user_lines.append(ln)
    return user_lines, state_hash, trace_hash, bogae_hash

def load_tests(root: Path, walks):
    tests = []
    for walk in walks:
        wdir = root / f"W{walk:02d}"
        if not wdir.exists():
            continue
        for tdir in sorted([p for p in wdir.iterdir() if p.is_dir()]):
            spec_path = tdir / "test.dtest.json"
            if spec_path.exists():
                spec = json.loads(spec_path.read_text(encoding="utf-8-sig"))
                spec["_dir"] = tdir
                spec["_spec_path"] = spec_path
                tests.append(spec)
    simple_order = {spec.get("name", spec["_dir"].name): spec for spec in tests}
    return tests

def resolve_expect(token, results, key):
    if token is None:
        return ("ANY", None)
    if isinstance(token, str):
        if token in ("ANY","ABSENT"):
            return (token, None)
        if token.startswith("SAME_AS:"):
            other = token.split(":",1)[1]
            return ("SAME_AS", other)
        if token.startswith("DIFFERS_FROM:"):
            other = token.split(":",1)[1]
            return ("DIFFERS_FROM", other)
        if token.startswith("0x") or token.startswith("blake3:"):
            return ("EXACT", token.lower())
    raise ValueError(f"Unsupported expect token for {key}: {token!r}")

def bless_writeback(spec):
    spec_path = spec["_spec_path"]
    spec_to_save = dict(spec)
    spec_to_save.pop("_dir", None)
    spec_to_save.pop("_spec_path", None)
    spec_path.write_text(json.dumps(spec_to_save, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

def ensure_files_exist(tdir: Path, files):
    missing = []
    for f in files:
        if not (tdir / f).exists():
            missing.append(f)
    return missing

def check_files_expectations(tdir: Path, files_spec: dict):
    failures = []
    for name, spec in (files_spec or {}).items():
        path = tdir / name
        if not path.exists():
            failures.append(f"{name}: missing")
            continue
        if "contains" in spec:
            content = path.read_text(encoding="utf-8", errors="replace")
            for needle in spec.get("contains", []):
                if needle not in content:
                    failures.append(f"{name}: missing '{needle}'")
        if "equals_file" in spec:
            expected_path = tdir / spec.get("equals_file")
            if not expected_path.exists():
                failures.append(f"{name}: expected file missing")
            else:
                actual_bytes = path.read_bytes()
                expected_bytes = expected_path.read_bytes()
                if actual_bytes != expected_bytes:
                    failures.append(f"{name}: content mismatch")
        if "jsonl_contains" in spec:
            lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
            objects = []
            for line in lines:
                line = line.strip()
                if not line:
                    continue
                try:
                    objects.append(json.loads(line))
                except json.JSONDecodeError:
                    continue
            for expected in spec.get("jsonl_contains", []):
                found = False
                for obj in objects:
                    if all(obj.get(k) == v for k, v in expected.items()):
                        found = True
                        break
                if not found:
                    failures.append(f"{name}: jsonl missing {expected}")
    return failures

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--root", default="tests/golden", help="Pack root (folder that contains W01..W11)")
    ap.add_argument("--teul-cli", default="teul-cli", help="Path to teul-cli")
    ap.add_argument("--walk", default="1", help="Walk(s): 1, 1-3, 1,2,5")
    ap.add_argument("--madi", type=int, default=None, help="Override madi for run tests (optional)")
    ap.add_argument("--seed", default=None, help="Override seed for run tests (optional, like 0x0)")
    ap.add_argument("--bless", action="store_true", help="Write state_hash/trace_hash into specs when ANY")
    args = ap.parse_args()

    root = Path(args.root).resolve()
    walks = parse_walk_arg(args.walk)
    tests = load_tests(root, walks)
    if not tests:
        print(f"No tests found for walks {walks} under {root}", file=sys.stderr)
        return 2

    results = {}  # name -> dict(exit, stdout_lines, stderr, state_hash, trace_hash)
    failed = 0

    for spec in tests:
        name = spec.get("name") or spec.get("id") or spec["_dir"].name
        cmd_kind = spec.get("command","run")
        entry = spec.get("entry","main.ddn")
        stdin_text = spec.get("stdin","")
        tdir = spec["_dir"]
        spec_args = spec.get("args", {})
        cli_extra = cli_args_from_spec(spec_args)

        if cmd_kind == "run":
            madi = args.madi if args.madi is not None else spec_args.get("madi", spec_args.get("ticks", 1))
            seed = args.seed if args.seed is not None else spec_args.get("seed", "0x0")
            cmd = [args.teul_cli, "run", entry, "--madi", str(madi), "--seed", str(seed)] + list(cli_extra)
        elif cmd_kind == "canon":
            cmd = [args.teul_cli, "canon", "--emit", "ddn", entry] + list(cli_extra)
        elif cmd_kind == "lint":
            cmd = [args.teul_cli, "lint", entry] + list(cli_extra)
        elif cmd_kind == "preproc":
            # convention: teul-cli preproc <in> --out <out>
            out_name = spec_args.get("out", "main.pp.ddn")
            cmd = [args.teul_cli, "preproc", entry, "--out", out_name] + list(cli_extra)
        elif cmd_kind == "check":
            cmd = [args.teul_cli, "check", entry] + list(cli_extra)
        elif cmd_kind == "build":
            cmd = [args.teul_cli, "build", entry] + list(cli_extra)
        elif cmd_kind == "ai_extract":
            out_name = spec_args.get("out", "ai.request.json")
            cmd = [args.teul_cli, "ai", "extract", entry, "--out", out_name] + list(cli_extra)
        elif cmd_kind == "replay_diff":
            a = spec_args.get("a")
            b = spec_args.get("b")
            out_name = spec_args.get("out")
            if not a or not b or not out_name:
                print(f"[{name}] replay_diff requires args a/b/out", file=sys.stderr)
                failed += 1
                continue
            cmd = [args.teul_cli, "replay", "diff", "--a", a, "--b", b, "--out", out_name]
            if spec_args.get("no_summary"):
                cmd.append("--no-summary")
        elif cmd_kind == "replay_verify":
            geoul = spec_args.get("geoul")
            if not geoul:
                print(f"[{name}] replay_verify requires args geoul", file=sys.stderr)
                failed += 1
                continue
            cmd = [args.teul_cli, "replay", "verify", "--geoul", geoul]
            if "until" in spec_args and spec_args.get("until") is not None:
                cmd += ["--until", str(spec_args.get("until"))]
            if "seek" in spec_args and spec_args.get("seek") is not None:
                cmd += ["--seek", str(spec_args.get("seek"))]
            if "entry" in spec_args and spec_args.get("entry") is not None:
                cmd += ["--entry", spec_args.get("entry")]
        elif cmd_kind == "replay_branch":
            geoul = spec_args.get("geoul")
            inject_sam = spec_args.get("inject_sam", spec_args.get("inject-sam"))
            out_name = spec_args.get("out")
            at = spec_args.get("at")
            if not geoul or inject_sam is None or out_name is None or at is None:
                print(f"[{name}] replay_branch requires args geoul/at/inject_sam/out", file=sys.stderr)
                failed += 1
                continue
            cmd = [
                args.teul_cli,
                "replay",
                "branch",
                "--geoul",
                geoul,
                "--at",
                str(at),
                "--inject-sam",
                inject_sam,
                "--out",
                out_name,
            ]
            if spec_args.get("entry") is not None:
                cmd += ["--entry", spec_args.get("entry")]
        elif cmd_kind == "geoul_query":
            geoul = spec_args.get("geoul")
            madi = spec_args.get("madi")
            key = spec_args.get("key")
            if not geoul or madi is None or not key:
                print(f"[{name}] geoul_query requires args geoul/madi/key", file=sys.stderr)
                failed += 1
                continue
            cmd = [
                args.teul_cli,
                "geoul",
                "query",
                "--geoul",
                geoul,
                "--madi",
                str(madi),
                "--key",
                key,
            ]
            if spec_args.get("entry") is not None:
                cmd += ["--entry", spec_args.get("entry")]
        elif cmd_kind == "geoul_backtrace":
            geoul = spec_args.get("geoul")
            key = spec_args.get("key")
            from_madi = spec_args.get("from")
            to_madi = spec_args.get("to")
            if not geoul or not key or from_madi is None or to_madi is None:
                print(f"[{name}] geoul_backtrace requires args geoul/key/from/to", file=sys.stderr)
                failed += 1
                continue
            cmd = [
                args.teul_cli,
                "geoul",
                "backtrace",
                "--geoul",
                geoul,
                "--key",
                key,
                "--from",
                str(from_madi),
                "--to",
                str(to_madi),
            ]
            if spec_args.get("entry") is not None:
                cmd += ["--entry", spec_args.get("entry")]
        elif cmd_kind == "bdl_packet":
            mode = spec_args.get("mode", "wrap")
            input_path = spec_args.get("input") or entry
            out_name = spec_args.get("out")
            if not out_name:
                print(f"[{name}] bdl_packet requires args out", file=sys.stderr)
                failed += 1
                continue
            if mode == "wrap":
                cmd = [args.teul_cli, "bdl", "packet", "wrap", input_path, "--out", out_name]
            elif mode == "unwrap":
                cmd = [args.teul_cli, "bdl", "packet", "unwrap", input_path, "--out", out_name]
            else:
                print(f"[{name}] Unsupported bdl_packet mode: {mode}", file=sys.stderr)
                failed += 1
                continue
        elif cmd_kind == "patch_preview":
            fmt = spec_args.get("format")
            cmd = [args.teul_cli, "patch", "preview", entry]
            if fmt:
                cmd += ["--format", fmt]
            cmd += list(cli_extra)
        elif cmd_kind == "patch_approve":
            out_name = spec_args.get("out")
            yes = spec_args.get("yes", False)
            notes = spec_args.get("notes")
            cmd = [args.teul_cli, "patch", "approve", entry]
            if out_name:
                cmd += ["--out", out_name]
            if yes:
                cmd.append("--yes")
            if notes:
                cmd += ["--notes", notes]
            cmd += list(cli_extra)
        elif cmd_kind == "patch_apply":
            approval = spec_args.get("approval")
            if not approval:
                print(f"[{name}] patch_apply requires args approval", file=sys.stderr)
                failed += 1
                continue
            out_name = spec_args.get("out")
            in_place = spec_args.get("in_place", spec_args.get("in-place", False))
            cmd = [args.teul_cli, "patch", "apply", entry, "--approval", approval]
            if out_name:
                cmd += ["--out", out_name]
            if in_place:
                cmd.append("--in-place")
            cmd += list(cli_extra)
        elif cmd_kind == "patch_verify":
            approval = spec_args.get("approval")
            if not approval:
                print(f"[{name}] patch_verify requires args approval", file=sys.stderr)
                failed += 1
                continue
            tests_root = spec_args.get("tests")
            walk = spec_args.get("walk")
            cmd = [args.teul_cli, "patch", "verify", entry, "--approval", approval]
            if tests_root:
                cmd += ["--tests", tests_root]
            if walk:
                cmd += ["--walk", str(walk)]
            cmd += list(cli_extra)
        elif cmd_kind == "patch_propose":
            out_name = spec_args.get("out")
            cmd = [args.teul_cli, "patch", "propose", entry]
            if out_name:
                cmd += ["--out", out_name]
            cmd += list(cli_extra)
        elif cmd_kind == "ai_prompt":
            profile = spec_args.get("profile", "lean")
            out_name = spec_args.get("out", "ai.prompt.txt")
            bundle = spec_args.get("bundle")
            cmd = [args.teul_cli, "ai", "prompt", "--profile", profile, "--out", out_name]
            if bundle:
                cmd += ["--bundle", bundle]
            cmd += list(cli_extra)
        elif cmd_kind == "intent_inspect":
            geoul = spec_args.get("geoul", entry)
            cmd = [args.teul_cli, "intent", "inspect", "--geoul", geoul]
            if spec_args.get("madi") is not None:
                cmd += ["--madi", str(spec_args.get("madi"))]
            if spec_args.get("agent") is not None:
                cmd += ["--agent", str(spec_args.get("agent"))]
            if spec_args.get("out"):
                cmd += ["--out", spec_args.get("out")]
        elif cmd_kind == "intent_mock":
            out_name = spec_args.get("out")
            if not out_name:
                print(f"[{name}] intent_mock requires args out", file=sys.stderr)
                failed += 1
                continue
            cmd = [args.teul_cli, "intent", "mock", entry, "--out", out_name]
            if spec_args.get("agent_id") is not None:
                cmd += ["--agent-id", str(spec_args.get("agent_id"))]
            if spec_args.get("madi") is not None:
                cmd += ["--madi", str(spec_args.get("madi"))]
            if spec_args.get("recv_seq") is not None:
                cmd += ["--recv-seq", str(spec_args.get("recv_seq"))]
        elif cmd_kind == "story_make":
            geoul = spec_args.get("geoul")
            out_name = spec_args.get("out")
            if not geoul or not out_name:
                print(f"[{name}] story_make requires args geoul/out", file=sys.stderr)
                failed += 1
                continue
            cmd = [args.teul_cli, "story", "make", "--geoul", geoul, "--out", out_name]
        elif cmd_kind == "timeline_make":
            geoul = spec_args.get("geoul")
            story = spec_args.get("story")
            out_name = spec_args.get("out")
            if not geoul or not story or not out_name:
                print(f"[{name}] timeline_make requires args geoul/story/out", file=sys.stderr)
                failed += 1
                continue
            cmd = [
                args.teul_cli,
                "timeline",
                "make",
                "--geoul",
                geoul,
                "--story",
                story,
                "--out",
                out_name,
            ]
        elif cmd_kind == "intent_merge":
            inputs = spec_args.get("inputs") or spec_args.get("in")
            if not inputs:
                print(f"[{name}] intent_merge requires args inputs", file=sys.stderr)
                failed += 1
                continue
            cmd = [args.teul_cli, "intent", "merge"]
            for path in inputs:
                cmd += ["--in", path]
            if spec_args.get("madi") is not None:
                cmd += ["--madi", str(spec_args.get("madi"))]
            if spec_args.get("agent") is not None:
                cmd += ["--agent", str(spec_args.get("agent"))]
            if spec_args.get("out"):
                cmd += ["--out", spec_args.get("out")]
        elif cmd_kind == "goal_parse":
            cmd = [args.teul_cli, "goal", "parse", entry]
            if spec_args.get("out"):
                cmd += ["--out", spec_args.get("out")]
        elif cmd_kind == "goal_plan":
            actions = spec_args.get("actions", entry)
            cmd = [args.teul_cli, "goal", "plan", "--actions", actions]
            if spec_args.get("out"):
                cmd += ["--out", spec_args.get("out")]
        elif cmd_kind == "goap_plan":
            cmd = [args.teul_cli, "goap", "plan", entry]
            if spec_args.get("out"):
                cmd += ["--out", spec_args.get("out")]
        elif cmd_kind == "observation_canon":
            cmd = [args.teul_cli, "observation", "canon", entry]
            if spec_args.get("out"):
                cmd += ["--out", spec_args.get("out")]
        elif cmd_kind == "nurigym_spec":
            out_name = spec_args.get("out", "build/nurigym")
            cmd = [args.teul_cli, "nurigym", "spec", "--from", entry, "--out", out_name]
            if spec_args.get("slots") is not None:
                cmd += ["--slots", str(spec_args.get("slots"))]
        elif cmd_kind == "nurigym_view":
            spec_path = spec_args.get("spec", entry)
            cmd = [args.teul_cli, "nurigym", "view", "--spec", spec_path]
        elif cmd_kind == "latency_simulate":
            cmd = [args.teul_cli, "latency", "simulate"]
            if spec_args.get("L") is not None:
                cmd += ["--L", str(spec_args.get("L"))]
            if spec_args.get("mode") is not None:
                cmd += ["--mode", spec_args.get("mode")]
            if spec_args.get("count") is not None:
                cmd += ["--count", str(spec_args.get("count"))]
            if spec_args.get("seed") is not None:
                cmd += ["--seed", str(spec_args.get("seed"))]
            if spec_args.get("out"):
                cmd += ["--out", spec_args.get("out")]
        elif cmd_kind == "safety_check":
            rules = spec_args.get("rules")
            intent = spec_args.get("intent")
            if not rules or not intent:
                print(f"[{name}] safety_check requires args rules/intent", file=sys.stderr)
                failed += 1
                continue
            cmd = [args.teul_cli, "safety", "check", "--rules", rules, "--intent", intent]
            if spec_args.get("out"):
                cmd += ["--out", spec_args.get("out")]
        elif cmd_kind == "dataset_export":
            geoul = spec_args.get("geoul")
            out_name = spec_args.get("out")
            if not geoul or not out_name:
                print(f"[{name}] dataset_export requires args geoul/out", file=sys.stderr)
                failed += 1
                continue
            cmd = [args.teul_cli, "dataset", "export", "--geoul", geoul, "--out", out_name]
            if spec_args.get("format"):
                cmd += ["--format", spec_args.get("format")]
            if spec_args.get("env_id"):
                cmd += ["--env-id", spec_args.get("env_id")]
        elif cmd_kind == "workshop_gen":
            geoul = spec_args.get("geoul")
            out_name = spec_args.get("out")
            if not geoul or not out_name:
                print(f"[{name}] workshop_gen requires args geoul/out", file=sys.stderr)
                failed += 1
                continue
            cmd = [args.teul_cli, "workshop", "gen", "--geoul", geoul, "--out", out_name]
        elif cmd_kind == "workshop_apply":
            workshop = spec_args.get("workshop")
            patch = spec_args.get("patch")
            if not workshop or not patch:
                print(f"[{name}] workshop_apply requires args workshop/patch", file=sys.stderr)
                failed += 1
                continue
            cmd = [args.teul_cli, "workshop", "apply", "--workshop", workshop, "--patch", patch]
        elif cmd_kind == "workshop_open":
            workshop = spec_args.get("workshop")
            if not workshop:
                print(f"[{name}] workshop_open requires args workshop", file=sys.stderr)
                failed += 1
                continue
            cmd = [args.teul_cli, "workshop", "open", "--workshop", workshop]
        elif cmd_kind == "view":
            cmd = [args.teul_cli, "view", entry] + list(cli_extra)
        elif cmd_kind == "multi":
            steps = spec_args.get("steps") or spec.get("steps") or []
            if not steps:
                steps = [{"cmd": "run", "in": entry, "args": {}}]
            rc = 0
            full_out = ""
            full_err = ""
            last_user_lines = []
            last_state_hash = None
            last_trace_hash = None
            last_bogae_hash = None
            for st in steps:
                kind = st.get("cmd") or st.get("command") or "run"
                ent = st.get("in") or st.get("entry") or entry
                st_args = st.get("args", st)
                st_cli = cli_extra + cli_args_from_spec(st_args)
                if kind == "canon":
                    emit = st.get("emit", "ddn")
                    cmd = [args.teul_cli, "canon", "--emit", emit, ent]
                    bridge = st.get("bridge")
                    if bridge:
                        cmd += ["--bridge", bridge]
                    out_name = st.get("out")
                    if out_name:
                        cmd += ["--out", out_name]
                    cmd += list(st_cli)
                elif kind == "check":
                    cmd = [args.teul_cli, "check", ent] + list(st_cli)
                elif kind == "run":
                    madi = st.get("madi", st.get("ticks", spec_args.get("madi", spec_args.get("ticks", 1))))
                    seed = st.get("seed", spec_args.get("seed", "0x0"))
                    cmd = [args.teul_cli, "run", ent, "--madi", str(madi), "--seed", str(seed)] + list(st_cli)
                elif kind == "view":
                    cmd = [args.teul_cli, "view", ent] + list(st_cli)
                elif kind == "replay_diff":
                    a = st.get("a")
                    b = st.get("b")
                    out_name = st.get("out")
                    if not a or not b or not out_name:
                        print(f"[{name}] replay_diff requires args a/b/out", file=sys.stderr)
                        rc = 99
                        break
                    cmd = [args.teul_cli, "replay", "diff", "--a", a, "--b", b, "--out", out_name]
                    if st.get("no_summary"):
                        cmd.append("--no-summary")
                elif kind == "replay_verify":
                    geoul = st.get("geoul")
                    if not geoul:
                        print(f"[{name}] replay_verify requires args geoul", file=sys.stderr)
                        rc = 99
                        break
                    cmd = [args.teul_cli, "replay", "verify", "--geoul", geoul]
                    if st.get("until") is not None:
                        cmd += ["--until", str(st.get("until"))]
                    if st.get("seek") is not None:
                        cmd += ["--seek", str(st.get("seek"))]
                    if st.get("entry") is not None:
                        cmd += ["--entry", st.get("entry")]
                elif kind == "replay_branch":
                    geoul = st.get("geoul")
                    inject_sam = st.get("inject_sam", st.get("inject-sam"))
                    out_name = st.get("out")
                    at = st.get("at")
                    if not geoul or inject_sam is None or out_name is None or at is None:
                        print(f"[{name}] replay_branch requires args geoul/at/inject_sam/out", file=sys.stderr)
                        rc = 99
                        break
                    cmd = [
                        args.teul_cli,
                        "replay",
                        "branch",
                        "--geoul",
                        geoul,
                        "--at",
                        str(at),
                        "--inject-sam",
                        inject_sam,
                        "--out",
                        out_name,
                    ]
                    if st.get("entry") is not None:
                        cmd += ["--entry", st.get("entry")]
                elif kind == "geoul_query":
                    geoul = st.get("geoul")
                    madi = st.get("madi")
                    key = st.get("key")
                    if not geoul or madi is None or not key:
                        print(f"[{name}] geoul_query requires args geoul/madi/key", file=sys.stderr)
                        rc = 99
                        break
                    cmd = [
                        args.teul_cli,
                        "geoul",
                        "query",
                        "--geoul",
                        geoul,
                        "--madi",
                        str(madi),
                        "--key",
                        key,
                    ]
                    if st.get("entry") is not None:
                        cmd += ["--entry", st.get("entry")]
                elif kind == "geoul_backtrace":
                    geoul = st.get("geoul")
                    key = st.get("key")
                    from_madi = st.get("from")
                    to_madi = st.get("to")
                    if not geoul or not key or from_madi is None or to_madi is None:
                        print(f"[{name}] geoul_backtrace requires args geoul/key/from/to", file=sys.stderr)
                        rc = 99
                        break
                    cmd = [
                        args.teul_cli,
                        "geoul",
                        "backtrace",
                        "--geoul",
                        geoul,
                        "--key",
                        key,
                        "--from",
                        str(from_madi),
                        "--to",
                        str(to_madi),
                    ]
                    if st.get("entry") is not None:
                        cmd += ["--entry", st.get("entry")]
                elif kind == "bdl_packet":
                    mode = st.get("mode", "wrap")
                    input_path = st.get("input") or ent
                    out_name = st.get("out")
                    if not out_name:
                        print(f"[{name}] bdl_packet requires args out", file=sys.stderr)
                        rc = 99
                        break
                    if mode == "wrap":
                        cmd = [args.teul_cli, "bdl", "packet", "wrap", input_path, "--out", out_name]
                    elif mode == "unwrap":
                        cmd = [args.teul_cli, "bdl", "packet", "unwrap", input_path, "--out", out_name]
                    else:
                        print(f"[{name}] Unsupported bdl_packet mode: {mode}", file=sys.stderr)
                        rc = 99
                        break
                elif kind == "patch_preview":
                    fmt = st_args.get("format")
                    cmd = [args.teul_cli, "patch", "preview", ent]
                    if fmt:
                        cmd += ["--format", fmt]
                    cmd += list(st_cli)
                elif kind == "patch_approve":
                    out_name = st_args.get("out")
                    yes = st_args.get("yes", False)
                    notes = st_args.get("notes")
                    cmd = [args.teul_cli, "patch", "approve", ent]
                    if out_name:
                        cmd += ["--out", out_name]
                    if yes:
                        cmd.append("--yes")
                    if notes:
                        cmd += ["--notes", notes]
                    cmd += list(st_cli)
                elif kind == "patch_apply":
                    approval = st_args.get("approval")
                    if not approval:
                        print(f"[{name}] patch_apply requires args approval", file=sys.stderr)
                        rc = 99
                        break
                    out_name = st_args.get("out")
                    in_place = st_args.get("in_place", st_args.get("in-place", False))
                    cmd = [args.teul_cli, "patch", "apply", ent, "--approval", approval]
                    if out_name:
                        cmd += ["--out", out_name]
                    if in_place:
                        cmd.append("--in-place")
                    cmd += list(st_cli)
                elif kind == "patch_verify":
                    approval = st_args.get("approval")
                    if not approval:
                        print(f"[{name}] patch_verify requires args approval", file=sys.stderr)
                        rc = 99
                        break
                    tests_root = st_args.get("tests")
                    walk = st_args.get("walk")
                    cmd = [args.teul_cli, "patch", "verify", ent, "--approval", approval]
                    if tests_root:
                        cmd += ["--tests", tests_root]
                    if walk:
                        cmd += ["--walk", str(walk)]
                    cmd += list(st_cli)
                elif kind == "patch_propose":
                    out_name = st_args.get("out")
                    cmd = [args.teul_cli, "patch", "propose", ent]
                    if out_name:
                        cmd += ["--out", out_name]
                    cmd += list(st_cli)
                elif kind == "ai_prompt":
                    profile = st_args.get("profile", "lean")
                    out_name = st_args.get("out", "ai.prompt.txt")
                    bundle = st_args.get("bundle")
                    cmd = [args.teul_cli, "ai", "prompt", "--profile", profile, "--out", out_name]
                    if bundle:
                        cmd += ["--bundle", bundle]
                    cmd += list(st_cli)
                elif kind == "intent_inspect":
                    geoul = st_args.get("geoul", ent)
                    cmd = [args.teul_cli, "intent", "inspect", "--geoul", geoul]
                    if st_args.get("madi") is not None:
                        cmd += ["--madi", str(st_args.get("madi"))]
                    if st_args.get("agent") is not None:
                        cmd += ["--agent", str(st_args.get("agent"))]
                    if st_args.get("out"):
                        cmd += ["--out", st_args.get("out")]
                elif kind == "intent_mock":
                    out_name = st_args.get("out")
                    if not out_name:
                        print(f"[{name}] intent_mock requires args out", file=sys.stderr)
                        rc = 99
                        break
                    cmd = [args.teul_cli, "intent", "mock", ent, "--out", out_name]
                    if st_args.get("agent_id") is not None:
                        cmd += ["--agent-id", str(st_args.get("agent_id"))]
                    if st_args.get("madi") is not None:
                        cmd += ["--madi", str(st_args.get("madi"))]
                    if st_args.get("recv_seq") is not None:
                        cmd += ["--recv-seq", str(st_args.get("recv_seq"))]
                elif kind == "story_make":
                    geoul = st_args.get("geoul")
                    out_name = st_args.get("out")
                    if not geoul or not out_name:
                        print(f"[{name}] story_make requires args geoul/out", file=sys.stderr)
                        rc = 99
                        break
                    cmd = [args.teul_cli, "story", "make", "--geoul", geoul, "--out", out_name]
                elif kind == "timeline_make":
                    geoul = st_args.get("geoul")
                    story = st_args.get("story")
                    out_name = st_args.get("out")
                    if not geoul or not story or not out_name:
                        print(f"[{name}] timeline_make requires args geoul/story/out", file=sys.stderr)
                        rc = 99
                        break
                    cmd = [
                        args.teul_cli,
                        "timeline",
                        "make",
                        "--geoul",
                        geoul,
                        "--story",
                        story,
                        "--out",
                        out_name,
                    ]
                elif kind == "intent_merge":
                    inputs = st_args.get("inputs") or st_args.get("in")
                    if not inputs:
                        print(f"[{name}] intent_merge requires args inputs", file=sys.stderr)
                        rc = 99
                        break
                    cmd = [args.teul_cli, "intent", "merge"]
                    for path in inputs:
                        cmd += ["--in", path]
                    if st_args.get("madi") is not None:
                        cmd += ["--madi", str(st_args.get("madi"))]
                    if st_args.get("agent") is not None:
                        cmd += ["--agent", str(st_args.get("agent"))]
                    if st_args.get("out"):
                        cmd += ["--out", st_args.get("out")]
                elif kind == "goal_parse":
                    cmd = [args.teul_cli, "goal", "parse", ent]
                    if st_args.get("out"):
                        cmd += ["--out", st_args.get("out")]
                elif kind == "goal_plan":
                    actions = st_args.get("actions", ent)
                    cmd = [args.teul_cli, "goal", "plan", "--actions", actions]
                    if st_args.get("out"):
                        cmd += ["--out", st_args.get("out")]
                elif kind == "goap_plan":
                    cmd = [args.teul_cli, "goap", "plan", ent]
                    if st_args.get("out"):
                        cmd += ["--out", st_args.get("out")]
                elif kind == "observation_canon":
                    cmd = [args.teul_cli, "observation", "canon", ent]
                    if st_args.get("out"):
                        cmd += ["--out", st_args.get("out")]
                elif kind == "nurigym_spec":
                    out_name = st_args.get("out", "build/nurigym")
                    cmd = [args.teul_cli, "nurigym", "spec", "--from", ent, "--out", out_name]
                    if st_args.get("slots") is not None:
                        cmd += ["--slots", str(st_args.get("slots"))]
                elif kind == "nurigym_view":
                    spec_path = st_args.get("spec", ent)
                    cmd = [args.teul_cli, "nurigym", "view", "--spec", spec_path]
                elif kind == "latency_simulate":
                    cmd = [args.teul_cli, "latency", "simulate"]
                    if st_args.get("L") is not None:
                        cmd += ["--L", str(st_args.get("L"))]
                    if st_args.get("mode") is not None:
                        cmd += ["--mode", st_args.get("mode")]
                    if st_args.get("count") is not None:
                        cmd += ["--count", str(st_args.get("count"))]
                    if st_args.get("seed") is not None:
                        cmd += ["--seed", str(st_args.get("seed"))]
                    if st_args.get("out"):
                        cmd += ["--out", st_args.get("out")]
                elif kind == "safety_check":
                    rules = st_args.get("rules")
                    intent = st_args.get("intent")
                    if not rules or not intent:
                        print(f"[{name}] safety_check requires args rules/intent", file=sys.stderr)
                        rc = 99
                        break
                    cmd = [args.teul_cli, "safety", "check", "--rules", rules, "--intent", intent]
                    if st_args.get("out"):
                        cmd += ["--out", st_args.get("out")]
                elif kind == "dataset_export":
                    geoul = st_args.get("geoul")
                    out_name = st_args.get("out")
                    if not geoul or not out_name:
                        print(f"[{name}] dataset_export requires args geoul/out", file=sys.stderr)
                        rc = 99
                        break
                    cmd = [args.teul_cli, "dataset", "export", "--geoul", geoul, "--out", out_name]
                    if st_args.get("format"):
                        cmd += ["--format", st_args.get("format")]
                    if st_args.get("env_id"):
                        cmd += ["--env-id", st_args.get("env_id")]
                elif kind == "workshop_gen":
                    geoul = st_args.get("geoul")
                    out_name = st_args.get("out")
                    if not geoul or not out_name:
                        print(f"[{name}] workshop_gen requires args geoul/out", file=sys.stderr)
                        rc = 99
                        break
                    cmd = [args.teul_cli, "workshop", "gen", "--geoul", geoul, "--out", out_name]
                elif kind == "workshop_apply":
                    workshop = st_args.get("workshop")
                    patch = st_args.get("patch")
                    if not workshop or not patch:
                        print(f"[{name}] workshop_apply requires args workshop/patch", file=sys.stderr)
                        rc = 99
                        break
                    cmd = [args.teul_cli, "workshop", "apply", "--workshop", workshop, "--patch", patch]
                elif kind == "workshop_open":
                    workshop = st_args.get("workshop")
                    if not workshop:
                        print(f"[{name}] workshop_open requires args workshop", file=sys.stderr)
                        rc = 99
                        break
                    cmd = [args.teul_cli, "workshop", "open", "--workshop", workshop]
                else:
                    print(f"[{name}] Unsupported multi step: {kind}", file=sys.stderr)
                    rc = 99
                    break
                rc, out, err = run_cmd(cmd, cwd=str(tdir), stdin_text=stdin_text)
                full_out += out
                full_err += err
                if kind == "run":
                    (
                        last_user_lines,
                        last_state_hash,
                        last_trace_hash,
                        last_bogae_hash,
                    ) = extract_hashes_and_lines(out)
                elif kind in ("replay_verify", "replay_branch", "geoul_query", "geoul_backtrace"):
                    last_user_lines = [ln.rstrip("\n") for ln in out.splitlines()]
                if rc != 0:
                    break
            results[name] = {
                "exit": rc,
                "stdout_lines": last_user_lines,
                "stdout_text": full_out,
                "stderr": full_err,
                "state_hash": last_state_hash,
                "trace_hash": last_trace_hash,
                "bogae_hash": last_bogae_hash,
            }
            rc = results[name]["exit"]
            user_lines = results[name]["stdout_lines"]
            err = results[name]["stderr"]
            state_hash = results[name]["state_hash"]
            trace_hash = results[name]["trace_hash"]
            bogae_hash = results[name]["bogae_hash"]
            # fallthrough to validation
        elif cmd_kind == "pipeline":
            # steps: [{command, entry, args...}]
            steps = spec.get("steps", [])
            rc = 0
            full_out = ""
            full_err = ""
            last_user_lines = []
            last_state_hash = None
            last_trace_hash = None
            last_bogae_hash = None
            for st in steps:
                kind = st.get("command","run")
                ent = st.get("entry", entry)
                st_args = st.get("args", {})
                st_cli = cli_extra + cli_args_from_spec(st_args)
                if kind == "preproc":
                    out_name = st_args.get("out","main.pp.ddn")
                    cmd = [args.teul_cli, "preproc", ent, "--out", out_name] + list(st_cli)
                elif kind == "run":
                    madi = st_args.get("madi", st_args.get("ticks", spec_args.get("madi", spec_args.get("ticks", 1))))
                    seed = st_args.get("seed", spec_args.get("seed", "0x0"))
                    cmd = [args.teul_cli, "run", ent, "--madi", str(madi), "--seed", str(seed)] + list(st_cli)
                elif kind == "view":
                    cmd = [args.teul_cli, "view", ent] + list(st_cli)
                elif kind == "canon":
                    cmd = [args.teul_cli, "canon", "--emit", "ddn", ent] + list(st_cli)
                elif kind == "check":
                    cmd = [args.teul_cli, "check", ent] + list(st_cli)
                elif kind == "build":
                    cmd = [args.teul_cli, "build", ent] + list(st_cli)
                elif kind == "replay_diff":
                    a = st_args.get("a")
                    b = st_args.get("b")
                    out_name = st_args.get("out")
                    if not a or not b or not out_name:
                        print(f"[{name}] replay_diff requires args a/b/out", file=sys.stderr)
                        rc = 99
                        break
                    cmd = [args.teul_cli, "replay", "diff", "--a", a, "--b", b, "--out", out_name]
                    if st_args.get("no_summary"):
                        cmd.append("--no-summary")
                elif kind == "replay_verify":
                    geoul = st_args.get("geoul")
                    if not geoul:
                        print(f"[{name}] replay_verify requires args geoul", file=sys.stderr)
                        rc = 99
                        break
                    cmd = [args.teul_cli, "replay", "verify", "--geoul", geoul]
                    if st_args.get("until") is not None:
                        cmd += ["--until", str(st_args.get("until"))]
                    if st_args.get("seek") is not None:
                        cmd += ["--seek", str(st_args.get("seek"))]
                    if st_args.get("entry") is not None:
                        cmd += ["--entry", st_args.get("entry")]
                elif kind == "replay_branch":
                    geoul = st_args.get("geoul")
                    inject_sam = st_args.get("inject_sam", st_args.get("inject-sam"))
                    out_name = st_args.get("out")
                    at = st_args.get("at")
                    if not geoul or inject_sam is None or out_name is None or at is None:
                        print(f"[{name}] replay_branch requires args geoul/at/inject_sam/out", file=sys.stderr)
                        rc = 99
                        break
                    cmd = [
                        args.teul_cli,
                        "replay",
                        "branch",
                        "--geoul",
                        geoul,
                        "--at",
                        str(at),
                        "--inject-sam",
                        inject_sam,
                        "--out",
                        out_name,
                    ]
                    if st_args.get("entry") is not None:
                        cmd += ["--entry", st_args.get("entry")]
                elif kind == "geoul_query":
                    geoul = st_args.get("geoul")
                    madi = st_args.get("madi")
                    key = st_args.get("key")
                    if not geoul or madi is None or not key:
                        print(f"[{name}] geoul_query requires args geoul/madi/key", file=sys.stderr)
                        rc = 99
                        break
                    cmd = [
                        args.teul_cli,
                        "geoul",
                        "query",
                        "--geoul",
                        geoul,
                        "--madi",
                        str(madi),
                        "--key",
                        key,
                    ]
                    if st_args.get("entry") is not None:
                        cmd += ["--entry", st_args.get("entry")]
                elif kind == "geoul_backtrace":
                    geoul = st_args.get("geoul")
                    key = st_args.get("key")
                    from_madi = st_args.get("from")
                    to_madi = st_args.get("to")
                    if not geoul or not key or from_madi is None or to_madi is None:
                        print(f"[{name}] geoul_backtrace requires args geoul/key/from/to", file=sys.stderr)
                        rc = 99
                        break
                    cmd = [
                        args.teul_cli,
                        "geoul",
                        "backtrace",
                        "--geoul",
                        geoul,
                        "--key",
                        key,
                        "--from",
                        str(from_madi),
                        "--to",
                        str(to_madi),
                    ]
                    if st_args.get("entry") is not None:
                        cmd += ["--entry", st_args.get("entry")]
                elif kind == "replay_branch":
                    geoul = st_args.get("geoul")
                    inject_sam = st_args.get("inject_sam", st_args.get("inject-sam"))
                    out_name = st_args.get("out")
                    at = st_args.get("at")
                    if not geoul or inject_sam is None or out_name is None or at is None:
                        print(f"[{name}] replay_branch requires args geoul/at/inject_sam/out", file=sys.stderr)
                        rc = 99
                        break
                    cmd = [
                        args.teul_cli,
                        "replay",
                        "branch",
                        "--geoul",
                        geoul,
                        "--at",
                        str(at),
                        "--inject-sam",
                        inject_sam,
                        "--out",
                        out_name,
                    ]
                    if st_args.get("entry") is not None:
                        cmd += ["--entry", st_args.get("entry")]
                elif kind == "bdl_packet":
                    mode = st_args.get("mode", "wrap")
                    input_path = st_args.get("input") or ent
                    out_name = st_args.get("out")
                    if not out_name:
                        print(f"[{name}] bdl_packet requires args out", file=sys.stderr)
                        rc = 99
                        break
                    if mode == "wrap":
                        cmd = [args.teul_cli, "bdl", "packet", "wrap", input_path, "--out", out_name]
                    elif mode == "unwrap":
                        cmd = [args.teul_cli, "bdl", "packet", "unwrap", input_path, "--out", out_name]
                    else:
                        print(f"[{name}] Unsupported bdl_packet mode: {mode}", file=sys.stderr)
                        rc = 99
                        break
                elif kind == "patch_preview":
                    fmt = st_args.get("format")
                    cmd = [args.teul_cli, "patch", "preview", ent]
                    if fmt:
                        cmd += ["--format", fmt]
                    cmd += list(st_cli)
                elif kind == "patch_approve":
                    out_name = st_args.get("out")
                    yes = st_args.get("yes", False)
                    notes = st_args.get("notes")
                    cmd = [args.teul_cli, "patch", "approve", ent]
                    if out_name:
                        cmd += ["--out", out_name]
                    if yes:
                        cmd.append("--yes")
                    if notes:
                        cmd += ["--notes", notes]
                    cmd += list(st_cli)
                elif kind == "patch_apply":
                    approval = st_args.get("approval")
                    if not approval:
                        print(f"[{name}] patch_apply requires args approval", file=sys.stderr)
                        rc = 99
                        break
                    out_name = st_args.get("out")
                    in_place = st_args.get("in_place", st_args.get("in-place", False))
                    cmd = [args.teul_cli, "patch", "apply", ent, "--approval", approval]
                    if out_name:
                        cmd += ["--out", out_name]
                    if in_place:
                        cmd.append("--in-place")
                    cmd += list(st_cli)
                elif kind == "patch_verify":
                    approval = st_args.get("approval")
                    if not approval:
                        print(f"[{name}] patch_verify requires args approval", file=sys.stderr)
                        rc = 99
                        break
                    tests_root = st_args.get("tests")
                    walk = st_args.get("walk")
                    cmd = [args.teul_cli, "patch", "verify", ent, "--approval", approval]
                    if tests_root:
                        cmd += ["--tests", tests_root]
                    if walk:
                        cmd += ["--walk", str(walk)]
                    cmd += list(st_cli)
                elif kind == "intent_inspect":
                    geoul = st_args.get("geoul", ent)
                    cmd = [args.teul_cli, "intent", "inspect", "--geoul", geoul]
                    if st_args.get("madi") is not None:
                        cmd += ["--madi", str(st_args.get("madi"))]
                    if st_args.get("agent") is not None:
                        cmd += ["--agent", str(st_args.get("agent"))]
                    if st_args.get("out"):
                        cmd += ["--out", st_args.get("out")]
                elif kind == "intent_mock":
                    out_name = st_args.get("out")
                    if not out_name:
                        print(f"[{name}] intent_mock requires args out", file=sys.stderr)
                        rc = 99
                        break
                    cmd = [args.teul_cli, "intent", "mock", ent, "--out", out_name]
                    if st_args.get("agent_id") is not None:
                        cmd += ["--agent-id", str(st_args.get("agent_id"))]
                    if st_args.get("madi") is not None:
                        cmd += ["--madi", str(st_args.get("madi"))]
                    if st_args.get("recv_seq") is not None:
                        cmd += ["--recv-seq", str(st_args.get("recv_seq"))]
                elif kind == "intent_merge":
                    inputs = st_args.get("inputs") or st_args.get("in")
                    if not inputs:
                        print(f"[{name}] intent_merge requires args inputs", file=sys.stderr)
                        rc = 99
                        break
                    cmd = [args.teul_cli, "intent", "merge"]
                    for path in inputs:
                        cmd += ["--in", path]
                    if st_args.get("madi") is not None:
                        cmd += ["--madi", str(st_args.get("madi"))]
                    if st_args.get("agent") is not None:
                        cmd += ["--agent", str(st_args.get("agent"))]
                    if st_args.get("out"):
                        cmd += ["--out", st_args.get("out")]
                elif kind == "goal_parse":
                    cmd = [args.teul_cli, "goal", "parse", ent]
                    if st_args.get("out"):
                        cmd += ["--out", st_args.get("out")]
                elif kind == "goal_plan":
                    actions = st_args.get("actions", ent)
                    cmd = [args.teul_cli, "goal", "plan", "--actions", actions]
                    if st_args.get("out"):
                        cmd += ["--out", st_args.get("out")]
                elif kind == "goap_plan":
                    cmd = [args.teul_cli, "goap", "plan", ent]
                    if st_args.get("out"):
                        cmd += ["--out", st_args.get("out")]
                elif kind == "observation_canon":
                    cmd = [args.teul_cli, "observation", "canon", ent]
                    if st_args.get("out"):
                        cmd += ["--out", st_args.get("out")]
                elif kind == "nurigym_spec":
                    out_name = st_args.get("out", "build/nurigym")
                    cmd = [args.teul_cli, "nurigym", "spec", "--from", ent, "--out", out_name]
                    if st_args.get("slots") is not None:
                        cmd += ["--slots", str(st_args.get("slots"))]
                elif kind == "nurigym_view":
                    spec_path = st_args.get("spec", ent)
                    cmd = [args.teul_cli, "nurigym", "view", "--spec", spec_path]
                elif kind == "latency_simulate":
                    cmd = [args.teul_cli, "latency", "simulate"]
                    if st_args.get("L") is not None:
                        cmd += ["--L", str(st_args.get("L"))]
                    if st_args.get("mode") is not None:
                        cmd += ["--mode", st_args.get("mode")]
                    if st_args.get("count") is not None:
                        cmd += ["--count", str(st_args.get("count"))]
                    if st_args.get("seed") is not None:
                        cmd += ["--seed", str(st_args.get("seed"))]
                    if st_args.get("out"):
                        cmd += ["--out", st_args.get("out")]
                elif kind == "safety_check":
                    rules = st_args.get("rules")
                    intent = st_args.get("intent")
                    if not rules or not intent:
                        print(f"[{name}] safety_check requires args rules/intent", file=sys.stderr)
                        rc = 99
                        break
                    cmd = [args.teul_cli, "safety", "check", "--rules", rules, "--intent", intent]
                    if st_args.get("out"):
                        cmd += ["--out", st_args.get("out")]
                elif kind == "dataset_export":
                    geoul = st_args.get("geoul")
                    out_name = st_args.get("out")
                    if not geoul or not out_name:
                        print(f"[{name}] dataset_export requires args geoul/out", file=sys.stderr)
                        rc = 99
                        break
                    cmd = [args.teul_cli, "dataset", "export", "--geoul", geoul, "--out", out_name]
                    if st_args.get("format"):
                        cmd += ["--format", st_args.get("format")]
                    if st_args.get("env_id"):
                        cmd += ["--env-id", st_args.get("env_id")]
                elif kind == "workshop_gen":
                    geoul = st_args.get("geoul")
                    out_name = st_args.get("out")
                    if not geoul or not out_name:
                        print(f"[{name}] workshop_gen requires args geoul/out", file=sys.stderr)
                        rc = 99
                        break
                    cmd = [args.teul_cli, "workshop", "gen", "--geoul", geoul, "--out", out_name]
                elif kind == "workshop_apply":
                    workshop = st_args.get("workshop")
                    patch = st_args.get("patch")
                    if not workshop or not patch:
                        print(f"[{name}] workshop_apply requires args workshop/patch", file=sys.stderr)
                        rc = 99
                        break
                    cmd = [args.teul_cli, "workshop", "apply", "--workshop", workshop, "--patch", patch]
                elif kind == "workshop_open":
                    workshop = st_args.get("workshop")
                    if not workshop:
                        print(f"[{name}] workshop_open requires args workshop", file=sys.stderr)
                        rc = 99
                        break
                    cmd = [args.teul_cli, "workshop", "open", "--workshop", workshop]
                else:
                    print(f"[{name}] Unsupported pipeline step: {kind}", file=sys.stderr)
                    rc = 99
                    break
                rc, out, err = run_cmd(cmd, cwd=str(tdir), stdin_text=stdin_text)
                full_out += out
                full_err += err
                if kind == "run":
                    (
                        last_user_lines,
                        last_state_hash,
                        last_trace_hash,
                        last_bogae_hash,
                    ) = extract_hashes_and_lines(out)
                elif kind in ("replay_verify", "replay_branch", "geoul_query", "geoul_backtrace"):
                    last_user_lines = [ln.rstrip("\n") for ln in out.splitlines()]
                if rc != 0:
                    break
            results[name] = {
                "exit": rc,
                "stdout_lines": last_user_lines,
                "stdout_text": full_out,
                "stderr": full_err,
                "state_hash": last_state_hash,
                "trace_hash": last_trace_hash,
                "bogae_hash": last_bogae_hash,
            }
            # then validate below as normal
            rc = results[name]["exit"]
            user_lines = results[name]["stdout_lines"]
            err = results[name]["stderr"]
            state_hash = results[name]["state_hash"]
            trace_hash = results[name]["trace_hash"]
            bogae_hash = results[name]["bogae_hash"]
            # fallthrough to validation
        else:
            print(f"[{name}] Unsupported command: {cmd_kind}", file=sys.stderr)
            failed += 1
            continue

        if cmd_kind in (
            "run",
            "canon",
            "lint",
            "preproc",
            "check",
            "build",
            "ai_extract",
            "ai_prompt",
            "view",
            "replay_diff",
            "replay_verify",
            "replay_branch",
            "geoul_query",
            "geoul_backtrace",
            "bdl_packet",
            "patch_preview",
            "patch_approve",
            "patch_apply",
            "patch_verify",
            "patch_propose",
            "intent_inspect",
            "intent_mock",
            "intent_merge",
            "story_make",
            "timeline_make",
            "goal_parse",
            "goal_plan",
            "goap_plan",
            "observation_canon",
            "nurigym_spec",
            "nurigym_view",
            "latency_simulate",
            "safety_check",
            "dataset_export",
            "workshop_gen",
            "workshop_apply",
            "workshop_open",
        ):
            rc, out, err = run_cmd(cmd, cwd=str(tdir), stdin_text=stdin_text)
            if cmd_kind == "run":
                user_lines, state_hash, trace_hash, bogae_hash = extract_hashes_and_lines(out)
                stdout_text = out
            elif cmd_kind in ("replay_verify", "replay_branch", "geoul_query", "geoul_backtrace"):
                user_lines = [ln.rstrip("\n") for ln in out.splitlines()]
                state_hash, trace_hash, bogae_hash = None, None, None
                stdout_text = out
            else:
                user_lines, state_hash, trace_hash, bogae_hash = [], None, None, None
                stdout_text = out

            results[name] = {
                "exit": rc,
                "stdout_lines": user_lines,
                "stdout_text": stdout_text,
                "stderr": err,
                "state_hash": state_hash,
                "trace_hash": trace_hash,
                "bogae_hash": bogae_hash,
            }

        exp = spec.get("expect", {})
        exp_exit = exp.get("exit", 0)
        ok = True

        # exit
        if results[name]["exit"] != exp_exit:
            ok = False

        # stdout lines (for run)
        exp_stdout_lines = exp.get("stdout")
        if exp_stdout_lines is not None:
            if results[name]["stdout_lines"] != exp_stdout_lines:
                ok = False

        # canon/preproc output exact compare if provided
        exp_stdout_text = exp.get("stdout_text")
        if exp_stdout_text is not None:
            if results[name]["stdout_text"] != exp_stdout_text:
                ok = False

        # stderr contains
        for needle in exp.get("stderr_contains", []):
            if needle not in results[name]["stderr"]:
                ok = False

        # files exist
        missing = ensure_files_exist(tdir, exp.get("files_exist", []))
        if missing:
            ok = False

        file_failures = check_files_expectations(tdir, exp.get("files", {}))
        if file_failures:
            ok = False

        # hash expectations (run only)
        if cmd_kind in ("run","pipeline","multi"):
            for hk in ("state_hash","trace_hash"):
                token = exp.get(hk)
                mode, other = resolve_expect(token, results, hk)
                actual = results[name][hk]

                if mode == "ANY":
                    if actual is None:
                        ok = False
                    elif args.bless and token == "ANY":
                        exp[hk] = actual
                elif mode == "ABSENT":
                    if actual is not None:
                        ok = False
                elif mode == "EXACT":
                    if actual != token.lower():
                        ok = False
                elif mode == "SAME_AS":
                    other_actual = results.get(other, {}).get(hk)
                    if other_actual is None or actual is None or actual != other_actual:
                        ok = False
                elif mode == "DIFFERS_FROM":
                    other_actual = results.get(other, {}).get(hk)
                    if other_actual is None or actual is None or actual == other_actual:
                        ok = False

            if "bogae_hash" in exp:
                token = exp.get("bogae_hash")
                mode, other = resolve_expect(token, results, "bogae_hash")
                actual = results[name]["bogae_hash"]

                if mode == "ANY":
                    if actual is None:
                        ok = False
                    elif args.bless and token == "ANY":
                        exp["bogae_hash"] = actual
                elif mode == "ABSENT":
                    if actual is not None:
                        ok = False
                elif mode == "EXACT":
                    if actual != token.lower():
                        ok = False
                elif mode == "SAME_AS":
                    other_actual = results.get(other, {}).get("bogae_hash")
                    if other_actual is None or actual is None or actual != other_actual:
                        ok = False
                elif mode == "DIFFERS_FROM":
                    other_actual = results.get(other, {}).get("bogae_hash")
                    if other_actual is None or actual is None or actual == other_actual:
                        ok = False

        # write back blessed hashes
        if args.bless and cmd_kind in ("run","pipeline","multi"):
            # update spec in-place
            if "expect" not in spec:
                spec["expect"] = exp
            else:
                spec["expect"].update(exp)
            bless_writeback(spec)

        if ok:
            print(f"PASS {name}")
        else:
            failed += 1
            print(f"FAIL {name}")
            print("--- stdout_lines ---")
            print("\n".join(results[name]["stdout_lines"]))
            print("--- stdout_text ---")
            print(results[name]["stdout_text"])
            print("--- stderr ---", file=sys.stderr)
            print(results[name]["stderr"], file=sys.stderr)
            if missing:
                print(f"--- missing files: {missing} ---", file=sys.stderr)
            if file_failures:
                print(f"--- file expectation failures: {file_failures} ---", file=sys.stderr)

    print(f"\nSummary: {len(tests)-failed} passed / {failed} failed / {len(tests)} total")
    return 0 if failed == 0 else 1

if __name__ == "__main__":
    raise SystemExit(main())
